use chrono::{DateTime, FixedOffset};
use ::*;

/// A group channel - potentially including other [`User`]s - separate from a
/// [`Guild`].
///
/// [`Guild`]: struct.Guild.html
/// [`User`]: struct.User.html
#[derive(Clone, Debug, Deserialize)]
pub struct Group {
    /// The Id of the group channel.
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    /// The optional icon of the group channel.
    pub icon: Option<String>,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the latest pinned message.
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    /// The name of the group channel.
    pub name: Option<String>,
    /// The Id of the group owner.
    pub owner_id: UserId,
    /// A map of the group's recipients.
    pub recipients: HashMap<UserId, User>,
}
