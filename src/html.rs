use std::collections::HashMap;
use std::sync::OnceLock;

use log::*;
use scraper::selectable::Selectable;
use scraper::selector::ToCss;
use scraper::ElementRef;
use scraper::Html;
use scraper::Node;
use scraper::Selector;

use crate::date::DateId;
use crate::lectionary::Lectionary;
use crate::lectionary::Reading;
use crate::lectionary::ReadingName;

/// Main container in which all other relevant elements are found
fn container_selector() -> &'static Selector {
    static CONTAINER_SELECTOR: OnceLock<Selector> = OnceLock::new();
    CONTAINER_SELECTOR.get_or_init(|| Selector::parse("#block-usccb-readings-content div.page-container").unwrap())
}
/// Use within element found by `CONTAINER_SELECTOR`. Finds the element that has the name of the day (e.g. Fourteenth Sunday in Ordinary Time )
fn day_name_selector() -> &'static Selector {
    static DAY_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
    DAY_NAME_SELECTOR.get_or_init(|| Selector::parse("div.b-lectionary div.innerblock :first-child").unwrap())
}
/// Use within element found by `CONTAINER_SELECTOR`. Finds all the verse(aka reading) containers
fn readings_selector() -> &'static Selector {
    static READINGS_SELECTOR: OnceLock<Selector> = OnceLock::new();
    READINGS_SELECTOR.get_or_init(|| Selector::parse("div.b-verse").unwrap())
}
/// Use within a element found by `READINGS_SELECTOR`
fn reading_name_selector() -> &'static Selector {
    static READING_NAME_SELECTOR: OnceLock<Selector> = OnceLock::new();
    READING_NAME_SELECTOR.get_or_init(|| Selector::parse(".name").unwrap())
}
/// Use within element found by `READINGS_SELECTOR`. The container with the actual text of the reading.
fn reading_content_selector() -> &'static Selector {
    static READING_CONTENT_SELECTOR: OnceLock<Selector> = OnceLock::new();
    READING_CONTENT_SELECTOR.get_or_init(|| Selector::parse("div.content-body").unwrap())
}
/// Use within element found by `READINGS_SELECTOR`. Finds the address (book, chapter, verse(s)) of the reading
fn reading_location_selector() -> &'static Selector {
    static READING_LOCATION_SELECTOR: OnceLock<Selector> = OnceLock::new();
    READING_LOCATION_SELECTOR.get_or_init(|| Selector::parse("div.content-header div.address a").unwrap())
}
/// Use within element found by `CONTAINER_SELECTOR`. On a holiday page, finds the link for the day time reading
fn day_link_selector() -> &'static Selector {
    static DAY_LINK_SELECTOR: OnceLock<Selector> = OnceLock::new();
    DAY_LINK_SELECTOR.get_or_init(|| Selector::parse("div.b-lectionary div.innerblock a[href$=\"day.cfm\" i ]").unwrap())
}

impl Lectionary {
    pub fn create_from_html(id: DateId, document: &Html) -> Result<Self, LectionaryHtmlError> {
        let container = document
            .select(container_selector())
            .next()
            .ok_or_else(|| LectionaryHtmlError::NoContainerFound { date: id.clone() })?;
        let day_name_elmnt = container
            .select(day_name_selector())
            .next()
            .ok_or_else(|| LectionaryHtmlError::NoDayNameElementFound { date: id.clone() })?;
        // First line of the inner text
        let day_name = element_to_plain_text(&day_name_elmnt)
            .lines()
            .next()
            .expect("Will always have at least 1 line")
            .to_owned();

        let readings = ParsedReadings::extract_from_container(container);
        let reading_1 = readings.reading_1.ok_or_else(|| LectionaryHtmlError::MissingReading {
            reading: ReadingName::Reading1,
            date: id.clone(),
        })?;
        let reading_2 = readings.reading_2;
        let resp_psalm = readings.resp_psalm.ok_or_else(|| LectionaryHtmlError::MissingReading {
            reading: ReadingName::Psalm,
            date: id.clone(),
        })?;
        let gospel = readings.gospel.ok_or_else(|| LectionaryHtmlError::MissingReading {
            reading: ReadingName::Gospel,
            date: id.clone(),
        })?;
        let alleluia = readings.allelia.ok_or_else(|| LectionaryHtmlError::MissingReading {
            reading: ReadingName::Alleluia,
            date: id.clone(),
        })?;

        Ok(Lectionary::new(id, day_name, reading_1, reading_2, resp_psalm, gospel, alleluia))
    }
}

