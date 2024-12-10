use std::fmt::{self, Display, Formatter};

use log::*;

use crate::date::DateId;
use crate::db::{LectionaryDbEntity, ReadingRow};
use crate::error::ReadingNameFromStringError;

#[derive(Debug)]
pub struct Lectionary {
    id: DateId,
    day_name: String,
    reading_1: Reading,
    reading_2: Option<Reading>,
    resp_psalm: Reading,
    gospel: Reading,
    alleluia: Reading,
}

impl Lectionary {
    pub fn new(
        id: DateId,
        day_name: String,
        reading_1: Reading,
        reading_2: Option<Reading>,
        resp_psalm: Reading,
        gospel: Reading,
        alleluia: Reading,
    ) -> Self {
        Self {
            id,
            day_name,
            reading_1,
            reading_2,
            resp_psalm,
            gospel,
            alleluia,
        }
    }

    pub fn get_id(&self) -> &DateId {
        &self.id
    }
    pub fn get_day_name(&self) -> &str {
        &self.day_name
    }
    pub fn get_reading_1(&self) -> &Reading {
        &self.reading_1
    }
    pub fn get_resp_psalm(&self) -> &Reading {
        &self.resp_psalm
    }
    pub fn get_gospel(&self) -> &Reading {
        &self.gospel
    }
    pub fn get_reading_2(&self) -> Option<&Reading> {
        self.reading_2.as_ref()
    }
    pub fn get_alleluia(&self) -> &Reading {
        &self.alleluia
    }
}

impl From<LectionaryDbEntity> for Lectionary {
    fn from(entity: LectionaryDbEntity) -> Self {
        Lectionary {
            id: entity.lect_row.id,
            day_name: entity.lect_row.name,
            reading_1: Reading::from(entity.first_reading_row),
            reading_2: entity.second_reading_row.map(Reading::from),
            resp_psalm: Reading::from(entity.psalm_row),
            gospel: Reading::from(entity.gospel_row),
            alleluia: Reading::from(entity.alleluia_row),
        }
    }
}

#[derive(Debug)]
pub enum ReadingName {
    Reading1,
    Reading2,
    Psalm,
    Gospel,
    Alleluia,
}
impl ReadingName {
    const READING1: &'static str = "Reading I";
    const READING2: &'static str = "Reading II";
    const PSALM: &'static str = "Responsorial Psalm";
    const GOSPEL: &'static str = "Gospel";
    const ALLELUIA: &'static str = "Alleluia";

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reading1 => Self::READING1,
            Self::Reading2 => Self::READING2,
            Self::Psalm => Self::PSALM,
            Self::Gospel => Self::GOSPEL,
            Self::Alleluia => Self::ALLELUIA,
        }
    }
}
impl Display for ReadingName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl TryFrom<String> for ReadingName {
    type Error = ReadingNameFromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        trace!("trimmed reading value: {}", trimmed);
        match trimmed {
            Self::READING1 | "Reading 1" => Ok(Self::Reading1),
            Self::READING2 | "Reading 2" => Ok(Self::Reading2),
            Self::PSALM => Ok(Self::Psalm),
            Self::GOSPEL => Ok(Self::Gospel),
            Self::ALLELUIA | "Alleluia See" => Ok(Self::Alleluia),
            _ => Err(Self::Error::from(value)),
        }
    }
}

#[derive(Debug)]
pub struct Reading {
    location: String,
    text: String,
}
impl Reading {
    pub fn new(location: String, text: String) -> Self {
        Self { location, text }
    }

    pub fn get_location(&self) -> &str {
        &self.location
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}
impl From<ReadingRow> for Reading {
    fn from(row: ReadingRow) -> Self {
        Self {
            location: row.location,
            text: row.content,
        }
    }
}
