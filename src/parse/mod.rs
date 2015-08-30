//! This module contains functions and types for parsing SQL queries.
//!

// TODO: Remove this 
#![allow(dead_code, unused_imports, unused_variables)]

pub mod ast;
pub mod token;
pub mod lex;

/// Represents a substring in the query string in byte indices.
pub struct Span {
    lo: u32,
    hi: u32,
}

/// Main function of this module: Takes a sql query as string and returns
/// the parsed AST.
pub fn parse(query: &str) -> ast::Query {
    ast::Query::Select
}
