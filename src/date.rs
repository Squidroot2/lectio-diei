use std::cmp::Ordering;
use std::fmt::{self, Display};

use chrono::format::ParseError;
use chrono::{DateTime, Local, NaiveDate, TimeDelta};
use sqlx::FromRow;
use sqlx::{
    sqlite::{Sqlite, SqliteValueRef},
    Decode, Type,
};

/// The str used for chrono formatting from date to `DateId`.
/// Represents a format like 040124 (April 1st, 2024)
const DATE_ID_FORMAT: &str = "%m%d%y";

/// Type-checked `String` used for url retrieval and database ids
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct DateId {
    id: String,
}

impl DateId {
    /// Reference to inner value
    /// Use this for binding to `sqlx` queries because implementing the `Encode` trait is more work than it's worth
    pub fn as_str(&self) -> &str {
        &self.id
    }

    /// Gets the `DateId` for today, local time
    pub fn today() -> Self {
        Self::from_local_datetime(&Local::now())
    }

    /// Checks that a given `str` is a valid `DateId` before returning it
    ///
    /// First converts to a `NaiveDate`, then  back to a `String` for storage within `DateId` struct
    pub fn checked_from_str(date_string: &str) -> Result<Self, ParseError> {
        let date = NaiveDate::parse_from_str(date_string, DATE_ID_FORMAT)?;
        Ok(Self::from_date(date))
    }

    /// Gets a list of `DateId`s for a range
    ///
    /// Note that `future_days` must include today (i.e. if future days is 0, today will not be included)
    pub fn get_list(past_days: u32, future_days: u32) -> Vec<DateId> {
        let length = past_days + future_days;
        let mut list = Vec::with_capacity(length as usize);
        let today = Local::now();
        for delta in (0 - i64::from(past_days))..i64::from(future_days) {
            let date = today + TimeDelta::days(delta);
            list.push(DateId::from_local_datetime(&date));
        }
        list
    }

    /// Returns a `DateId` for given local `DateTime`
    pub fn from_local_datetime(date: &DateTime<Local>) -> Self {
        let id = date.format(DATE_ID_FORMAT).to_string();
        Self { id }
    }

    /// Returns as `DateId` for given `NaiveDate`
    fn from_date(date: NaiveDate) -> Self {
        let id = date.format(DATE_ID_FORMAT).to_string();
        Self { id }
    }
}

impl Display for DateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'r> Decode<'r, Sqlite> for DateId {
    fn decode(value_ref: SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let id = <&str as Decode<Sqlite>>::decode(value_ref)?.to_owned();
        debug_assert_eq!(6, id.len());
        debug_assert!(id.chars().all(char::is_numeric));

        Ok(Self { id })
    }
}

impl Type<Sqlite> for DateId {
    fn type_info() -> <Sqlite as sqlx::Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl PartialOrd for DateId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compares years, then month-day
        match self.id[4..6].cmp(&other.id[4..6]) {
            Ordering::Equal => self.id[0..4].cmp(&other.id[0..4]),
            ord => ord,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn get_date_string_correct() {
        let date = Local.with_ymd_and_hms(2024, 7, 14, 0, 0, 0).unwrap();
        let date_id = DateId::from_local_datetime(&date);
        assert_eq!(date_id.as_str(), "071424");
    }

    #[test]
    fn checked_from_str_success() {
        let date = "070707";
        let id = DateId::checked_from_str(date);
        assert_eq!(date, id.unwrap().as_str());
    }

    #[test]
    fn checked_from_str_error() {
        assert!(DateId::checked_from_str("0707071").is_err());
        assert!(DateId::checked_from_str("0707").is_err());
        assert!(DateId::checked_from_str("June12").is_err());
    }

    #[test]
    fn get_list_correct_length() {
        let list = DateId::get_list(5, 3);
        assert_eq!(8, list.len());
    }
}