impl Reading {
    fn from_container(reading_container: ElementRef<'_>) -> Result<Self, ReadingHtmlError> {
        let location = if let Some(location_elmt) = reading_container.select(reading_location_selector()).next() {
            replace_entities(location_elmt.inner_html()).trim().to_owned()
        } else {
            warn!("No location element found for reading");
            String::new()
        };
        let content = reading_container
            .select(reading_content_selector())
            .next()
            .ok_or(ReadingHtmlError)?;
        let full_text = element_to_plain_text(&content);

        // Some reading will have alternates noted with "OR:". only take first
        let text = full_text.split("OR:\n").next().expect("Split will always have at least 1 element");

        Ok(Reading::new(location, text.to_owned()))
    }
}

/// For temporary use while constructing a `Lectionary` from html
#[derive(Default)]
struct ParsedReadings {
    reading_1: Option<Reading>,
    reading_2: Option<Reading>,
    resp_psalm: Option<Reading>,
    gospel: Option<Reading>,
    allelia: Option<Reading>,
}

impl ParsedReadings {
    fn extract_from_container(container: ElementRef<'_>) -> Self {
        let mut out = ParsedReadings::default();

        let readings = container.select(readings_selector());
        for reading_elmt in readings {
            debug!("parsing reading {:?} in container {:?}", reading_elmt, container);
            trace!("full reading elmnt: \n{}", reading_elmt.inner_html());
            if let Some(name_elmnt) = reading_elmt.select(reading_name_selector()).next() {
                debug!("Extracting reading name from reading name element {}", name_elmnt.html());
                match ReadingName::try_from(replace_entities(name_elmnt.inner_html())) {
                    Ok(name) => {
                        info!("Idenitfied reading name as '{name}'. Parsing reading...");
                        match Reading::from_container(reading_elmt) {
                            Ok(reading) => match name {
                                ReadingName::Reading1 => out.reading_1 = Some(reading),
                                ReadingName::Reading2 => out.reading_2 = Some(reading),
                                ReadingName::Psalm => out.resp_psalm = Some(reading),
                                ReadingName::Gospel => out.gospel = Some(reading),
                                ReadingName::Alleluia => out.allelia = Some(reading),
                            },
                            Err(e) => error!("Failed to process element '{name}'; Reason: {e}"),
                        }
                    }
                    Err(e) => warn!("Unable to identify reading name: {e}"),
                };
            } else {
                error!("Found reading element with no name element");
            }
        } // END FOR LOOP

        out
    }
}

/// Converts an element to plain text, removing tags like '\<strong\>' while keeping the text within those elements
fn element_to_plain_text(element: &ElementRef) -> String {
    let mut plain_text = String::new();
    for node in element.children() {
        match node.value() {
            Node::Text(text) => {
                plain_text.push_str(text.trim_matches('\n'));
            }
            Node::Element(element) => match element.name() {
                "br" => plain_text.push('\n'),
                "p" => {
                    plain_text.push('\n');
                    let elmt_ref = ElementRef::wrap(node).expect("Node of value Element will always wrap to ElementRef");
                    plain_text.push_str(&element_to_plain_text(&elmt_ref));
                }
                _ => {
                    let elmt_ref = ElementRef::wrap(node).expect("Node of value Element will always wrap to ElementRef");
                    plain_text.push_str(&element_to_plain_text(&elmt_ref));
                }
            },
            _ => {}
        }
    }
    // For some reason, the nodes start with large blocks of whitespace.
    plain_text.trim().to_string()
}

/// `HashMap` of expected html entities with their replacement character
fn html_entites() -> &'static HashMap<&'static str, &'static str> {
    static HTML_ENTITIES: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    HTML_ENTITIES.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert("&nbsp;", " ");
        map.insert("&amp;", "&");
        map.insert("&lt;", "<");
        map.insert("&gt;", ">");
        map.insert("&quot;", "\"");
        map.insert("&#39;", "'");
        map.insert("&apos;", "'");
        map
    })
}

/// Use when getting the inner text of an html element
pub fn replace_entities(mut value: String) -> String {
    for (entity, target) in html_entites() {
        if value.contains(entity) {
            value = value.replace(entity, target);
        }
    }
    value
}

