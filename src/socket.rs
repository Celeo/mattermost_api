//! Websocket client and trait for interacting with the websocket API.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Websocket event broadcast information
#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketEventBroadcast {
    /// Users who were omitted from receiving the event
    pub omit_users: Option<HashMap<String, bool>>,
    /// Event recipient
    pub user_id: Option<String>,
    #[allow(missing_docs)]
    pub channel_id: String,
    #[allow(missing_docs)]
    pub team_id: String,
}

/// Event data from the websocket API
#[derive(Debug, Serialize, Deserialize)]
pub struct WebsocketEvent {
    /// Event type
    pub event: String,
    /// Event data
    pub data: serde_json::Value,
    /// Event recipient information
    pub broadcast: WebsocketEventBroadcast,
    /// Sequence number
    pub seq: usize,
}

/// Handler trait for receiving websocket messages.
///
/// Implement on a struct you create, and pass to
/// `connect_to_websocket` to connect to your
/// Mattermost instance's websocket API.
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
#[async_trait]
pub trait WebsocketHandler: Send + Sync {
    /// Function to implement to receive websocket messages.
    async fn callback(&self, _message: WebsocketEvent) {}
}

/// Websocket event names.
#[allow(missing_docs)]
pub mod websocket_event_types {
    pub const ADDED_TO_TEAM: &str = "added_to_team";
    pub const AUTHENTICATION_CHALLENGE: &str = "authentication_challenge";
    pub const CHANNEL_CONVERTED: &str = "channel_converted";
    pub const CHANNEL_CREATED: &str = "channel_created";
    pub const CHANNEL_DELETED: &str = "channel_deleted";
    pub const CHANNEL_MEMBER_UPDATED: &str = "channel_member_updated";
    pub const CHANNEL_UPDATED: &str = "channel_updated";
    pub const CHANNEL_VIEWED: &str = "channel_viewed";
    pub const CONFIG_CHANGED: &str = "config_changed";
    pub const DELETE_TEAM: &str = "delete_team";
    pub const DIRECT_ADDED: &str = "direct_added";
    pub const EMOJI_ADDED: &str = "emoji_added";
    pub const EPHEMERAL_MESSAGE: &str = "ephemeral_message";
    pub const GROUP_ADDED: &str = "group_added";
    pub const HELLO: &str = "hello";
    pub const LEAVE_TEAM: &str = "leave_team";
    pub const LICENSE_CHANGED: &str = "license_changed";
    pub const MEMBERROLE_UPDATED: &str = "memberrole_updated";
    pub const NEW_USER: &str = "new_user";
    pub const PLUGIN_DISABLED: &str = "plugin_disabled";
    pub const PLUGIN_ENABLED: &str = "plugin_enabled";
    pub const PLUGIN_STATUSES_CHANGED: &str = "plugin_statuses_changed";
    pub const POST_DELETED: &str = "post_deleted";
    pub const POST_EDITED: &str = "post_edited";
    pub const POST_UNREAD: &str = "post_unread";
    pub const POSTED: &str = "posted";
    pub const PREFERENCE_CHANGED: &str = "preference_changed";
    pub const PREFERENCES_CHANGED: &str = "preferences_changed";
    pub const PREFERENCES_DELETED: &str = "preferences_deleted";
    pub const REACTION_ADDED: &str = "reaction_added";
    pub const REACTION_REMOVED: &str = "reaction_removed";
    pub const RESPONSE: &str = "response";
    pub const ROLE_UPDATED: &str = "role_updated";
    pub const STATUS_CHANGE: &str = "status_change";
    pub const TYPING: &str = "typing";
    pub const UPDATE_TEAM: &str = "update_team";
    pub const USER_ADDED: &str = "user_added";
    pub const USER_REMOVED: &str = "user_removed";
    pub const USER_ROLE_UPDATED: &str = "user_role_updated";
    pub const USER_UPDATED: &str = "user_updated";
    pub const DIALOG_OPENED: &str = "dialog_opened";
    pub const THREAD_UPDATED: &str = "thread_updated";
    pub const THREAD_FOLLOW_CHANGED: &str = "thread_follow_changed";
    pub const THREAD_READ_CHANGED: &str = "thread_read_changed";
}
