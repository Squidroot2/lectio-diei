use log::*;
use regex::Regex;

use crate::{
    args::{CommonArguments, DisplayReadingsArgs, FormattingArgs, ReadingArg},
    config::Config,
    lectionary::{Lectionary, Reading, ReadingName},
};

/// Used for reading1, reading2, gospel. Not psalm
#[derive(Clone, Copy)]
enum LineBreaks {
    /// Removes all lines breaks
    None,
    /// Keeps original line breaks
    Original,
    /// Sets a maximum width
    Width(u16),
}

impl LineBreaks {
    fn from_config_and_args(config_original_linebreaks: bool, config_max_width: u16, args: FormattingArgs) -> Self {
        // First look at args, since args overwrite config
        if args.original_linebreaks {
            return Self::Original;
        }
        if let Some(arg_max_width) = args.max_width {
            if arg_max_width == 0 {
                return Self::None;
            }
            return Self::Width(arg_max_width);
        }
        // Args not set, use config
        if config_original_linebreaks {
            return Self::Original;
        }
        if config_max_width == 0 {
            return Self::None;
        }
        Self::Width(config_max_width)
    }
}

/// Says what readings to print
pub enum ReadingsOptions {
    All,
    DayOnly,
    Specified(Vec<ReadingArg>),
}

impl ReadingsOptions {
    fn from_config_and_args(config_reading_order: Vec<ReadingArg>, args: DisplayReadingsArgs) -> Self {
        // First look at args, since args overwrite configs
        if args.day_only {
            return Self::DayOnly;
        }
        if args.all {
            return Self::All;
        }
        // Prefer to use commandline arguments over config
        Self::Specified(args.readings.unwrap_or(config_reading_order))
    }
}

pub struct DisplaySettings {
    pub readings_to_display: ReadingsOptions,
    //TODO handle color and no_color
    pub _no_color: bool,
    line_breaks: LineBreaks,
}

impl DisplaySettings {
    pub fn from_config_and_args(
        config: Config,
        reading_args: DisplayReadingsArgs,
        formatting_args: FormattingArgs,
        args: CommonArguments,
    ) -> Self {
        Self {
            _no_color: args.no_color,
            readings_to_display: ReadingsOptions::from_config_and_args(config.display.reading_order, reading_args),
            line_breaks: LineBreaks::from_config_and_args(config.display.original_linebreaks, config.display.max_width, formatting_args),
        }
    }
}

const ALL_READINGS: [ReadingArg; 4] = [ReadingArg::Reading1, ReadingArg::Reading2, ReadingArg::Psalm, ReadingArg::Gospel];

impl Lectionary {
    /// Displays the lectionary with the given `DisplaySettings`
    pub fn pretty_print(&self, settings: &DisplaySettings) {
        let list = match &settings.readings_to_display {
            ReadingsOptions::All => ALL_READINGS.as_slice(),
            ReadingsOptions::DayOnly => &[],
            ReadingsOptions::Specified(list) => list.as_slice(),
        };
        let dashes = self.get_dash_seperator();
        self.print_day_name(&dashes);
        for reading in list {
            match reading {
                ReadingArg::Reading1 => {
                    self.get_reading_1()
                        .pretty_print_as_reading(ReadingName::Reading1.as_str(), &dashes, settings.line_breaks);
                }
                ReadingArg::Reading2 => {
                    let _ = self.get_reading_2().inspect(|reading_2| {
                        reading_2.pretty_print_as_reading(ReadingName::Reading2.as_str(), &dashes, settings.line_breaks);
                    });
                }
                ReadingArg::Psalm => self.get_resp_psalm().pretty_print_as_psalm(ReadingName::Psalm.as_str(), &dashes),
                ReadingArg::Gospel => {
                    self.get_gospel()
                        .pretty_print_as_reading(ReadingName::Gospel.as_str(), &dashes, settings.line_breaks);
                }
                ReadingArg::Alleluia => self
                    .get_alleluia()
                    .pretty_print_as_alleliua(ReadingName::Alleluia.as_str(), &dashes),
            }
        }
    }

    fn get_dash_seperator(&self) -> String {
        let dash_length = self.get_day_name().len() + 4;
        let mut dashes = String::with_capacity(dash_length);
        for _ in 0..dash_length {
            dashes.push('-');
        }
        dashes
    }

    fn print_day_name(&self, dashes: &str) {
        println!("{dashes}");
        println!("  {}  ", self.get_day_name());
        println!("{dashes}");
    }
}

impl Reading {
    /// prints the reading
    ///
    /// seperator is the line seperating the heading from the text
    fn pretty_print_as_reading(&self, heading: &str, seperator: &str, line_breaks: LineBreaks) {
        self.print_heading(heading);
        println!("{seperator}");
        match line_breaks {
            LineBreaks::Original => println!("{}", self.get_text()),
            LineBreaks::None => println!("{}", self.get_text().replace('\n', " ")),
            LineBreaks::Width(width) => Self::print_word_wrapped_text(self.get_text(), width),
        };
        println!("{seperator}");
    }

    /// Should only be used for Psalms
    fn pretty_print_as_psalm(&self, heading: &str, seperator: &str) {
        self.print_heading(heading);
        println!("{seperator}");
        let mut lines = self.get_text().lines();
        if let Some(first_line) = lines.next() {
            println!("{}", Self::format_psalm_first_line(first_line));
            for line in lines {
                println!("{line}");
            }
        } else {
            error!("Can't format the psalm: it has no content");
        }
        println!("{seperator}");
    }

    /// Similar to psalm but without modifications to the first line
    fn pretty_print_as_alleliua(&self, heading: &str, seperator: &str) {
        self.print_heading(heading);
        println!("{seperator}");
        println!("{}", self.get_text());
        println!("{seperator}");
    }

    fn print_heading(&self, heading: &str) {
        if self.get_location().is_empty() {
            println!("{heading}");
        } else {
            println!("{heading} ({})", self.get_location());
        }
    }

    /// Removes the verse number from the first line of the psalm
    fn format_psalm_first_line(first_line: &str) -> String {
        let pattern = Regex::new(r"\(.+\)\s+").expect("Should be valid regex");
        let mut out = String::new();
        for part in pattern.splitn(first_line, 2) {
            out += part;
        }
        out
    }

    fn print_word_wrapped_text(text: &str, max_width: u16) {
        let words = text.split_ascii_whitespace();
        let mut current_line = String::new();
        for word in words {
            if (current_line.len() + word.len()) > max_width.into() {
                println!("{current_line}");
                current_line.clear();
            }
            current_line.push_str(word);
            current_line.push(' ');
        }
        println!("{current_line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn psalm_heading_formatted() {
        assert_eq!("R. Test Line", Reading::format_psalm_first_line("R. (8)   Test Line"));
    }
}
