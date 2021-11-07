//! Main logic

use crate::{models, prelude::*};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use serde_json::json;

/// Authentication data, either a login_id and password
/// or a personal access token. Required for being able
/// to make calls to a Mattermost instance API.
///
/// For more information, see the
/// [Mattermost docs](https://api.mattermost.com/#tag/authentication).
#[derive(Debug)]
pub struct AuthenticationData {
    pub(crate) login_id: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) token: Option<String>,
}

impl AuthenticationData {
    /// Create a struct instance from a user's login_id and password.
    pub fn from_password(login_id: &str, password: &str) -> Self {
        Self {
            login_id: Some(login_id.to_owned()),
            password: Some(password.to_owned()),
            token: None,
        }
    }

    /// Create a struct instance from a user's personal access token.
    ///
    /// Personal access tokens must be enabled per instance by an admin.
    pub fn from_access_token(token: &str) -> Self {
        Self {
            login_id: None,
            password: None,
            token: Some(token.to_owned()),
        }
    }

    /// If the auth data is using a login_id and password.
    pub fn using_password(&self) -> bool {
        self.password.is_some()
    }

    /// If the auth data is using a personal access token.
    pub fn using_token(&self) -> bool {
        self.token.is_none()
    }
}

/// Struct to interact with a Mattermost instance API.
#[derive(Debug)]
pub struct Mattermost {
    pub(crate) instance_url: String,
    pub(crate) authentication_data: AuthenticationData,
    pub(crate) client: Client,
    pub(crate) auth_token: Option<String>,
}

impl Mattermost {
    /// Create a new instance of the struct to interact with the instance API.
    ///
    /// The `instance_url` variable should be the root URL of your Mattermost
    /// instance.
    pub fn new(instance_url: &str, authentication_data: AuthenticationData) -> Self {
        let auth_token = authentication_data.token.clone();
        Self {
            instance_url: instance_url.to_owned(),
            authentication_data,
            client: Client::new(),
            auth_token,
        }
    }

    /// Get a session token from the stored login_id and password.
    /// Required when using login_id and password authentication,
    /// before making any calls to the instance API.
    ///
    /// Does nothing if the `AuthenticationData` this struct instance
    /// was created with used a personal access token.
    pub async fn store_session_token(&mut self) -> Result<(), ApiError> {
        if self.authentication_data.using_token() {
            return Ok(());
        }
        let url = format!("{}/api/v4/users/login", self.instance_url);
        let resp = self
            .client
            .post(&url)
            .json(&json!({
                "login_id": self.authentication_data.login_id.as_ref().unwrap(),
                "password": self.authentication_data.password.as_ref().unwrap(),
            }))
            .send()
            .await?;
        let session_token = resp
            .headers()
            .get("Token")
            .ok_or_else(|| ApiError::CouldNotGetToken(resp.status().as_u16()))?;
        self.auth_token = Some(session_token.to_str()?.to_string());
        Ok(())
    }

    /// Headers for interacting with the API.
    fn request_headers(&self) -> Result<HeaderMap, ApiError> {
        let mut map = HeaderMap::new();
        map.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        map.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        map.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!(
                "Bearer {}",
                self.auth_token.as_ref().ok_or(ApiError::MissingAuthToken)?
            ))?,
        );
        Ok(map)
    }

    // ===========================================================================================
    //      API endpoints
    // ===========================================================================================

    /// Get information for a team by its name,
    pub async fn get_team_info(&self, name: &str) -> Result<models::TeamInformation, ApiError> {
        let resp = self
            .client
            .get(&format!("{}/api/v4/teams/name/{}", self.instance_url, name))
            .headers(self.request_headers()?)
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
