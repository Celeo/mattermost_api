//! Client struct and functions for interacting with the REST API.

use crate::{models, prelude::*};
use async_tungstenite::{tokio::ConnectStream, tungstenite::Message, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Method,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use url::Url;

/// Authentication data, either a login_id and password
/// or a personal access token. Required for being able
/// to make calls to a Mattermost instance API.
///
/// Use `from_password` and `from_access_token` to create
/// an instance of this struct.
///
/// For more information, see the
/// [Mattermost docs](https://api.mattermost.com/#tag/authentication).
#[derive(Debug, Clone)]
pub struct AuthenticationData {
    pub(crate) login_id: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) token: Option<String>,
}

impl AuthenticationData {
    /// Create a struct instance from a user's login_id and password.
    pub fn from_password(login_id: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            login_id: Some(login_id.into()),
            password: Some(password.into()),
            token: None,
        }
    }

    /// Create a struct instance from a user's personal access token.
    ///
    /// Personal access tokens must be enabled per instance by an admin.
    pub fn from_access_token(token: impl Into<String>) -> Self {
        Self {
            login_id: None,
            password: None,
            token: Some(token.into()),
        }
    }

    /// If the auth data is using a login_id and password.
    pub fn using_password(&self) -> bool {
        self.password.is_some()
    }

    /// If the auth data is using a personal access token.
    pub fn using_token(&self) -> bool {
        self.token.is_some()
    }
}

/// Struct to interact with a Mattermost instance API.
///
/// Use the `new` function to create an instance of this struct.
#[derive(Debug, Clone)]
pub struct Mattermost {
    pub(crate) instance_url: Url,
    pub(crate) authentication_data: AuthenticationData,
    pub(crate) client: Client,
    pub(crate) auth_token: Option<String>,
    #[cfg(feature = "ws-keep-alive")]
    pub(crate) ping_interval: std::time::Duration,
}

impl Mattermost {
    /// Create a new instance of the struct to interact with the instance API.
    ///
    /// The `instance_url` variable should be the root URL of your Mattermost
    /// instance.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mattermost_api::prelude::*;
    /// # async fn run() {
    /// let auth = AuthenticationData::from_password("you@example.com", "password");
    /// let api = Mattermost::new("https://your-mattermost-instance.com", auth);
    /// # }
    /// ```
    pub fn new(
        instance_url: impl AsRef<str>,
        authentication_data: AuthenticationData,
    ) -> Result<Self, ApiError> {
        let mut instance_url = Url::parse(instance_url.as_ref())?;
        let auth_token = authentication_data.token.clone();

        if instance_url.path() == "/" {
            instance_url.set_path("/api/v4/");
        }

        Ok(Self {
            instance_url,
            authentication_data,
            client: Client::new(),
            auth_token,
            #[cfg(feature = "ws-keep-alive")]
            ping_interval: std::time::Duration::from_secs(30),
        })
    }

    #[cfg(feature = "ws-keep-alive")]
    /// Changes the interval between sending ping messages to keep the websocket connection alive.
    ///
    /// The default is 30 seconds.
    pub fn with_ping_interval(mut self, interval: std::time::Duration) -> Self {
        self.ping_interval = interval;
        self
    }

