use std::error::Error;
use std::fmt::{self, Display};

use log::info;
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{Sqlite, SqlitePool},
    Executor, FromRow, Transaction,
};

use crate::date::DateId;
use crate::lectionary::{Lectionary, Reading};
use crate::path;

pub struct Database {
    connection: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let db_url = Self::get_db_url()?;
        let pool = Self::init_db(&db_url).await?;

        Ok(Self { connection: pool })
    }

    pub async fn init_db(db_url: &str) -> Result<SqlitePool, Box<dyn Error>> {
        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            info!("Creating new database at {}", &db_url);
            Sqlite::create_database(db_url).await?;
        }
        let pool = SqlitePool::connect(db_url).await?;
        pool.execute("PRAGMA foreign_keys = ON;").await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(pool)
    }

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

    pub async fn get_lectionary(&self, id: &DateId) -> Result<Lectionary, Box<dyn Error>> {
        let lect_row = sqlx::query_as::<_, LectionaryRow>("SELECT id, name FROM lectionary WHERE id = $1 LIMIT 1")
            .bind(id.as_str())
            .fetch_optional(&self.connection)
            .await?
            .ok_or("Not Present")?;

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

    async fn get_reading_row(&self, lect_id: &DateId, reading_type: DbReadingType) -> Result<ReadingRow, sqlx::Error> {
        sqlx::query_as::<_, ReadingRow>("SELECT location, content FROM reading WHERE lectionary_id=$1 AND reading_type=$2 LIMIT 1")
            .bind(lect_id.as_str())
            .bind(reading_type.as_str())
            .fetch_one(&self.connection)
            .await
    }

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

    fn get_db_url() -> Result<String, Box<dyn Error>> {
        let mut db_url = String::from("sqlite://");
        let file_path = path::create_and_get_db_path()?;

        db_url.push_str(file_path.to_str().expect("file path must be valid string"));

        Ok(db_url)
    }
}

pub struct LectionaryDbEntity {
    pub lect_row: LectionaryRow,
    pub first_reading_row: ReadingRow,
    pub psalm_row: ReadingRow,
    pub gospel_row: ReadingRow,
    pub second_reading_row: Option<ReadingRow>,
}

#[derive(Debug, FromRow)]
pub struct LectionaryRow {
    pub id: DateId,
    pub name: String,
}

#[derive(Debug, FromRow)]
pub struct ReadingRow {
    pub location: String,
    pub content: String,
}

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