/// If html doc is a holiday page, returns the endpoint for the day reading
pub fn get_holiday_day_reading_link(doc: &Html) -> Option<&str> {
    if let Some(container) = doc.select(container_selector()).next() {
        if let Some(day_link) = container.select(day_link_selector()).next() {
            info!("Found day reading for html document");
            Some(day_link.attr("href").expect("Found link must have href attribute"))
        } else {
            debug!("No day link ({}) found on html page", day_link_selector().to_css_string());
            None
        }
    } else {
        error!(
            "No main container ({}) found in html document",
            container_selector().to_css_string()
        );
        None
    }
}

/// Represents a failure to parse an HTML element into a Reading struct
#[derive(thiserror::Error, Debug)]
#[error("Missing Content from Reading")]
struct ReadingHtmlError;

/// Represents a failure to parse a HTML document in to a Lectionary struct
#[derive(thiserror::Error, Debug)]
pub enum LectionaryHtmlError {
    #[error("No main readings container found from {date}")]
    NoContainerFound { date: DateId },
    #[error("No day name element found from {date}")]
    NoDayNameElementFound { date: DateId },
    #[error("Missing required reading '{reading}' from {date}")]
    MissingReading { reading: ReadingName, date: DateId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn element_to_plain_text_works() {
        let html = Html::parse_fragment(r"<p>This is a <strong>test</strong> with some <br>extra&nbsp;stuff</p>");
        assert_eq!(
            element_to_plain_text(&html.root_element()),
            "This is a test with some \nextra\u{a0}stuff"
        );
    }

    #[test]
    fn element_to_plain_text_works_on_real() {
        let html = Html::parse_fragment(
            r#"
                <p>After entering a boat, Jesus made the crossing, and came into his own town.<br>
And there people brought to him a paralytic lying on a stretcher.<br>
When Jesus saw their faith, he said to the paralytic,<br>
"Courage, child, your sins are forgiven."<br>
At that, some of the scribes said to themselves,<br>
"This man is blaspheming."<br>
Jesus knew what they were thinking, and said,<br>
"Why do you harbor evil thoughts?<br>
Which is easier, to say, 'Your sins are forgiven,'<br>
or to say, 'Rise and walk'?<br>
But that you may know that the Son of Man<br>
has authority on earth to forgive sins"–<br>
he then said to the paralytic,<br>
"Rise, pick up your stretcher, and go home."<br>
He rose and went home.<br>
When the crowds saw this they were struck with awe<br>
and glorified God who had given such authority to men.</p>

<p>&nbsp;</p>"#,
        );
        assert_eq!(
            element_to_plain_text(&html.root_element()),
            r#"After entering a boat, Jesus made the crossing, and came into his own town.
And there people brought to him a paralytic lying on a stretcher.
When Jesus saw their faith, he said to the paralytic,
"Courage, child, your sins are forgiven."
At that, some of the scribes said to themselves,
"This man is blaspheming."
Jesus knew what they were thinking, and said,
"Why do you harbor evil thoughts?
Which is easier, to say, 'Your sins are forgiven,'
or to say, 'Rise and walk'?
But that you may know that the Son of Man
has authority on earth to forgive sins"–
he then said to the paralytic,
"Rise, pick up your stretcher, and go home."
He rose and went home.
When the crowds saw this they were struck with awe
and glorified God who had given such authority to men."#
        );
    }

    use std::{fs::File, io::Read, path::PathBuf};
    fn html_from_test_resource(file_name: &str) -> Html {
        let mut html_string = String::new();
        let mut path = PathBuf::new();
        path.push("tests");
        path.push("resources");
        path.push(file_name);

        File::open(path).unwrap().read_to_string(&mut html_string).unwrap();
        Html::parse_document(&html_string)
    }

    #[test]
    fn derialize_sunday_lectionary() {
        let html_doc = html_from_test_resource("sunday_or.html");
        let lectionary = Lectionary::create_from_html(DateId::today(), &html_doc).unwrap();
        assert_eq!(&DateId::today(), lectionary.get_id());
        assert!(lectionary.get_reading_2().is_some());
    }

    #[test]
    fn find_holiday_reading_link() {
        let html_doc = html_from_test_resource("assumption.html");
        let link = get_holiday_day_reading_link(&html_doc).unwrap();
        assert_eq!("/bible/readings/081524-day.cfm", link);
    }

    #[test]
    fn dont_find_holiday_reading_link() {
        let html_doc = html_from_test_resource("sunday_or.html");
        assert!(get_holiday_day_reading_link(&html_doc).is_none());
    }
}
