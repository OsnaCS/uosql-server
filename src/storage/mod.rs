//! Storage Engine trait and several implementations
//!
//!

/// Storage Engine Interface.
///
/// A storage engine, like MyISAM and InnoDB, is responsible for reading and
/// writing data to disk, maintain and use indices, check for data corruption
/// and repair corrupt files.
///
/// Each table in a database may use a different storage engine.
pub trait Engine {

}

/// A database table
///
/// Through this type, you can retreive certain meta information about the
/// table (like column names, column types, storage engine, ...). It's `access`
/// method locks the table globally and returns a storage engine to access
/// the table data.
pub struct Table;

impl Table {
    pub fn load(database: &str, table: &str) -> Table {
        // TODO: Read the .tbl file from disk and parse it
        Table
    }

    pub fn columns(&self) -> &[Column] {
        // TODO: Return real columns
        &[]
    }
}

/// A table column. Has a name, a type, ...
pub struct Column;

// # Some information for the `storage` working group:
//
// You work at the very bottom of the database: The thing that actually
// writes the data disk. Everything in this module is used only by the
// query execution module.
//
// The file layout may look like this:
// We have some `db_dir` where everything lives. In that directory, there are
// subdirectories for every database. In each of those subdirs is optionally
// a file `db.meta` that contains information about the database (such as
// permissions). The tables of each database are saved in *.tbl files that
// live inside the database directory.
//
// Your task is to provide types and method to:
// - read a specific table from a specific
//   database from file
// - create a new table in a database
// - query meta information about a table (columns for example)
// - lock a table for reading/writing it's data through a storage engine
//
// The other main task is to:
// - specify the storage engine interface
// - implement a simple storage engine (heap/flatfiles)
//
