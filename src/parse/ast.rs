//! Defines types of the abstract syntax tree
//!

/// Top level type. Is returned by `parse`.
#[derive(Debug, Clone)]
pub enum Query {
    Select,
    Insert,
    Update,
    Delete
}
