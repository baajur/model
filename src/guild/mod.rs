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

impl Guild {
    /// Calculate a [`Member`]'s permissions in the guild.
    ///
    /// [`Member`]: struct.Member.html
    pub fn member_permissions<U>(&self, user_id: U) -> Permissions
        where U: Into<UserId> {
        let user_id = user_id.into();

        if user_id == self.owner_id {
            return Permissions::all();
        }

        let everyone = match self.roles.get(&RoleId(self.id.0)) {
            Some(everyone) => everyone,
            None => {
                error!(
                    "(╯°□°）╯︵ ┻━┻ @everyone role ({}) missing in '{}'",
                    self.id,
                    self.name,
                );

                return Permissions::empty();
            },
        };

        let member = match self.members.get(&user_id) {
            Some(member) => member,
            None => return everyone.permissions,
        };

        let mut permissions = everyone.permissions;

        for role in &member.roles {
            if let Some(role) = self.roles.get(&role) {
                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                    return Permissions::all();
                }

                permissions |= role.permissions;
            } else {
                warn!(
                    "(╯°□°）╯︵ ┻━┻ {} on {} has non-existent role {:?}",
                    member.user.id,
                    self.id,
                    role,
                );
            }
        }

        permissions
    }

    /// Gets a list of all the members (satisfying the status provided to the function) in this
    /// guild.
    pub fn members_with_status(&self, status: OnlineStatus) -> Vec<&Member> {
        let mut members = vec![];

        for (&id, member) in &self.members {
            match self.presences.get(&id) {
                Some(presence) => if status == presence.status {
                    members.push(member);
                },
                None => continue,
            }
        }

        members
    }

    /// Retrieves the first [`Member`] found that matches the name - with an
    /// optional discriminator - provided.
    ///
    /// Searching with a discriminator given is the most precise form of lookup,
    /// as no two people can share the same username *and* discriminator.
    ///
    /// If a member can not be found by username or username#discriminator,
    /// then a search will be done for the nickname. When searching by nickname,
    /// the hash (`#`) and everything after it is included in the search.
    ///
    /// The following are valid types of searches:
    ///
    /// - **username**: "zey"
    /// - **username and discriminator**: "zey#5479"
    /// - **nickname**: "zeyla" or "zeylas#nick"
    ///
    /// [`Member`]: struct.Member.html
    pub fn member_named(&self, name: &str) -> Option<&Member> {
        let (name, discrim) = if let Some(pos) = name.find('#') {
            let split = name.split_at(pos);

            match split.1.parse::<u16>() {
                Ok(discrim_int) => (split.0, Some(discrim_int)),
                Err(_) => (name, None),
            }
        } else {
            (&name[..], None)
        };

        self.members
            .values()
            .find(|member| {
                let name_matches = member.user.name == name;
                let discrim_matches = match discrim {
                    Some(discrim) => member.user.discriminator == discrim,
                    None => true,
                };

                name_matches && discrim_matches
            })
            .or_else(|| {
                self.members.values().find(|member| {
                    member.nick.as_ref().map_or(false, |nick| nick == name)
                })
            })
    }

    /// Retrieves all [`Member`] that start with a given `String`.
    ///
    /// `sorted` decides whether the best early match of the `prefix`
    /// should be the criteria to sort the result.
    /// For the `prefix` "zey" and the unsorted result:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    /// It would be sorted:
    /// - "zeya", "zeyaa", "zeyla", "zeyzey", "zeyzeyzey"
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_starting_with(&self, prefix: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member|

                if case_sensitive {
                    member.user.name.starts_with(prefix)
                } else {
                    starts_with_case_insensitive(&member.user.name, prefix)
                }

                || member.nick.as_ref()
                    .map_or(false, |nick|

                    if case_sensitive {
                        nick.starts_with(prefix)
                    } else {
                        starts_with_case_insensitive(nick, prefix)
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = match a.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&a.user.name[..], prefix) {
                                a.user.name.clone()
                            } else {
                                nick.clone()
                            }
                        },
                        None => a.user.name.clone(),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&b.user.name[..], prefix) {
                                b.user.name.clone()
                            } else {
                                nick.clone()
                            }
                        },
                        None => b.user.name.clone(),
                    };

                    closest_to_origin(prefix, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` as
    /// either username or nick, with a priority on username.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sorted` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// It will look at the account name first, if that does not fit the
    /// search-criteria `substring`, the display-name will be considered.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Due to two fields of a `Member` being candidates for
    /// the searched field, setting `sorted` to `true` will result in an overhead,
    /// as both fields have to be considered again for sorting.
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member|

                if case_sensitive {
                    member.user.name.contains(substring)
                } else {
                    contains_case_insensitive(&member.user.name, substring)
                }

                || member.nick.as_ref()
                    .map_or(false, |nick| {

                        if case_sensitive {
                            nick.contains(substring)
                        } else {
                            contains_case_insensitive(nick, substring)
                        }
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = match a.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&a.user.name[..], substring) {
                                a.user.name.clone()
                            } else {
                                nick.clone()
                            }
                        },
                        None => a.user.name.clone(),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                            if contains_case_insensitive(&b.user.name[..], substring) {
                                b.user.name.clone()
                            } else {
                                nick.clone()
                            }
                        },
                        None => b.user.name.clone(),
                    };

                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` in
    /// their username.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_username_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member| {
                if case_sensitive {
                    member.user.name.contains(substring)
                } else {
                    contains_case_insensitive(&member.user.name, substring)
                }
            }).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = &a.user.name;
                    let name_b = &b.user.name;
                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Retrieves all [`Member`] containing a given `String` in
    /// their nick.
    ///
    /// If the substring is "yla", following results are possible:
    /// - "zeyla", "meiyla", "yladenisyla"
    /// If 'case_sensitive' is false, the following are not found:
    /// - "zeYLa", "meiyLa", "LYAdenislyA"
    ///
    /// `sort` decides whether the best early match of the search-term
    /// should be the criteria to sort the result.
    /// For the `substring` "zey" and the unsorted result:
    /// - "azey", "zey", "zeyla", "zeylaa", "zeyzeyzey"
    /// It would be sorted:
    /// - "zey", "azey", "zeyla", "zeylaa", "zeyzeyzey"
    ///
    /// **Note**: Instead of panicing, when sorting does not find
    /// a nick, the username will be used (this should never happen).
    ///
    /// [`Member`]: struct.Member.html
    pub fn members_nick_containing(&self, substring: &str, case_sensitive: bool, sorted: bool) -> Vec<&Member> {
        let mut members: Vec<&Member> = self.members
            .values()
            .filter(|member|
                member.nick.as_ref()
                    .map_or(false, |nick| {

                        if case_sensitive {
                            nick.contains(substring)
                        } else {
                            contains_case_insensitive(nick, substring)
                        }
                    })).collect();

        if sorted {
            members
                .sort_by(|a, b| {
                    let name_a = match a.nick {
                        Some(ref nick) => {
                            nick.clone()
                        },
                        None => a.user.name.clone(),
                    };

                    let name_b = match b.nick {
                        Some(ref nick) => {
                                nick.clone()
                            },
                        None => b.user.name.clone(),
                    };

                    closest_to_origin(substring, &name_a[..], &name_b[..])
                });
            members
        } else {
            members
        }
    }

    /// Calculate a [`User`]'s permissions in a given channel in the guild.
    ///
    /// [`User`]: struct.User.html
    pub fn permissions_in<C, U>(&self, channel_id: C, user_id: U) -> Permissions
        where C: Into<ChannelId>, U: Into<UserId> {
        let user_id = user_id.into();

        // The owner has all permissions in all cases.
        if user_id == self.owner_id {
            return Permissions::all();
        }

        let channel_id = channel_id.into();

        // Start by retrieving the @everyone role's permissions.
        let everyone = match self.roles.get(&RoleId(self.id.0)) {
            Some(everyone) => everyone,
            None => {
                error!(
                    "(╯°□°）╯︵ ┻━┻ @everyone role ({}) missing in '{}'",
                    self.id,
                    self.name
                );

                return Permissions::empty();
            },
        };

        // Create a base set of permissions, starting with `@everyone`s.
        let mut permissions = everyone.permissions;

        let member = match self.members.get(&user_id) {
            Some(member) => member,
            None => return everyone.permissions,
        };

        for &role in &member.roles {
            if let Some(role) = self.roles.get(&role) {
                permissions |= role.permissions;
            } else {
                warn!(
                    "(╯°□°）╯︵ ┻━┻ {} on {} has non-existent role {:?}",
                    member.user.id,
                    self.id,
                    role
                );
            }
        }

        // Administrators have all permissions in any channel.
        if permissions.contains(Permissions::ADMINISTRATOR) {
            return Permissions::all();
        }

        if let Some(channel) = self.channels.get(&channel_id) {
            // If this is a text channel, then throw out voice permissions.
            if channel.kind == ChannelType::Text {
                permissions &= !(Permissions::CONNECT
                    | Permissions::SPEAK
                    | Permissions::MUTE_MEMBERS
                    | Permissions::DEAFEN_MEMBERS
                    | Permissions::MOVE_MEMBERS
                    | Permissions::USE_VAD);
            }

            // Apply the permission overwrites for the channel for each of the
            // overwrites that - first - applies to the member's roles, and then
            // the member itself.
            //
            // First apply the denied permission overwrites for each, then apply
            // the allowed.

            // Roles
            for overwrite in &channel.permission_overwrites {
                if let PermissionOverwriteType::Role(role) = overwrite.kind {
                    if role.0 != self.id.0 && !member.roles.contains(&role) {
                        continue;
                    }

                    permissions = (permissions & !overwrite.deny) | overwrite.allow;
                }
            }

            // Member
            for overwrite in &channel.permission_overwrites {
                if PermissionOverwriteType::Member(user_id) != overwrite.kind {
                    continue;
                }

                permissions = (permissions & !overwrite.deny) | overwrite.allow;
            }
        } else {
            warn!(
                "(╯°□°）╯︵ ┻━┻ Guild {} does not contain channel {}",
                self.id,
                channel_id
            );
        }

        // The default channel is always readable.
        if channel_id.0 == self.id.0 {
            permissions |= Permissions::READ_MESSAGES;
        }

        // No SEND_MESSAGES => no message-sending-related actions
        // If the member does not have the `SEND_MESSAGES` permission, then
        // throw out message-able permissions.
        if !permissions.contains(Permissions::SEND_MESSAGES) {
            permissions &= !(Permissions::SEND_TTS_MESSAGES
                | Permissions::MENTION_EVERYONE
                | Permissions::EMBED_LINKS
                | Permissions::ATTACH_FILES);
        }

        // If the member does not have the `READ_MESSAGES` permission, then
        // throw out actionable permissions.
        if !permissions.contains(Permissions::READ_MESSAGES) {
            permissions &= Permissions::KICK_MEMBERS
                | Permissions::BAN_MEMBERS
                | Permissions::ADMINISTRATOR
                | Permissions::MANAGE_GUILD
                | Permissions::CHANGE_NICKNAME
                | Permissions::MANAGE_NICKNAMES;
        }

        permissions
    }
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

/// Checks if a `&str` contains another `&str`.
fn contains_case_insensitive(to_look_at: &str, to_find: &str) -> bool {
    to_look_at.to_lowercase().contains(to_find)
}

/// Checks if a `&str` starts with another `&str`.
fn starts_with_case_insensitive(to_look_at: &str, to_find: &str) -> bool {
    to_look_at.to_lowercase().starts_with(to_find)
}

/// Takes a `&str` as `origin` and tests if either
/// `word_a` or `word_b` is closer.
///
/// **Note**: Normally `word_a` and `word_b` are
/// expected to contain `origin` as substring.
/// If not, using `closest_to_origin` would sort these
/// the end.
fn closest_to_origin(origin: &str, word_a: &str, word_b: &str) -> std::cmp::Ordering {
    let value_a = match word_a.find(origin) {
        Some(value) => value + word_a.len(),
        None => return std::cmp::Ordering::Greater,
    };

    let value_b = match word_b.find(origin) {
        Some(value) => value + word_b.len(),
        None => return std::cmp::Ordering::Less,
    };

    value_a.cmp(&value_b)
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
