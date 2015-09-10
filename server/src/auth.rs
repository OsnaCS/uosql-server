//! Module for authenticating users and check privileges
//!
//! Contains the `User` type and functions to:
//!
//! - check if a username-password pair is valid
//! - load the corresponding user
//! - check user permissions for every query
//!

use super::storage;
/// Contains information about the user that opened the connection. Is used
/// for every type of access control.
pub struct User {
    pub _name: String,
    pub _currentDatabase: Option<storage::Database>,
}

/// Errors that may occur during user authentication
pub enum AuthError {
    UserNotFound,
    WrongPassword,
}

/// Validates username and password and returns the matched user.
///
/// **Note:** Currently nothing is checked yet and a meaningless `User` object
/// is returned!
///
/// # Failures
/// If the user was not found or the password does not match, an `Err` value
/// is returned. See `AuthError` for more information.
pub fn find_user(_name: &str, _passwd: &str) -> Result<User, AuthError> {
    debug!("User '{}' was succesfully (pseudo-!) authenticated", _name);
    Ok(User {
        _name: _name.into(),
        _currentDatabase: None,
    })
}
