use std::fmt::{self, Display, Formatter};

use log::*;
use once_cell::sync::Lazy;
use scraper::element_ref::ElementRef;
use scraper::selectable::Selectable;
use scraper::selector::Selector;
use scraper::Html;

use crate::date::DateId;
use crate::db::{LectionaryDbEntity, ReadingRow};
use crate::error::{LectionaryHtmlError, ReadingHtmlError, ReadingNameFromStringError};
use crate::html;

/// Main container in which all other relevant elements are found
static CONTAINER_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("#block-usccb-readings-content div.page-container").unwrap());
/// Use within element found by CONTAINER_SELECTOR. Finds the element that has the name of the day (e.g. Fourteenth Sunday in Ordinary Time )
static DAY_NAME_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.b-lectionary div.innerblock :first-child").unwrap());
/// Use within element found by CONTAINER_SELECTOR. Finds all the verse(aka reading) containers
static READINGS_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.b-verse").unwrap());

#[derive(Debug)]
pub struct Lectionary {
    id: DateId,
    day_name: String,
    reading_1: Reading,
    reading_2: Option<Reading>,
    resp_psalm: Reading,
    gospel: Reading,
}

impl Lectionary {
    pub fn create_from_html(id: DateId, document: Html) -> Result<Self, LectionaryHtmlError> {
        let container = document
            .select(&CONTAINER_SELECTOR)
            .next()
            .ok_or(LectionaryHtmlError::NoContainerFound)?;
        let day_name_elmnt = container
            .select(&DAY_NAME_SELECTOR)
            .next()
            .ok_or(LectionaryHtmlError::NoDayNameElementFound)?;
        let day_name = day_name_elmnt.inner_html().trim().to_owned();

        let readings = ParsedReadings::extract_from_container(container);
        let reading_1 = readings
            .reading_1
            .ok_or(LectionaryHtmlError::MissingReading(ReadingName::Reading1))?;
        let reading_2 = readings.reading_2;
        let resp_psalm = readings.resp_psalm.ok_or(LectionaryHtmlError::MissingReading(ReadingName::Psalm))?;
        let gospel = readings.gospel.ok_or(LectionaryHtmlError::MissingReading(ReadingName::Gospel))?;

        Ok(Lectionary {
            id,
            day_name,
            reading_1,
            reading_2,
            resp_psalm,
            gospel,
        })
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
    pub fn pretty_print(&self) {
        let dash_length = self.day_name.len() + 4;
        let mut dashes = String::new();
        for _ in 0..dash_length {
            dashes.push('-')
        }
        println!("{}", dashes);
        println!("| {} |", self.day_name);
        println!("{}", dashes);

        println!("{} ({})", ReadingName::READING1, self.reading_1.location);
        println!("{}", dashes);
        //TODO clean up the stored text so that there is no need to replace new lines
        println!("{}", self.reading_1.text.replace('\n', " "));

        println!("{}", dashes);
        println!("{} ({})", ReadingName::PSALM, self.resp_psalm.location);
        println!("{}", dashes);
        println!("{}", self.resp_psalm.text);

        println!("{}", dashes);
        println!("{} ({})", ReadingName::GOSPEL, self.gospel.location);
        println!("{}", dashes);
        println!("{}", self.gospel.text.replace('\n', " "))
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
        }
    }
}

/// Use within a element found by READINGS_SELECTOR
static READING_NAME_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(".name").unwrap());

/// For temporary use while constructing a Lectionary from html
#[derive(Default)]
struct ParsedReadings {
    reading_1: Option<Reading>,
    reading_2: Option<Reading>,
    resp_psalm: Option<Reading>,
    gospel: Option<Reading>,
}

impl ParsedReadings {
    fn extract_from_container(container: ElementRef<'_>) -> Self {
        let mut out = ParsedReadings::default();

        let readings = container.select(&READINGS_SELECTOR);
        for reading_elmt in readings {
            if let Some(name_elmnt) = reading_elmt.select(&READING_NAME_SELECTOR).next() {
                match ReadingName::try_from(html::replace_entities(name_elmnt.inner_html())) {
                    Ok(name) => match Reading::from_container(reading_elmt) {
                        Ok(reading) => match name {
                            ReadingName::Reading1 => out.reading_1 = Some(reading),
                            ReadingName::Reading2 => out.reading_2 = Some(reading),
                            ReadingName::Psalm => out.resp_psalm = Some(reading),
                            ReadingName::Gospel => out.gospel = Some(reading),
                            ReadingName::Alleluia => {}
                        },
                        Err(e) => error!("Failed to process element '{}'; Reason: {}", name, e),
                    },
                    Err(e) => error!("Failed to convert name to ReadingName: {}", e),
                };
            } else {
                error!("Found reading element with no name element");
            }
        } // END FOR LOOP

        out
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
    const READING1: &'static str = "Reading 1";
    const READING2: &'static str = "Reading 2";
    const PSALM: &'static str = "Responsorial Psalm";
    const GOSPEL: &'static str = "Gospel";
    const ALLELUIA: &'static str = "Alleluia";

    fn as_str(&self) -> &'static str {
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
            Self::READING1 => Ok(Self::Reading1),
            Self::READING2 => Ok(Self::Reading2),
            Self::PSALM => Ok(Self::Psalm),
            Self::GOSPEL => Ok(Self::Gospel),
            Self::ALLELUIA => Ok(Self::Alleluia),
            _ => Err(Self::Error::from(value)),
        }
    }
}

/// Use within element found by READINGS_SELECTOR. The container with the actual text of the reading.
static READING_CONTENT_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.content-body").unwrap());

/// Use within element found by READINGS_SELECTOR. Finds the address (book, chapter, verse(s)) of the reading
static READING_LOCATION_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div.content-header div.address a").unwrap());

#[derive(Debug)]
pub struct Reading {
    location: String,
    text: String,
}
impl Reading {
    pub fn get_location(&self) -> &str {
        &self.location
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    fn from_container(reading_container: ElementRef<'_>) -> Result<Self, ReadingHtmlError> {
        let location_elmt = reading_container
            .select(&READING_LOCATION_SELECTOR)
            .next()
            .ok_or(ReadingHtmlError::MissingLocation)?;
        let location = html::replace_entities(location_elmt.inner_html());
        let content = reading_container
            .select(&READING_CONTENT_SELECTOR)
            .next()
            .ok_or(ReadingHtmlError::MissingContent)?;
        let text = html::element_to_plain_text(&content);

        Ok(Reading { location, text })
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