    /// Get a session token from the stored login_id and password.
    /// Required when using login_id and password authentication,
    /// before making any calls to the instance API.
    ///
    /// Does nothing if the `AuthenticationData` this struct instance
    /// was created with used a personal access token.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mattermost_api::prelude::*;
    /// # async fn run() {
    /// let auth = AuthenticationData::from_password("you@example.com", "password");
    /// let mut api = Mattermost::new("https://your-mattermost-instance.com", auth).unwrap();
    /// api.store_session_token().await.unwrap();
    /// # }
    /// ```
    pub async fn store_session_token(&mut self) -> Result<(), ApiError> {
        if self.authentication_data.using_token() {
            debug!("Using personal access token; getting a session token is a no-op");
            return Ok(());
        }
        debug!("Getting a session token from login_id and password");
        let url = self.instance_url.join("users/login")?;
        let resp = self
            .client
            .post(url)
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
        debug!("Session token retrieved and stored");
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

    /// Helper function for query that joins the instance url with an endpoint in an expected manner
    fn endpoint_url(&self, endpoint: &str) -> Result<Url, ApiError> {
        Ok(self.instance_url.join(endpoint.trim_start_matches('/'))?)
    }

    /// Make a query to the Mattermost instance API.
    ///
    /// This method is "raw" in that the calling code must
    /// supply the method, query parameters, body, **and**
    /// a struct for the shape of the data returned (or
    /// `serde_json::Value` for "dynamic" data).
    ///
    /// Developers are encouraged to look for a specific endpoint
    /// function that is for the API endpoint that is desired,
    /// but this function is exposed to calling code so that
    /// this library can be more flexible.
    pub async fn query<T: DeserializeOwned>(
        &self,
        method: &str,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<&str>,
    ) -> Result<T, ApiError> {
        let url = self.endpoint_url(endpoint)?;
        let method = Method::try_from(method)?;

        debug!("Making {method} request to {url} with query {query:?}",);

        let mut req_builder = self
            .client
            .request(method, url.clone())
            .headers(self.request_headers()?)
            .query(query.unwrap_or(&[]));
        req_builder = match body {
            Some(b) => req_builder.body(b.to_owned()),
            None => req_builder,
        };
        let resp = self.client.execute(req_builder.build()?).await?;
        if !resp.status().is_success() {
            error!(
                "Got status {} when requesting data from {}",
                resp.status(),
                url
            );
            let status = resp.status().as_u16();
            // attempt to get the standard error information out and return that
            if let Ok(text) = resp.text().await {
                debug!("{text}");
                if let Ok(data) = serde_json::from_str::<MattermostError>(&text) {
                    return Err(ApiError::MattermostApiError(data));
                }
            }
            // fallback to generic HTTP status code error
            return Err(ApiError::StatusCodeError(status));
        }
        Ok(resp.json().await?)
    }

    /// Send a post request with a JSON body and optional query parameters.
    pub async fn post<J: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        body: &J,
    ) -> Result<T, ApiError> {
        let url = self.endpoint_url(endpoint)?;

        debug!("Making post request to {url} with query {query:?}");

        let req_builder = self
            .client
            .post(url.clone())
            .headers(self.request_headers()?)
            .query(query.unwrap_or(&[]))
            .json(body);
        let resp = self.client.execute(req_builder.build()?).await?;
        if !resp.status().is_success() {
            error!(
                "Got status {} when requesting data from {}",
                resp.status(),
                url
            );
            let status = resp.status().as_u16();
            // attempt to get the standard error information out and return that
            if let Ok(text) = resp.text().await {
                debug!("{text}");
                if let Ok(data) = serde_json::from_str::<MattermostError>(&text) {
                    return Err(ApiError::MattermostApiError(data));
                }
            }
            // fallback to generic HTTP status code error
            return Err(ApiError::StatusCodeError(status));
        }
        Ok(resp.json().await?)
    }

    /// Helper-function for connect_to_websocket that convets http schemes to ws equivalent
    fn ws_instance_url(&self) -> Result<Url, ApiError> {
        let mut url = self.instance_url.clone();

        match url.scheme() {
            "http" => url.set_scheme("ws").ok(),
            "https" => url.set_scheme("wss").ok(),
            _ => None,
        };

        Ok(url)
    }

