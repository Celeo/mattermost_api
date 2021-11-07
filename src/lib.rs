//! Mattermost API wrapper.
//!
//! For the full Mattermost API information, see [their docs].
//!
//! [their docs]: https://api.mattermost.com

#![deny(clippy::all)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod client;
pub mod errors;
pub mod models;
pub mod prelude;
