use std::fmt::{self, Display};

use log::*;
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{Sqlite, SqlitePool},
    Executor, FromRow, Row, Transaction,
};

use crate::date::DateId;
use crate::error::{DatabaseGetError, DatabaseInitError};
use crate::lectionary::{Lectionary, Reading};
use crate::path::{self};

/// A wrapper over a Sqlite pool that defines functions for working with the database
#[derive(Clone)]
pub struct DatabaseHandle {
    connection: SqlitePool,
}

impl DatabaseHandle {
    pub async fn new() -> Result<Self, DatabaseInitError> {
        let db_url = Self::get_db_url()?;
        let pool = Self::init_db(&db_url).await?;

        Ok(Self { connection: pool })
    }

    /// Inserts a lectionary data into the lectionary and readings tables
    pub async fn insert_lectionary(&self, lectionary: &Lectionary) -> Result<(), sqlx::Error> {
        let mut transaction = self.connection.begin().await?;

        let id = lectionary.get_id();

        let insert_lect = sqlx::query("INSERT OR REPLACE INTO lectionary (id, name) VALUES ($1, $2)")
            .bind(id.as_str())
            .bind(lectionary.get_day_name());
        transaction.execute(insert_lect).await?;

        Self::insert_reading(&mut transaction, lectionary.get_reading_1(), id, DbReadingType::FirstReading).await?;
        Self::insert_reading(&mut transaction, lectionary.get_resp_psalm(), id, DbReadingType::Psalm).await?;
        Self::insert_reading(&mut transaction, lectionary.get_gospel(), id, DbReadingType::Gospel).await?;
        if let Some(reading_2) = lectionary.get_reading_2() {
            Self::insert_reading(&mut transaction, reading_2, id, DbReadingType::SecondReading).await?;
        }

        transaction.commit().await
    }

    /// Gets a lectionary from the database
    ///
    /// Requires reading from both the lectionary table and then the readings table
    pub async fn get_lectionary(&self, id: &DateId) -> Result<Lectionary, DatabaseGetError> {
        let lect_row = sqlx::query_as::<_, LectionaryRow>("SELECT id, name FROM lectionary WHERE id = $1 LIMIT 1")
            .bind(id.as_str())
            .fetch_optional(&self.connection)
            .await?
            .ok_or(DatabaseGetError::NotPresent)?;

        let first_reading_row = self.get_reading_row(id, DbReadingType::FirstReading).await?;
        let psalm_row = self.get_reading_row(id, DbReadingType::Psalm).await?;
        let gospel_row = self.get_reading_row(id, DbReadingType::Gospel).await?;
        let second_reading_row = self.get_reading_row(id, DbReadingType::SecondReading).await.ok();

        let entity = LectionaryDbEntity {
            lect_row,
            first_reading_row,
            psalm_row,
            gospel_row,
            second_reading_row,
        };

        Ok(Lectionary::from(entity))
    }

    /// Removes a single lectionary by its `DateId`
    pub async fn remove_lectionary(&self, id: &DateId) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM lectionary WHERE id=$1")
            .bind(id.as_str())
            .execute(&self.connection)
            .await?;

