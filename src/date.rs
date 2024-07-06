use std::fmt::{self, Display};

use chrono::format::ParseError;
use chrono::{DateTime, Local, NaiveDate};
use sqlx::{
    sqlite::{Sqlite, SqliteValueRef},
    Decode, Type,
};

/// Type checked String used for url retrieval and database ids
#[derive(Debug)]
pub struct DateId {
    value: String,
}

impl DateId {
    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn today() -> Self {
        Self::from(&Local::now())
    }

    pub fn checked_from_str(date_string: &str) -> Result<Self, ParseError> {
        let date = NaiveDate::parse_from_str(date_string, "%m%d%y")?;
        Ok(Self::from(&date))
    }
}

impl Display for DateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&DateTime<Local>> for DateId {
    fn from(date: &DateTime<Local>) -> Self {
        let value = date.format("%m%d%y").to_string();
        Self { value }
    }
}

impl From<&NaiveDate> for DateId {
    fn from(date: &NaiveDate) -> Self {
        let value = date.format("%m%d%y").to_string();
        Self { value }
    }
}

impl<'r> Decode<'r, Sqlite> for DateId {
    fn decode(value_ref: SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as Decode<Sqlite>>::decode(value_ref)?.to_owned();
        debug_assert_eq!(6, value.len());
        debug_assert!(value.chars().all(|c| c.is_numeric()));

        Ok(Self { value })
    }
}

impl Type<Sqlite> for DateId {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn get_date_string_correct() {
        let date = Local.with_ymd_and_hms(2024, 07, 14, 0, 0, 0).unwrap();
        let date_id = DateId::from(&date);
        assert_eq!(date_id.as_str(), "071424");
    }
}
