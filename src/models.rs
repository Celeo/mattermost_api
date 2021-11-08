//! Struct models for API requests and responses.

#![allow(missing_docs)]

use crate::prelude::*;

/// Error struct from Mattermost.
///
/// See [here] for more information.
///
/// [here]: https://api.mattermost.com/#tag/errors
#[derive(Debug, Deserialize)]
pub struct MattermostError {
    pub id: String,
    pub message: String,
    pub request_id: String,
    pub status_code: i16,
    pub is_oauth: bool,
}

/// Response struct from /teams/name/{name}
#[derive(Debug, Deserialize)]
pub struct TeamInformation {
    pub id: String,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub display_name: String,
    pub name: String,
    pub description: String,
    pub email: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub allowed_domains: String,
    pub invite_id: String,
    pub allow_open_invite: bool,
    pub policy_id: Option<String>,
}

/// Response struct from /users/{user_id}/teams/unread
#[derive(Debug, Deserialize)]
pub struct TeamsUnreadInformation {
    pub teams_id: String,
    pub msg_count: u64,
    pub mention_count: u64,
}

/// Information about a single channel on the instance.
#[derive(Debug, Deserialize)]
pub struct ChannelInformation {
    //
}
