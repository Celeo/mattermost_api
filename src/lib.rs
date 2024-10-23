//! Mattermost API wrapper.
//!
//! For the full Mattermost API information, see [their docs].
//!
//! To start, create an instance of the [`AuthenticationData`]
//! struct, passing in either a login_id and password (likely
//! email and password), or a personal access token. Pass this
//! struct instance along with the URL of the target Mattermost
//! instance to [`Mattermost::new`].
//!
//! # Example
//!
//! ```rust,no_run
//! use mattermost_api::prelude::*;
//! # async fn run() {
//! let auth = AuthenticationData::from_password("you@example.com", "password");
//! let mut api = Mattermost::new("https://your-mattermost-instance.com", auth).unwrap();
//! api.store_session_token().await.unwrap();
//! let team_info = api.get_team("Best-Team-Ever").await.unwrap();
//! # }
//! ```
//!
//! [their docs]: https://api.mattermost.com
//! [`AuthenticationData`]: struct.AuthenticationData.html
//! [`Mattermost::new`]: struct.Mattermost.html

#![deny(clippy::all)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod errors;
pub mod models;
pub mod prelude;
pub mod socket;
/// Re-exported since websocket events have untyped data for now
pub use serde_json::Value;