    /// Connect to the websocket API on the instance.
    ///
    /// This method loops, sending messages received from
    /// the websocket connection to the passed handler. The
    /// authentication handshake is handled with the
    /// connection is made, but otherwise no handling of
    /// messages is currently implemented.
    ///
    /// This function is likely to experience a great
    /// deal of change soon.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_trait::async_trait;
    /// use mattermost_api::prelude::*;
    ///
    /// struct Handler {}
    ///
    /// #[async_trait]
    /// impl WebsocketHandler for Handler {
    ///     async fn callback(&self, message: WebsocketEvent) {
    ///         println!("{:?}", message);
    ///     }
    /// }
    ///
    /// # async fn run() {
    /// let auth = AuthenticationData::from_password("you@example.com", "password");
    /// let mut api = Mattermost::new("https://your-mattermost-instance.com", auth).unwrap();
    /// api.store_session_token().await.unwrap();
    /// api.connect_to_websocket(Handler {}).await.unwrap();
    /// # }
    /// ```
    pub async fn connect_to_websocket<H: WebsocketHandler + 'static>(
        &mut self,
        handler: H,
    ) -> Result<(), ApiError> {
        let url = self.ws_instance_url()?.join("websocket")?;
        let (mut stream, _response) = async_tungstenite::tokio::connect_async(url)
            .await
            .map_err(Box::new)?;
        stream
            .send(Message::Text(serde_json::to_string(&json!({
              "seq": 1,
              "action": "authentication_challenge",
              "data": {
                "token": self.auth_token.as_ref().unwrap()
              }
            }))?))
            .await
            .map_err(Box::new)?;

        self.receive_events(stream, handler).await
    }

    #[cfg(not(feature = "ws-keep-alive"))]
    async fn receive_events<H: WebsocketHandler + 'static>(
        &self,
        mut stream: WebSocketStream<ConnectStream>,
        handler: H,
    ) -> Result<(), ApiError> {
        loop {
            if let Some(event) = stream.next().await {
                let event = event.map_err(|err| {
                    error!("Error getting websocket message: {err}");
                    ApiError::WebsocketError(Box::new(err))
                })?;

                if self.handle_event(&handler, event).await? {
                    break;
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "ws-keep-alive")]
    async fn receive_events<H: WebsocketHandler + 'static>(
        &self,
        mut stream: WebSocketStream<ConnectStream>,
        handler: H,
    ) -> Result<(), ApiError> {
        let mut ping_interval = tokio::time::interval(self.ping_interval);

        loop {
            tokio::select! {
                Some(event) = stream.next() => {
                    let event = event.map_err(|err| {
                        error!("Error getting websocket message: {err}");
                        ApiError::WebsocketError(Box::new(err))
                    })?;

                    if self.handle_event(&handler, event).await? {
                        break;
                    }
                },
                _ = ping_interval.tick() => {
                    if let Err(err) = stream.send(Message::Ping(vec![])).await {
                        error!("Error sending Ping message through websocket: {err}");
                    }
                }
            }
        }

        Ok(())
    }

    /// Internal method to aplly the users handler to text events.
    ///
    /// Returns true if the connection is closing.
    async fn handle_event<H: WebsocketHandler + 'static>(
        &self,
        handler: &H,
        message: Message,
    ) -> Result<bool, ApiError> {
        match message {
            Message::Text(text) if text.contains("seq_reply") => {
                // for now, replies are not sent to the handler
                debug!("Reply text message received. Skipping.");
                Ok(false)
            }
            Message::Text(text) => {
                debug!("Non-reply text message received. Calling handler.");

                let as_struct = serde_json::from_str(&text).map_err(|err| {
                    error!("Could not parse websocket event JSON: {err}");
                    ApiError::JsonProcessingError(err)
                })?;

                handler.callback(as_struct).await;

                Ok(false)
            }
            Message::Close(_) => {
                debug!("Close message received.");
                Ok(true)
            }
            message => {
                debug!("Non-text, non-close message received: {message:#?}");
                Ok(false)
            }
        }
    }

    // ===========================================================================================
    //      API endpoints
    // ===========================================================================================

    /// Get a team's information.
    pub async fn get_team(&self, id: &str) -> Result<models::TeamInformation, ApiError> {
        self.query("GET", &format!("teams/{id}"), None, None).await
    }

    /// Get information for a team by its name,
    pub async fn get_team_by_name(&self, name: &str) -> Result<models::TeamInformation, ApiError> {
        self.query("GET", &format!("teams/name/{name}"), None, None)
            .await
    }

    /// List teams that are open or, if the user has the "manage_system" permission, exist.
    pub async fn get_teams(&self) -> Result<Vec<models::TeamInformation>, ApiError> {
        self.query("GET", "teams", None, None).await
    }

    /// Get the number of unread messages and mentions for all member teams of the user.
    pub async fn get_team_unreads_for(
        &self,
        user_id: &str,
    ) -> Result<Vec<models::TeamsUnreadInformation>, ApiError> {
        self.query("GET", &format!("users/{user_id}/teams/unread"), None, None)
            .await
    }

    /// Get the number of unread messages and mentions for the specific team the user is in.
    ///
    /// Requires either the "read_channel" or "edit_other_users" permission.
    pub async fn get_team_unreads_for_in(
        &self,
        user_id: &str,
        team_id: &str,
    ) -> Result<models::TeamsUnreadInformation, ApiError> {
        self.query(
            "GET",
            &format!("users/{user_id}/teams/{team_id}/unread"),
            None,
            None,
        )
        .await
    }

    /// Get all channels on the instance.
    ///
    /// Requires the "manage_system" permission.
    pub async fn get_all_channels(
        &self,
        not_associated_to_group: Option<&str>,
        page: Option<u64>,
        per_page: Option<u64>,
        exclude_default_channels: Option<bool>,
        exclude_policy_constrained: Option<bool>,
    ) -> Result<Vec<models::ChannelInformation>, ApiError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(v) = not_associated_to_group {
            query.push(("not_associated_to_group", v.into()));
        }
        if let Some(v) = page {
            query.push(("page", v.to_string()));
        }
        if let Some(v) = per_page {
            query.push(("per_page", v.to_string()));
        }
        if let Some(v) = exclude_default_channels {
            query.push(("exclude_default_channels", v.to_string()));
        }
        if let Some(v) = exclude_policy_constrained {
            query.push(("exclude_policy_constrained", v.to_string()));
        }
        let query: Vec<(&str, &str)> = query.iter().map(|(a, b)| (*a, &**b)).collect();
        self.query("GET", "channels", Some(&query), None).await
    }

    /// Get a channel's information.
    ///
    /// Requires the "read_channel" permission for that channel.
    pub async fn get_channel(
        &self,
        channel_id: &str,
    ) -> Result<models::ChannelInformation, ApiError> {
        self.query("GET", &format!("channels/{channel_id}"), None, None)
            .await
    }

    /// Get public channels' information.
    ///
    /// Requires the "list_team_channels" permission.
    pub async fn get_public_channels(
        &self,
        team_id: &str,
    ) -> Result<Vec<models::ChannelInformation>, ApiError> {
        self.query("GET", &format!("teams/{team_id}/channels"), None, None)
            .await
    }

    /// Create a new post from the given body.
    ///
    /// ```rust,no_run
    /// # use mattermost_api::{models, prelude::*};
    /// # async fn execute() {
    /// # let api = Mattermost::new("", AuthenticationData::from_password("", "")).unwrap();
    /// let body = models::PostBody {
    ///     channel_id: "some-channel-id".into(),
    ///     message: "Hello, channel!".into(),
    ///     root_id: None,
    /// };
    /// let response = api.create_post(&body).await.unwrap();
    /// # }
    /// ```
    /// Must have "create_post" permission for the channel the post is being created in.
    pub async fn create_post(&self, body: &models::PostBody) -> Result<models::Post, ApiError> {
        self.post("posts", None, body).await
    }
}

