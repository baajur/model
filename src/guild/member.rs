use ::*;
use chrono::{DateTime, FixedOffset};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(all(feature = "builder", feature = "cache", feature = "model"))]
use builder::EditMember;
#[cfg(all(feature = "cache", feature = "model"))]
use internal::prelude::*;
#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(all(feature = "cache", feature = "model", feature = "utils"))]
use utils::Colour;
#[cfg(all(feature = "cache", feature = "model"))]
use {CACHE, http, utils};

/// A trait for allowing both u8 or &str or (u8, &str) to be passed into the `ban` methods in `Guild` and `Member`.
pub trait BanOptions {
    fn dmd(&self) -> u8 { 0 }
    fn reason(&self) -> &str { "" }
}

impl BanOptions for u8 {
    fn dmd(&self) -> u8 { *self }
}

impl BanOptions for str {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for &'a str {
    fn reason(&self) -> &str { self }
}

impl BanOptions for String {
    fn reason(&self) -> &str { self }
}

impl<'a> BanOptions for (u8, &'a str) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { self.1 }
}

impl BanOptions for (u8, String) {
    fn dmd(&self) -> u8 { self.0 }

    fn reason(&self) -> &str { &self.1 }
}

/// Information about a member of a guild.
#[derive(Clone, Debug, Deserialize)]
pub struct Member {
    /// Indicator of whether the member can hear in voice channels.
    pub deaf: bool,
    /// The unique Id of the guild that the member is a part of.
    pub guild_id: GuildId,
    /// Timestamp representing the date when the member joined.
    pub joined_at: Option<DateTime<FixedOffset>>,
    /// Indicator of whether the member can speak in voice channels.
    pub mute: bool,
    /// The member's nickname, if present.
    ///
    /// Can't be longer than 32 characters.
    pub nick: Option<String>,
    /// Vector of Ids of [`Role`]s given to the member.
    pub roles: Vec<RoleId>,
    /// Attached User struct.
    pub user: User,
}

impl Display for Member {
    /// Mentions the user so that they receive a notification.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // assumes a `member` has already been bound
    /// println!("{} is a member!", member);
    /// ```
    ///
    // This is in the format of `<@USER_ID>`.
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.user.mention(), f)
    }
}
