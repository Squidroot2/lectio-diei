use std::borrow::Cow;

use crate::{
    args::{CommonArguments, DisplayReadingsArgs, ReadingArg},
    config::Config,
    lectionary::{Lectionary, Reading, ReadingName},
};

pub struct DisplaySettings {
    pub readings_to_display: ReadingsOptions,
    pub no_color: bool,
}

impl DisplaySettings {
    pub fn from_config_and_args(config: Config, reading_args: DisplayReadingsArgs, args: CommonArguments) -> Self {
        Self {
            no_color: args.no_color,
            readings_to_display: ReadingsOptions::from_config_and_args(config, reading_args),
        }
    }
}

/// Says what to print
pub enum ReadingsOptions {
    All,
    DayOnly,
    Specified(Vec<ReadingArg>),
}

impl ReadingsOptions {
    fn from_config_and_args(config: Config, args: DisplayReadingsArgs) -> Self {
        if args.day_only {
            return Self::DayOnly;
        } else if args.all {
            return Self::All;
        }
        // Prefer to use commandline arguments over config
        Self::Specified(args.readings.unwrap_or(config.display.reading_order))
    }
}

impl Lectionary {
    pub fn pretty_print(&self, settings: DisplaySettings) {
        match settings.readings_to_display {
            ReadingsOptions::All => self.print_all(),
            ReadingsOptions::DayOnly => self.print_day_only(),
            ReadingsOptions::Specified(list) => self.print_list(list),
        }
    }

    /// Prints all readings in their default order
    ///
    ///
    fn print_all(&self) {
        let dashes = self.get_dash_seperator();

        self.print_day_name(&dashes);
        self.print_reading_one(&dashes);
        self.print_resp_psalm(&dashes);
        self.print_reading_two(&dashes);
    }

    fn print_day_only(&self) {
        let dashes = self.get_dash_seperator();
        self.print_day_name(&dashes)
    }

    /// Prints readings in a specified order
    fn print_list(&self, list: Vec<ReadingArg>) {
        let dashes = self.get_dash_seperator();
        self.print_day_name(&dashes);
        for reading in list {
            match reading {
                ReadingArg::Reading1 => self.print_reading_one(&dashes),
                ReadingArg::Reading2 => self.print_reading_two(&dashes),
                ReadingArg::Psalm => self.print_resp_psalm(&dashes),
                ReadingArg::Gospel => self.print_gospel(&dashes),
            }
        }
    }

    fn get_dash_seperator(&self) -> String {
        let dash_length = self.get_day_name().len() + 4;
        let mut dashes = String::with_capacity(dash_length);
        for _ in 0..dash_length {
            dashes.push('-')
        }
        dashes
    }

    fn print_day_name(&self, dashes: &str) {
        println!("{}", dashes);
        println!("  {}  ", self.get_day_name());
        println!("{}", dashes);
    }

    fn print_reading_one(&self, seperator: &str) {
        self.get_reading_1().pretty_print(ReadingName::Reading1.as_str(), &seperator, false);
    }

    fn print_resp_psalm(&self, seperator: &str) {
        self.get_resp_psalm().pretty_print(ReadingName::Psalm.as_str(), &seperator, true);
    }

    fn print_reading_two(&self, seperator: &str) {
        self.get_reading_2()
            .inspect(|reading_2| reading_2.pretty_print(ReadingName::Reading2.as_str(), &seperator, false));
    }

    fn print_gospel(&self, seperator: &str) {
        self.get_gospel().pretty_print(ReadingName::Gospel.as_str(), &seperator, false);
    }
}

impl Reading {
    pub fn pretty_print(&self, heading: &str, seperator: &str, preserve_newlines: bool) {
        let text: Cow<'_, str> = match preserve_newlines {
            true => Cow::Borrowed(self.get_text()),
            false => Cow::Owned(self.get_text().replace('\n', " ")),
        };
        println!("{} ({})", heading, self.get_location());
        println!("{}", seperator);
        println!("{}", text);
        println!("{}", seperator);
    }
}
