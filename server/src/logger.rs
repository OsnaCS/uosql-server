//! Logging functionality
//!
//! This module defines a logging implementation for the `log`
//! crate published by the Rust developer.
//!

use log::*;
use std::path::Path;
use std::fs;
use std::io;
use term::{self, ToStyle};
use std::io::Write;
use std::sync::Mutex;
use std::ops::DerefMut;

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
                logfile: file.map(|f| Mutex::new(f)),
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
    logfile: Option<Mutex<fs::File>>,
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

        // Prepare module path (remove crate name & add "::" if missing)
        let pos = record.target().find("::");
        let mod_path = match pos {
            None => "::",
            Some(pos) => &record.target()[pos ..],
        };

        // Ignore the leading 'src/' in the file path
        let src_file = &record.location().file();

        // If a logfile is specified (file logging is enabled)
        if let Some(ref file) = self.logfile {
            // Aquire a lock on the file to log into file. We may unwrap here
            // because it will just panic if a thread paniced before, while
            // holding the lock. It's very unlikely (maybe even impossible)
            // that the thread will panic during the `write!`. And if it
            // happens we want to propagate the panic to all threads.
            // We ignore the result of `write!`, because: What else should we
            // do? ;)
            let _ = write!(file.lock().unwrap().deref_mut(),
                "[{level: <5}][{module} @ {file}:{line}]> {msg}\n",
                level = record.level(),
                module = mod_path,
                file = src_file,
                line = record.location().line(),
                msg = record.args());
        }

        // If logging to stdout is enabled
        if self.stdout {
            let (lvl_col, msg_col) = get_colors(record.level());

            println!("[{level: <5}][{module} @ {file}:{line}]{delim} {msg}",
                level = lvl_col.paint(record.level()),
                module = mod_path,
                file = term::Color::Blue.paint(src_file),
                line = record.location().line(),
                delim = term::Color::White.paint("$"),
                msg = msg_col.paint(record.args()));
        }
    }
}

fn get_colors(lvl: LogLevel) -> (term::Style, term::Style) {
    use term::{Attr, ToStyle};
    use term::Color::*;
    use log::LogLevel::*;

    // Style for the user's message
    let msg_col = match lvl {
        Error   => Attr::Bold   .fg(Red),
        Warn    => Attr::Plain  .fg(Yellow),
        Info    => Attr::Plain  .fg(White),
        Debug   => Attr::Plain  .fg(NotSet),
        Trace   => Attr::Dim    .fg(NotSet),
    };

    // Color for the first info field: The log level
    let lvl_col = match lvl {
        Error   => Attr::Bold   .fg(Red),
        Warn    => Attr::Plain  .fg(Yellow),
        Info    => Attr::Plain  .fg(White),
        Debug   => Attr::Plain  .fg(NotSet),
        Trace   => Attr::Dim    .fg(NotSet),
    };

    (lvl_col, msg_col)
}
