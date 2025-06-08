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
    pub event: WebsocketEventType,
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
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebsocketEventType {
    AddedToTeam,
    AuthenticationChallenge,
    ChannelConverted,
    ChannelCreated,
    ChannelDeleted,
    ChannelMemberUpdated,
    ChannelUpdated,
    ChannelViewed,
    ConfigChanged,
    DeleteTeam,
    DirectAdded,
    EmojiAdded,
    EphemeralMessage,
    GroupAdded,
    Hello,
    LeaveTeam,
    LicenseChanged,
    MemberroleUpdated,
    NewUser,
    PluginDisabled,
    PluginEnabled,
    PluginStatusesChanged,
    PostDeleted,
    PostEdited,
    PostUnread,
    Posted,
    PreferenceChanged,
    PreferencesChanged,
    PreferencesDeleted,
    ReactionAdded,
    ReactionRemoved,
    Response,
    RoleUpdated,
    StatusChange,
    Typing,
    UpdateTeam,
    UserAdded,
    UserRemoved,
    UserRoleUpdated,
    UserUpdated,
    DialogOpened,
    ThreadUpdated,
    ThreadFollowChanged,
    ThreadReadChanged,
}
