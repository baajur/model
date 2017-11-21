mod emoji;
mod guild_id;
mod integration;
mod member;
mod partial_guild;
mod role;
mod audit_log;

pub use self::emoji::*;
pub use self::guild_id::*;
pub use self::integration::*;
pub use self::member::*;
pub use self::partial_guild::*;
pub use self::role::*;
pub use self::audit_log::*;

use chrono::{DateTime, FixedOffset};
use ::*;
use serde::de::Error as DeError;
use serde_json;
use super::utils::*;

#[cfg(all(feature = "cache", feature = "model"))]
use CACHE;
#[cfg(feature = "model")]
use http;
#[cfg(feature = "model")]
use builder::{EditGuild, EditMember, EditRole};
#[cfg(feature = "model")]
use constants::LARGE_THRESHOLD;
#[cfg(feature = "model")]
use std;

/// A representation of a banning of a user.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct Ban {
    /// The reason given for this ban.
    pub reason: Option<String>,
    /// The user that was banned.
    pub user: User,
}

/// Information about a Discord guild, such as channels, emojis, etc.
#[derive(Clone, Debug)]
pub struct Guild {
    /// Id of a voice channel that's considered the AFK channel.
    pub afk_channel_id: Option<ChannelId>,
    /// The amount of seconds a user can not show any activity in a voice
    /// channel before being moved to an AFK channel -- if one exists.
    pub afk_timeout: u64,
    /// All voice and text channels contained within a guild.
    ///
    /// This contains all channels regardless of permissions (i.e. the ability
    /// of the bot to read from or connect to them).
    pub channels: HashMap<ChannelId, GuildChannel>,
    /// Indicator of whether notifications for all messages are enabled by
    /// default in the guild.
    pub default_message_notifications: u64,
    /// All of the guild's custom emojis.
    pub emojis: HashMap<EmojiId, Emoji>,
    /// VIP features enabled for the guild. Can be obtained through the
    /// [Discord Partnership] website.
    ///
    /// The following is a list of known features:
    ///
    /// - `INVITE_SPLASH`
    /// - `VANITY_URL`
    /// - `VERIFIED`
    /// - `VIP_REGIONS`
    ///
    /// [Discord Partnership]: https://discordapp.com/partners
    pub features: Vec<String>,
    /// The hash of the icon used by the guild.
    ///
    /// In the client, this appears on the guild list on the left-hand side.
    pub icon: Option<String>,
    /// The unique Id identifying the guild.
    ///
    /// This is equivilant to the Id of the default role (`@everyone`) and also
    /// that of the default channel (typically `#general`).
    pub id: GuildId,
    /// The date that the current user joined the guild.
    pub joined_at: DateTime<FixedOffset>,
    /// Indicator of whether the guild is considered "large" by Discord.
    pub large: bool,
    /// The number of members in the guild.
    pub member_count: u64,
    /// Users who are members of the guild.
    ///
    /// Members might not all be available when the [`ReadyEvent`] is received
    /// if the [`member_count`] is greater than the `LARGE_THRESHOLD` set by
    /// the library.
    ///
    /// [`ReadyEvent`]: events/struct.ReadyEvent.html
    pub members: HashMap<UserId, Member>,
    /// Indicator of whether the guild requires multi-factor authentication for
    /// [`Role`]s or [`User`]s with moderation permissions.
    ///
    /// [`Role`]: struct.Role.html
    /// [`User`]: struct.User.html
    pub mfa_level: u64,
    /// The name of the guild.
    pub name: String,
    /// The Id of the [`User`] who owns the guild.
    ///
    /// [`User`]: struct.User.html
    pub owner_id: UserId,
    /// A mapping of [`User`]s' Ids to their current presences.
    ///
    /// [`User`]: struct.User.html
    pub presences: HashMap<UserId, Presence>,
    /// The region that the voice servers that the guild uses are located in.
    pub region: String,
    /// A mapping of the guild's roles.
    pub roles: HashMap<RoleId, Role>,
    /// An identifying hash of the guild's splash icon.
    ///
    /// If the [`InviteSplash`] feature is enabled, this can be used to generate
    /// a URL to a splash image.
    ///
    /// [`InviteSplash`]: enum.Feature.html#variant.InviteSplash
    pub splash: Option<String>,
    /// Indicator of the current verification level of the guild.
    pub verification_level: VerificationLevel,
    /// A mapping of of [`User`]s to their current voice state.
    ///
    /// [`User`]: struct.User.html
    pub voice_states: HashMap<UserId, VoiceState>,
}