        match result.rows_affected() {
            0 => Ok(false),
            1 => Ok(true),
            rows => {
                // This should never happen
                warn!("Query should have removed 1 row but removed {}", rows);
                Ok(true)
            }
        }
    }

    /// Deletes the entire lectionary table
    ///
    /// To be used with the 'db purge' command
    pub async fn remove_all(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM lectionary").execute(&self.connection).await?;

        Ok(result.rows_affected())
    }

    /// Deletes entries outside a given range
    ///
    /// If latest is None, it only deletes old entries
    /// Returns the number of Succesfully removed rows
    /// Only fails if cannot GET ids. Failure to remove will write errors to log but return Ok
    pub async fn remove_outside_range(&self, earliest: DateId, maybe_latest: Option<DateId>) -> Result<u64, sqlx::Error> {
        let all_ids = sqlx::query_as::<_, DateId>("SELECT id FROM lectionary")
            .fetch_all(&self.connection)
            .await?;
        //TODO remove new ones too
        let ids_outside_range = all_ids
            .into_iter()
            .filter(|id| id < &earliest || maybe_latest.as_ref().is_some_and(|latest| *id > *latest));
        let mut count_removed = 0;
        for id in ids_outside_range {
            if let Err(e) = self.remove_lectionary(&id).await {
                error!("Failed to remove lectionary '{}' ({})", id, e);
            } else {
                info!("Succesfully removed lectionary '{}' during clean operation", id);
                count_removed += 1;
            }
        }
        Ok(count_removed)
    }

    /// Returns a count of the number of rows in the lectionary table
    pub async fn get_lectionary_count(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("SELECT COUNT(*) FROM lectionary").fetch_one(&self.connection).await?;

        // Must cast from i64 because u64 does not implement Encode
        result
            .try_get::<'_, i64, _>(0)
            .map(|signed| u64::try_from(signed).expect("Row count should never be negative"))
    }

    /// Determines if a lectionary with a given id is present
    ///
    /// More efficient than `get_lectionary` because it doesn't try to decode the whole reading
    pub async fn lectionary_present(&self, id: &DateId) -> Result<bool, sqlx::Error> {
        sqlx::query("SELECT id FROM lectionary WHERE id=$1")
            .bind(id.as_str())
            .fetch_optional(&self.connection)
            .await
            .map(|success| success.is_some())
    }

    /// Gets all of the rows from the lectionary table
    ///
    /// Does not touch the reading table
    pub async fn get_lectionary_rows(&self) -> Result<Vec<LectionaryRow>, sqlx::Error> {
        sqlx::query_as::<_, LectionaryRow>("SELECT id, name FROM lectionary")
            .fetch_all(&self.connection)
            .await
    }

    /// Gets a reading row for a specified lectionary with a given type
    async fn get_reading_row(&self, lect_id: &DateId, reading_type: DbReadingType) -> Result<ReadingRow, sqlx::Error> {
        sqlx::query_as::<_, ReadingRow>("SELECT location, content FROM reading WHERE lectionary_id=$1 AND reading_type=$2 LIMIT 1")
            .bind(lect_id.as_str())
            .bind(reading_type.as_str())
            .fetch_one(&self.connection)
            .await
    }

    /// Inserts a single reading into the reading table
    async fn insert_reading(
        transaction: &mut Transaction<'_, Sqlite>,
        reading: &Reading,
        lectionary_id: &DateId,
        reading_type: DbReadingType,
    ) -> Result<(), sqlx::Error> {
        let insert_reading = sqlx::query("INSERT INTO reading (lectionary_id, reading_type, location, content) VALUES ($1, $2, $3, $4)")
            .bind(lectionary_id.as_str())
            .bind(reading_type.as_str())
            .bind(reading.get_location())
            .bind(reading.get_text());
        transaction.execute(insert_reading).await?;
        Ok(())
    }

    /// Initializes a connection to the Sqlite database after ensuring it exists
    async fn init_db(db_url: &str) -> Result<SqlitePool, DatabaseInitError> {
        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            warn!("Creating new database at '{}' (You should only see this once)", &db_url);
            Sqlite::create_database(db_url)
                .await
                .map_err(DatabaseInitError::CreateDatabaseError)?;
        }
        let pool = SqlitePool::connect(db_url).await.map_err(DatabaseInitError::PoolCreationFailed)?;
        // Without this PRAGMA statement, foreign key constraints are not enforced. This would mean we could end up with orphan readings
        pool.execute("PRAGMA foreign_keys = ON;")
            .await
            .map_err(DatabaseInitError::PragmaForeignKeysFailure)?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(DatabaseInitError::MigrationError)?;

        Ok(pool)
    }

    /// Creates the path to the Sqlite database
    fn get_db_url() -> Result<String, DatabaseInitError> {
        let mut db_url = String::from("sqlite://");
        let file_path = path::create_and_get_db_path().map_err(DatabaseInitError::CannotGetUrl)?;

        //TODO may need to look in to this expect statement when implementing Windows support
        db_url.push_str(file_path.to_str().expect("file path should be valid string"));

        Ok(db_url)
    }
}

/// Intermediate struct used for creating a ```Lectionary``` struct
pub struct LectionaryDbEntity {
    pub lect_row: LectionaryRow,
    pub first_reading_row: ReadingRow,
    pub psalm_row: ReadingRow,
    pub gospel_row: ReadingRow,
    pub second_reading_row: Option<ReadingRow>,
}

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct LectionaryRow {
    pub id: DateId,
    pub name: String,
}

impl PartialOrd for LectionaryRow {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LectionaryRow {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug, FromRow)]
pub struct ReadingRow {
    pub location: String,
    pub content: String,
}

//TODO I have three enums that more or less serve the same function. Should maybe fix that
enum DbReadingType {
    FirstReading,
    SecondReading,
    Psalm,
    Gospel,
}

impl DbReadingType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::FirstReading => "first_reading",
            Self::SecondReading => "second_reading",
            Self::Psalm => "psalm",
            Self::Gospel => "gospel",
        }
    }
}

impl Display for DbReadingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