#[cfg(test)]
mod url_tests {
    use super::{AuthenticationData, Mattermost};
    use crate::errors::ApiError;

    impl PartialEq for ApiError {
        fn eq(&self, other: &Self) -> bool {
            self.to_string() == other.to_string()
        }
    }

    #[test]
    fn invalid_instance_url_fails_fast() {
        let Err(err) = Mattermost::new("herp derp", AuthenticationData::from_access_token("x"))
        else {
            panic!("Expected an error")
        };

        assert_eq!(err, ApiError::UrlError(url::ParseError::EmptyHost))
    }

    #[test]
    fn api_v4_path_is_added_by_default() {
        let client = Mattermost::new(
            "http://www.mattermost.com",
            AuthenticationData::from_access_token("x"),
        )
        .expect("This should work");

        assert_eq!(
            client.instance_url.as_str(),
            "http://www.mattermost.com/api/v4/"
        )
    }

    #[test]
    fn api_path_can_be_overridden() {
        let client = Mattermost::new(
            "http://www.mattermost.com/ipa/v5/",
            AuthenticationData::from_access_token("x"),
        )
        .expect("This should work");

        assert_eq!(
            client.instance_url.as_str(),
            "http://www.mattermost.com/ipa/v5/"
        )
    }

    #[test]
    fn http_urls_are_properly_converted_to_ws_urls() {
        let http_client = Mattermost::new(
            "http://www.mattermost.com",
            AuthenticationData::from_access_token("x"),
        )
        .unwrap();
        assert_eq!(
            http_client.ws_instance_url().unwrap().as_str(),
            "ws://www.mattermost.com/api/v4/"
        );

        let https_client = Mattermost::new(
            "https://www.mattermost.com/",
            AuthenticationData::from_access_token("x"),
        )
        .unwrap();
        assert_eq!(
            https_client.ws_instance_url().unwrap().as_str(),
            "wss://www.mattermost.com/api/v4/"
        );
    }

    #[test]
    fn endpoint_urls_are_joined_as_expected() {
        let client = Mattermost::new(
            "https://www.mattermost.com",
            AuthenticationData::from_access_token("x"),
        )
        .unwrap();

        assert_eq!(
            client.endpoint_url("/herp/derp").unwrap().as_str(),
            "https://www.mattermost.com/api/v4/herp/derp",
        );
    }
}