impl<'de> Deserialize<'de> for Guild {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let mut map = JsonMap::deserialize(deserializer)?;

        let id = map.get("id")
            .and_then(|x| x.as_str())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(guild_id) = id {
            if let Some(array) = map.get_mut("channels").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(channel) = value.as_object_mut() {
                        channel
                            .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                    }
                }
            }

            if let Some(array) = map.get_mut("members").and_then(|x| x.as_array_mut()) {
                for value in array {
                    if let Some(member) = value.as_object_mut() {
                        member
                            .insert("guild_id".to_string(), Value::Number(Number::from(guild_id)));
                    }
                }
            }
        }

        let afk_channel_id = match map.remove("afk_channel_id") {
            Some(v) => serde_json::from_value::<Option<ChannelId>>(v)
                .map_err(DeError::custom)?,
            None => None,
        };
        let afk_timeout = map.remove("afk_timeout")
            .ok_or_else(|| DeError::custom("expected guild afk_timeout"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let channels = map.remove("channels")
            .ok_or_else(|| DeError::custom("expected guild channels"))
            .and_then(deserialize_guild_channels)
            .map_err(DeError::custom)?;
        let default_message_notifications = map.remove("default_message_notifications")
            .ok_or_else(|| {
                DeError::custom("expected guild default_message_notifications")
            })
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let emojis = map.remove("emojis")
            .ok_or_else(|| DeError::custom("expected guild emojis"))
            .and_then(deserialize_emojis)
            .map_err(DeError::custom)?;
        let features = map.remove("features")
            .ok_or_else(|| DeError::custom("expected guild features"))
            .and_then(serde_json::from_value::<Vec<String>>)
            .map_err(DeError::custom)?;
        let icon = match map.remove("icon") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let id = map.remove("id")
            .ok_or_else(|| DeError::custom("expected guild id"))
            .and_then(GuildId::deserialize)
            .map_err(DeError::custom)?;
        let joined_at = map.remove("joined_at")
            .ok_or_else(|| DeError::custom("expected guild joined_at"))
            .and_then(DateTime::deserialize)
            .map_err(DeError::custom)?;
        let large = map.remove("large")
            .ok_or_else(|| DeError::custom("expected guild large"))
            .and_then(bool::deserialize)
            .map_err(DeError::custom)?;
        let member_count = map.remove("member_count")
            .ok_or_else(|| DeError::custom("expected guild member_count"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let members = map.remove("members")
            .ok_or_else(|| DeError::custom("expected guild members"))
            .and_then(deserialize_members)
            .map_err(DeError::custom)?;
        let mfa_level = map.remove("mfa_level")
            .ok_or_else(|| DeError::custom("expected guild mfa_level"))
            .and_then(u64::deserialize)
            .map_err(DeError::custom)?;
        let name = map.remove("name")
            .ok_or_else(|| DeError::custom("expected guild name"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let owner_id = map.remove("owner_id")
            .ok_or_else(|| DeError::custom("expected guild owner_id"))
            .and_then(UserId::deserialize)
            .map_err(DeError::custom)?;
        let presences = map.remove("presences")
            .ok_or_else(|| DeError::custom("expected guild presences"))
            .and_then(deserialize_presences)
            .map_err(DeError::custom)?;
        let region = map.remove("region")
            .ok_or_else(|| DeError::custom("expected guild region"))
            .and_then(String::deserialize)
            .map_err(DeError::custom)?;
        let roles = map.remove("roles")
            .ok_or_else(|| DeError::custom("expected guild roles"))
            .and_then(deserialize_roles)
            .map_err(DeError::custom)?;
        let splash = match map.remove("splash") {
            Some(v) => Option::<String>::deserialize(v).map_err(DeError::custom)?,
            None => None,
        };
        let verification_level = map.remove("verification_level")
            .ok_or_else(|| DeError::custom("expected guild verification_level"))
            .and_then(VerificationLevel::deserialize)
            .map_err(DeError::custom)?;
        let voice_states = map.remove("voice_states")
            .ok_or_else(|| DeError::custom("expected guild voice_states"))
            .and_then(deserialize_voice_states)
            .map_err(DeError::custom)?;

        Ok(Self {
            afk_channel_id: afk_channel_id,
            afk_timeout: afk_timeout,
            channels: channels,
            default_message_notifications: default_message_notifications,
            emojis: emojis,
            features: features,
            icon: icon,
            id: id,
            joined_at: joined_at,
            large: large,
            member_count: member_count,
            members: members,
            mfa_level: mfa_level,
            name: name,
            owner_id: owner_id,
            presences: presences,
            region: region,
            roles: roles,
            splash: splash,
            verification_level: verification_level,
            voice_states: voice_states,
        })
    }
}

/// Information relating to a guild's widget embed.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct GuildEmbed {
    /// The Id of the channel to show the embed for.
    pub channel_id: ChannelId,
    /// Whether the widget embed is enabled.
    pub enabled: bool,
}

/// Representation of the number of members that would be pruned by a guild
/// prune operation.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct GuildPrune {
    /// The number of members that would be pruned by the operation.
    pub pruned: u64,
}

/// Basic information about a guild.
#[derive(Clone, Debug, Deserialize)]
pub struct GuildInfo {
    /// The unique Id of the guild.
    ///
    /// Can be used to calculate creation date.
    pub id: GuildId,
    /// The hash of the icon of the guild.
    ///
    /// This can be used to generate a URL to the guild's icon image.
    pub icon: Option<String>,
    /// The name of the guild.
    pub name: String,
    /// Indicator of whether the current user is the owner.
    pub owner: bool,
    /// The permissions that the current user has.
    pub permissions: Permissions,
}

impl From<PartialGuild> for GuildContainer {
    fn from(guild: PartialGuild) -> GuildContainer { GuildContainer::Guild(guild) }
}

impl From<GuildId> for GuildContainer {
    fn from(guild_id: GuildId) -> GuildContainer { GuildContainer::Id(guild_id) }
}

impl From<u64> for GuildContainer {
    fn from(id: u64) -> GuildContainer { GuildContainer::Id(GuildId(id)) }
}

/// Data for an unavailable guild.
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct GuildUnavailable {
    /// The Id of the [`Guild`] that is unavailable.
    ///
    /// [`Guild`]: struct.Guild.html
    pub id: GuildId,
    /// Indicator of whether the guild is unavailable.
    ///
    /// This should always be `true`.
    pub unavailable: bool,
}

#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum GuildStatus {
    OnlinePartialGuild(PartialGuild),
    OnlineGuild(Guild),
    Offline(GuildUnavailable),
}

enum_number!(
    #[doc="The level to set as criteria prior to a user being able to send
    messages in a [`Guild`].

    [`Guild`]: struct.Guild.html"]
    VerificationLevel {
        /// Does not require any verification.
        None = 0,
        /// Must have a verified email on the user's Discord account.
        Low = 1,
        /// Must also be a registered user on Discord for longer than 5 minutes.
        Medium = 2,
        /// Must also be a member of the guild for longer than 10 minutes.
        High = 3,
        /// Must have a verified phone on the user's Discord account.
        Higher = 4,
    }
);
