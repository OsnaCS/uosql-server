//! This module contains functions and types for parsing SQL queries.
//!

pub mod ast;

/// Main function of this module: Takes a sql query as string and returns
/// the parsed AST.
pub fn parse(query: &str) -> ast::Query {
    ast::Query::Select
}
