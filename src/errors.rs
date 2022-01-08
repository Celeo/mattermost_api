//! Errors

use crate::models::MattermostError;
use thiserror::Error;

/// Errors that may arise over the course of using the library.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Could not turn login_id and password into a session token, response code {0}")]
    CouldNotGetToken(u16),
    #[error("No token was supplied or retrieved")]
    MissingAuthToken,
    #[error("HTTP client error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("HTTP header printing error")]
    ReqwestHeaderError(#[from] reqwest::header::ToStrError),
    #[error("JSON processing error")]
    JsonProcessingError(#[from] serde_json::Error),
    #[error("HTTP header value parsing error")]
    ReqwestHeaderValueError(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Invalid HTTP method")]
    HttpMethodError(#[from] http::method::InvalidMethod),
    #[error("Mattermost API returned error: {0:?}")]
    MattermostApiError(MattermostError),
    #[error("Non-standard remote status code error")]
    StatusCodeError(u16),
    #[error("Websocket connection error")]
    WebsocketError(#[from] async_tungstenite::tungstenite::Error),
}
