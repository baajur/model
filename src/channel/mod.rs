mod attachment;
mod channel_id;
mod embed;
mod group;
mod guild_channel;
mod message;
mod private_channel;
mod reaction;
mod channel_category;

pub use self::attachment::*;
pub use self::channel_id::*;
pub use self::embed::*;
pub use self::group::*;
pub use self::guild_channel::*;
pub use self::message::*;
pub use self::private_channel::*;
pub use self::reaction::*;
pub use self::channel_category::*;

use ::*;
use serde::de::Error as DeError;
use serde_json;
use super::utils::deserialize_u64;

#[cfg(feature = "model")]
use builder::{CreateMessage, GetMessages};
#[cfg(feature = "model")]
use http::AttachmentType;
#[cfg(feature = "model")]
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A container for any channel.
#[derive(Clone, Debug)]
pub enum Channel {
    /// A group. A group comprises of only one channel.
    Group(Group),
    /// A [text] or [voice] channel within a [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [text]: enum.ChannelType.html#variant.Text
    /// [voice]: enum.ChannelType.html#variant.Voice
    Guild(GuildChannel),
    /// A private channel to another [`User`]. No other users may access the
    /// channel. For multi-user "private channels", use a group.
    ///
    /// [`User`]: struct.User.html
    Private(PrivateChannel),
    /// A category of [`GuildChannel`]s
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    Category(ChannelCategory),
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let v = JsonMap::deserialize(deserializer)?;
        let kind = {
            let kind = v.get("type").ok_or_else(|| DeError::missing_field("type"))?;

            kind.as_u64().unwrap()
        };

        match kind {
            0 | 2 => serde_json::from_value::<GuildChannel>(Value::Object(v))
                .map(Channel::Guild)
                .map_err(DeError::custom),
            1 => serde_json::from_value::<PrivateChannel>(Value::Object(v))
                .map(Channel::Private)
                .map_err(DeError::custom),
            3 => serde_json::from_value::<Group>(Value::Object(v))
                .map(Channel::Group)
                .map_err(DeError::custom),
            4 => serde_json::from_value::<ChannelCategory>(Value::Object(v))
                .map(Channel::Category)
                .map_err(DeError::custom),
            _ => Err(DeError::custom("Unknown channel type")),
        }
    }
}

#[cfg(feature = "model")]
impl Display for Channel {
    /// Formats the channel into a "mentioned" string.
    ///
    /// This will return a different format for each type of channel:
    ///
    /// - [`Group`]s: the generated name retrievable via [`Group::name`];
    /// - [`PrivateChannel`]s: the recipient's name;
    /// - [`GuildChannel`]s: a string mentioning the channel that users who can
    /// see the channel can click on.
    ///
    /// [`Group`]: struct.Group.html
    /// [`Group::name`]: struct.Group.html#method.name
    /// [`GuildChannel`]: struct.GuildChannel.html
    /// [`PrivateChannel`]: struct.PrivateChannel.html
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            Channel::Group(ref group) => Display::fmt(&group.read().name(), f),
            Channel::Guild(ref ch) => Display::fmt(&ch.read().id.mention(), f),
            Channel::Private(ref ch) => {
                let channel = ch.read();
                let recipient = channel.recipient.read();

                Display::fmt(&recipient.name, f)
            },
            Channel::Category(ref category) => Display::fmt(&category.read().name, f),
        }
    }
}

enum_number!(
    /// A representation of a type of channel.
    ChannelType {
        #[doc="An indicator that the channel is a text [`GuildChannel`].

[`GuildChannel`]: struct.GuildChannel.html"]
        Text = 0,
        #[doc="An indicator that the channel is a [`PrivateChannel`].

[`PrivateChannel`]: struct.PrivateChannel.html"]
        Private = 1,
        #[doc="An indicator that the channel is a voice [`GuildChannel`].

[`GuildChannel`]: struct.GuildChannel.html"]
        Voice = 2,
        #[doc="An indicator that the channel is the channel of a [`Group`].

[`Group`]: struct.Group.html"]
        Group = 3,
        #[doc="An indicator that the channel is the channel of a [`ChannelCategory`].

[`ChannelCategory`]: struct.ChannelCategory.html"]
        Category = 4,
    }
);

impl ChannelType {
    pub fn name(&self) -> &str {
        match *self {
            ChannelType::Group => "group",
            ChannelType::Private => "private",
            ChannelType::Text => "text",
            ChannelType::Voice => "voice",
            ChannelType::Category => "category",
        }
    }
}

#[derive(Deserialize)]
struct PermissionOverwriteData {
    allow: Permissions,
    deny: Permissions,
    #[serde(deserialize_with = "deserialize_u64")] id: u64,
    #[serde(rename = "type")] kind: String,
}

/// A channel-specific permission overwrite for a member or role.
#[derive(Clone, Debug)]
pub struct PermissionOverwrite {
    pub allow: Permissions,
    pub deny: Permissions,
    pub kind: PermissionOverwriteType,
}

impl<'de> Deserialize<'de> for PermissionOverwrite {
    fn deserialize<D: Deserializer<'de>>(deserializer: D)
                                         -> StdResult<PermissionOverwrite, D::Error> {
        let data = PermissionOverwriteData::deserialize(deserializer)?;

        let kind = match &data.kind[..] {
            "member" => PermissionOverwriteType::Member(UserId(data.id)),
            "role" => PermissionOverwriteType::Role(RoleId(data.id)),
            _ => return Err(DeError::custom("Unknown PermissionOverwriteType")),
        };

        Ok(PermissionOverwrite {
            allow: data.allow,
            deny: data.deny,
            kind: kind,
        })
    }
}

/// The type of edit being made to a Channel's permissions.
///
/// This is for use with methods such as `GuildChannel::create_permission`.
///
/// [`GuildChannel::create_permission`]: struct.GuildChannel.html#method.create_permission
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PermissionOverwriteType {
    /// A member which is having its permission overwrites edited.
    Member(UserId),
    /// A role which is having its permission overwrites edited.
    Role(RoleId),
}
