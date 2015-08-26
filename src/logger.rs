//! Logging functionality
//!
//! This module defines a logging implementation for the `log`
//! crate published by the Rust developer.
//!

use log::*;
use std::path::Path;
use std::fs;
use std::io;

/// Returns a builder that can enable the logger globally.
pub fn with_loglevel(lvl: LogLevelFilter) -> Builder<'static> {
    Builder {
        lvl: lvl,
        logfile: None,
        stdout: true,
    }
}

/// A builder type to easily configure the logger.
pub struct Builder<'a> {
    lvl: LogLevelFilter,
    logfile: Option<&'a Path>,
    stdout: bool,
}

#[allow(dead_code)]
impl<'a> Builder<'a> {
    /// Enables logging into the given file
    pub fn with_logfile<'b>(self, path: &'b Path) -> Builder<'b> {
        Builder {
            lvl: self.lvl,
            logfile: Some(path),
            stdout: self.stdout,
        }
    }

    /// Disables logging to stdout (which is enabled by default)
    pub fn without_stdout(self) -> Builder<'a> {
        Builder {
            lvl: self.lvl,
            logfile: self.logfile,
            stdout: false,
        }
    }

    /// Creates the `Logger` from the given configuration and enables it
    /// globally. Any log messages generated before this method is called,
    /// will be ignored.
    ///
    /// # Failures
    /// - Returns an `Err` if the a logfile was specified, but it could not be
    /// opened in write-append-create mode.
    /// - Returns an `Err` with kind `AlreadyExists` if this method is called
    /// more than once in one running program.
    pub fn enable(self) -> io::Result<()> {
        // Try to open the logfile in write-append mode, if any was specified
        let file = match self.logfile {
            Some(path) => {
                Some(try!(fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create(true)
                    .open(path)))
            },
            None => None,
        };

        set_logger(|filter| {
            filter.set(self.lvl);
            Box::new(Logger {
                level_filter: filter,
                logfile: file,
                stdout: self.stdout,
            })
        }).map_err(|_| io::Error::new(
            io::ErrorKind::AlreadyExists,
            "method 'enable' was called more than once!"
            )
        )
    }
}

/// Type to do the actual logging. You don't need to interact with it directly:
/// Use macros and functions of the `log` crate.
struct Logger {
    level_filter: MaxLogLevelFilter,
    logfile: Option<fs::File>,
    stdout: bool,
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level_filter.get()
    }

    fn log(&self, record: &LogRecord) {
        // Early return if the message won't be printed
        if !self.enabled(record.metadata()) {
            return;
        }

        println!("[{level}][{thread}][{module} @ {file}:{line}] {msg}",
            level = record.level(),
            thread = "T?",
            module = record.target(),
            file = record.location().file(),
            line = record.location().line(),
            msg = record.args());
    }
}
