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
