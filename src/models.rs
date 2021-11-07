//! Struct models for API requests and responses.

#![allow(missing_docs)]

use crate::prelude::*;

/// Response struct from /teams/name/{name}
#[derive(Debug, Deserialize, Serialize)]
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
    pub policy_id: String,
}
