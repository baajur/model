use chrono::{DateTime, FixedOffset};
use std::fmt::{Display, Formatter, Result as FmtResult};
use ::utils::deserialize_single_recipient;
use ::*;

#[cfg(feature = "model")]
use builder::{CreateMessage, GetMessages};
#[cfg(feature = "model")]
use http::AttachmentType;
#[cfg(feature = "model")]
use internal::RwLockExt;

/// A Direct Message text channel with another user.
#[derive(Clone, Debug, Deserialize)]
pub struct PrivateChannel {
    /// The unique Id of the private channel.
    ///
    /// Can be used to calculate the first message's creation date.
    pub id: ChannelId,
    /// The Id of the last message sent.
    pub last_message_id: Option<MessageId>,
    /// Timestamp of the last time a [`Message`] was pinned.
    ///
    /// [`Message`]: struct.Message.html
    pub last_pin_timestamp: Option<DateTime<FixedOffset>>,
    /// Indicator of the type of channel this is.
    ///
    /// This should always be [`ChannelType::Private`].
    ///
    /// [`ChannelType::Private`]: enum.ChannelType.html#variant.Private
    #[serde(rename = "type")]
    pub kind: ChannelType,
    /// The recipient to the private channel.
    #[serde(deserialize_with = "deserialize_single_recipient", rename = "recipients")]
    pub recipient: User,
}

impl Display for PrivateChannel {
    /// Formats the private channel, displaying the recipient's username.
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&self.recipient.name)
    }
}
