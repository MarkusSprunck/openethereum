// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Logger for OpenEthereum executables

extern crate ansi_term;
extern crate arrayvec;
extern crate atty;
extern crate chrono;
extern crate env_logger;
extern crate log as rlog;
extern crate parking_lot;
extern crate regex;
extern crate time;

#[macro_use]
extern crate lazy_static;

mod rotating;

use ansi_term::Colour;
use chrono::SecondsFormat;
use env_logger::{Builder as LogBuilder, Formatter};
use parking_lot::Mutex;
use regex::Regex;
use std::{
    env, fs,
    io::Write,
    sync::{Arc, Weak},
    thread,
};

pub use rotating::{init_log, RotatingLogger};

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub mode: Option<String>,
    pub color: bool,
    pub file: Option<String>,
    pub json: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mode: None,
            color: true,
            file: None,
            json: true,
        }
    }
}

lazy_static! {
    static ref ROTATING_LOGGER: Mutex<Weak<RotatingLogger>> = Mutex::new(Default::default());
}

/// Escapes multiline message string for json output, e.g. call stacks
fn escape(text: &String) -> String {
    text.replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('"', "\\\"")
}

/// Sets up the logger
pub fn setup_log(config: &Config) -> Result<Arc<RotatingLogger>, String> {
    use rlog::*;

    let mut levels = String::new();
    let mut builder = LogBuilder::new();
    // Disable info logging by default for some modules:
    builder.filter(Some("ws"), LevelFilter::Warn);
    builder.filter(Some("hyper"), LevelFilter::Warn);
    builder.filter(Some("rustls"), LevelFilter::Error);
    // Enable info for others.
    builder.filter(None, LevelFilter::Info);

    if let Ok(lvl) = env::var("RUST_LOG") {
        levels.push_str(&lvl);
        levels.push(',');
        builder.parse(&lvl);
    }

    if let Some(ref s) = config.mode {
        levels.push_str(s);
        builder.parse(s);
    }

    let isatty = atty::is(atty::Stream::Stderr);
    let enable_color = config.color && isatty;
    let enable_json = config.json;
    let logs = Arc::new(RotatingLogger::new(levels));
    let logger = logs.clone();
    let mut open_options = fs::OpenOptions::new();

    let maybe_file = match config.file.as_ref() {
        Some(f) => Some(
            open_options
                .append(true)
                .create(true)
                .open(f)
                .map_err(|e| format!("Cannot write to log file given: {f}, {e}"))?,
        ),
        None => None,
    };

    let format = move |buf: &mut Formatter, record: &Record| {
        let with_color = if max_level() <= LevelFilter::Info && !enable_json {
            let utc_time = chrono::Utc::now();
            let timestamp = utc_time.format("%Y-%m-%d %H:%M:%S %Z").to_string();
            format!(
                "{} {}",
                Colour::Black.bold().paint(timestamp),
                record.args()
            )
        } else {
            let name = thread::current()
                .name()
                .map_or_else(Default::default, |x| x.to_string());
            if enable_json {
                let utc_time = chrono::Utc::now();
                let timestamp = utc_time.to_rfc3339_opts(SecondsFormat::Millis, true);
                format!(
                    "{{\"@timestamp\":\"{}\",\"@version\":\"1\",\"SERVICE\":\"{}\",\"level\":\"{}\",\"STEP\":\"{}\",\"message\":\"{}\"}}",
                    timestamp,
                    name,
                    record.level(),
                    record.target(),
                    escape(&record.args().to_string())
                )
            } else {
                let utc_time = chrono::Utc::now();
                let timestamp = utc_time.format("%Y-%m-%d %H:%M:%S %Z").to_string();
                let name = thread::current().name().map_or_else(Default::default, |x| {
                    format!("{}", Colour::Blue.bold().paint(x))
                });
                format!(
                    "{} {} {} {}  {}",
                    Colour::Black.bold().paint(timestamp),
                    name,
                    record.level(),
                    record.target(),
                    record.args()
                )
            }
        };

        let removed_color = kill_color(with_color.as_ref());

        let ret = match enable_color {
            true => with_color,
            false => removed_color.clone(),
        };

        if let Some(mut file) = maybe_file.as_ref() {
            // ignore errors - there's nothing we can do
            let _ = file.write_all(removed_color.as_bytes());
            let _ = file.write_all(b"\n");
        }
        logger.append(removed_color);
        if !isatty && record.level() <= Level::Info && atty::is(atty::Stream::Stdout) {
            // duplicate INFO/WARN output to console
            println!("{ret}");
        }

        writeln!(buf, "{ret}")
    };

    builder.format(format);
    builder
        .try_init()
        .map(|_| {
            *ROTATING_LOGGER.lock() = Arc::downgrade(&logs);
            logs
        })
        // couldn't create new logger - try to fall back on previous logger.
        .or_else(|err| {
            ROTATING_LOGGER
                .lock()
                .upgrade()
                .ok_or_else(|| format!("{err:?}"))
        })
}

fn kill_color(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("\x1b\\[[^m]+m").unwrap();
    }
    RE.replace_all(s, "").to_string()
}

#[test]
fn should_remove_colour() {
    let before = "test";
    let after = kill_color(&Colour::Red.bold().paint(before));
    assert_eq!(after, "test");
}

#[test]
fn should_remove_multiple_colour() {
    let t = format!(
        "{} {}",
        Colour::Red.bold().paint("test"),
        Colour::White.normal().paint("again")
    );
    let after = kill_color(&t);
    assert_eq!(after, "test again");
}
