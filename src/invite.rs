use chrono::{DateTime, FixedOffset};
use super::*;

#[cfg(feature = "model")]
use builder::CreateInvite;
#[cfg(feature = "model")]
use internal::prelude::*;
#[cfg(all(feature = "cache", feature = "model"))]
use super::{Permissions, utils as model_utils};
#[cfg(feature = "model")]
use {http, utils};

/// Information about an invite code.
///
/// Information can not be accessed for guilds the current user is banned from.
#[derive(Clone, Debug, Deserialize)]
pub struct Invite {
    /// The approximate number of [`Member`]s in the related [`Guild`].
    ///
    /// [`Guild`]: struct.Guild.html
    /// [`Member`]: struct.Member.html
    pub approximate_member_count: Option<u64>,
    /// The approximate number of [`Member`]s with an active session in the
    /// related [`Guild`].
    ///
    /// An active session is defined as an open, heartbeating WebSocket connection.
    /// These include [invisible][`OnlineStatus::Invisible`] members.
    ///
    /// [`OnlineStatus::Invisible`]: enum.OnlineStatus.html#variant.Invisible
    pub approximate_presence_count: Option<u64>,
    /// The unique code for the invite.
    pub code: String,
    /// A representation of the minimal amount of information needed about the
    /// [`GuildChannel`] being invited to.
    ///
    /// [`GuildChannel`]: struct.GuildChannel.html
    pub channel: InviteChannel,
    /// a representation of the minimal amount of information needed about the
    /// [`Guild`] being invited to.
    pub guild: InviteGuild,
}

/// A inimal information about the channel an invite points to.
#[derive(Clone, Debug, Deserialize)]
pub struct InviteChannel {
    pub id: ChannelId,
    pub name: String,
    #[serde(rename = "type")] pub kind: ChannelType,
}

/// A minimal amount of information about the guild an invite points to.
#[derive(Clone, Debug, Deserialize)]
pub struct InviteGuild {
    pub id: GuildId,
    pub icon: Option<String>,
    pub name: String,
    pub splash_hash: Option<String>,
    pub text_channel_count: Option<u64>,
    pub voice_channel_count: Option<u64>,
}

/// Detailed information about an invite.
/// This information can only be retrieved by anyone with the [Manage Guild]
/// permission. Otherwise, a minimal amount of information can be retrieved via
/// the [`Invite`] struct.
///
/// [`Invite`]: struct.Invite.html
/// [Manage Guild]: permissions/constant.MANAGE_GUILD.html
#[derive(Clone, Debug, Deserialize)]
pub struct RichInvite {
    /// A representation of the minimal amount of information needed about the
    /// channel being invited to.
    pub channel: InviteChannel,
    /// The unique code for the invite.
    pub code: String,
    /// When the invite was created.
    pub created_at: DateTime<FixedOffset>,
    /// A representation of the minimal amount of information needed about the
    /// guild being invited to.
    pub guild: InviteGuild,
    /// The user that created the invite.
    pub inviter: User,
    /// The maximum age of the invite in seconds, from when it was created.
    pub max_age: u64,
    /// The maximum number of times that an invite may be used before it expires.

    /// Note that this does not supercede the [`max_age`] value, if the value of
    /// [`temporary`] is `true`. If the value of `temporary` is `false`, then the
    /// invite _will_ self-expire after the given number of max uses.

    /// If the value is `0`, then the invite is permanent.
    ///
    /// [`max_age`]: #structfield.max_age
    /// [`temporary`]: #structfield.temporary
    pub max_uses: u64,
    /// Indicator of whether the invite self-expires after a certain amount of
    /// time or uses.
    pub temporary: bool,
    /// The amount of times that an invite has been used.
    pub uses: u64,
}
