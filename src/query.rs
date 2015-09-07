//! Query excecution module
//!
//! This module takes the output of the SQL parser and executes that query
//! by calling the appropriate `storage` and `auth` methods.
//!

use parse::ast;

fn execute_from_ast(query: ast::Query) {
    info!("Not implemented! :-(");
}
