//! Errors

use thiserror::Error;

/// Errors that may arise over the course of using the library.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Could not turn login_id and password into a session token
    #[error("Could not turn login_id and password into a session token, response code {0}")]
    CouldNotGetToken(u16),
    /// No token was supplied or retrieved
    #[error("No token was supplied or retrieved")]
    MissingAuthToken,
    /// HTTP client error
    #[error("HTTP client error")]
    ReqwestError(#[from] reqwest::Error),
    /// HTTP header printing error
    #[error("HTTP header printing error")]
    ReqwestHeaderError(#[from] reqwest::header::ToStrError),
    /// HTTP header value parsing error
    #[error("HTTP header value parsing error")]
    ReqwestHeaderValueError(#[from] reqwest::header::InvalidHeaderValue),
    /// Invalid HTTP method
    #[error("Invalid HTTP method")]
    HttpMethodError(#[from] http::method::InvalidMethod),
    /// Error that can be thrown by any function that makes HTTP
    /// calls to external resources for response codes that
    /// aren't valid as defined [by reqwest].
    ///
    /// [by reqwest]: <https://docs.rs/reqwest/0.11.6/reqwest/struct.StatusCode.html#method.is_success>
    #[error("Invalid HTTP status code received: {0}")]
    InvalidStatusCode(u16),
}
