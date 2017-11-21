use ::*;
use super::super::utils::{deserialize_emojis, deserialize_roles};

#[cfg(feature = "model")]
use builder::{EditGuild, EditMember, EditRole};

/// Partial information about a [`Guild`]. This does not include information
/// like member data.
///
/// [`Guild`]: struct.Guild.html
#[derive(Clone, Debug, Deserialize)]
pub struct PartialGuild {
    pub id: GuildId,
    pub afk_channel_id: Option<ChannelId>,
    pub afk_timeout: u64,
    pub default_message_notifications: u64,
    pub embed_channel_id: Option<ChannelId>,
    pub embed_enabled: bool,
    #[serde(deserialize_with = "deserialize_emojis")] pub emojis: HashMap<EmojiId, Emoji>,
    /// Features enabled for the guild.
    ///
    /// Refer to [`Guild::features`] for more information.
    ///
    /// [`Guild::features`]: struct.Guild.html#structfield.features
    pub features: Vec<String>,
    pub icon: Option<String>,
    pub mfa_level: u64,
    pub name: String,
    pub owner_id: UserId,
    pub region: String,
    #[serde(deserialize_with = "deserialize_roles")] pub roles: HashMap<RoleId, Role>,
    pub splash: Option<String>,
    pub verification_level: VerificationLevel,
}
