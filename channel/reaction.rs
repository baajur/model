use std::fmt::{Display, Formatter, Result as FmtResult, Write as FmtWrite};
use ::client::rest;
use ::internal::prelude::*;
use ::model::*;

#[cfg(feature="cache")]
use ::client::CACHE;

impl Reaction {
    /// Deletes the reaction, but only if the current user is the user who made
    /// the reaction or has permission to.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, then returns a
    /// [`ClientError::InvalidPermissions`] if the current user does not have
    /// the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    /// [permissions]: permissions
    pub fn delete(&self) -> Result<()> {
        let user_id = feature_cache! {{
            let user = if self.user_id == CACHE.read().unwrap().user.id {
                None
            } else {
                Some(self.user_id.0)
            };

            // If the reaction is one _not_ made by the current user, then ensure
            // that the current user has permission* to delete the reaction.
            //
            // Normally, users can only delete their own reactions.
            //
            // * The `Manage Messages` permission.
            if user.is_some() {
                let req = permissions::MANAGE_MESSAGES;

                if !utils::user_has_perms(self.channel_id, req).unwrap_or(true) {
                    return Err(Error::Client(ClientError::InvalidPermissions(req)));
                }
            }

            user
        } else {
            Some(self.user_id.0)
        }};

        rest::delete_reaction(self.channel_id.0,
                              self.message_id.0,
                              user_id,
                              &self.emoji)
    }

    /// Retrieves the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// The default `limit` is `50` - specify otherwise to receive a different
    /// maximum number of users. The maximum that may be retrieve at a time is
    /// `100`, if a greater number is provided then it is automatically reduced.
    ///
    /// The optional `after` attribute is to retrieve the users after a certain
    /// user. This is useful for pagination.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required [permissions].
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    /// [permissions]: permissions
    pub fn users<R, U>(&self,
                       reaction_type: R,
                       limit: Option<u8>,
                       after: Option<U>)
                       -> Result<Vec<User>>
                       where R: Into<ReactionType>,
                             U: Into<UserId> {
        rest::get_reaction_users(self.channel_id.0,
                                 self.message_id.0,
                                 &reaction_type.into(),
                                 limit.unwrap_or(50),
                                 after.map(|u| u.into().0))
    }
}

/// The type of a [`Reaction`] sent.
///
/// [`Reaction`]: struct.Reaction.html
#[derive(Clone, Debug)]
pub enum ReactionType {
    /// A reaction with a [`Guild`]s custom [`Emoji`], which is unique to the
    /// guild.
    ///
    /// [`Emoji`]: struct.Emoji.html
    /// [`Guild`]: struct.Guild.html
    Custom {
        /// The Id of the custom [`Emoji`].
        ///
        /// [`Emoji`]: struct.Emoji.html
        id: EmojiId,
        /// The name of the custom emoji. This is primarily used for decoration
        /// and distinguishing the emoji client-side.
        name: String,
    },
    /// A reaction with a twemoji.
    Unicode(String),
}

impl ReactionType {
    /// Creates a data-esque display of the type. This is not very useful for
    /// displaying, as the primary client can not render it, but can be useful
    /// for debugging.
    ///
    /// **Note**: This is mainly for use internally. There is otherwise most
    /// likely little use for it.
    pub fn as_data(&self) -> String {
        match *self {
            ReactionType::Custom { id, ref name } => {
                format!("{}:{}", name, id)
            },
            ReactionType::Unicode(ref unicode) => unicode.clone(),
        }
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Self> {
        let mut map = into_map(value)?;
        let name = remove(&mut map, "name").and_then(into_string)?;

        // Only custom emoji reactions (`ReactionType::Custom`) have an Id.
        Ok(match opt(&mut map, "id", EmojiId::decode)? {
            Some(id) => ReactionType::Custom {
                id: id,
                name: name,
            },
            None => ReactionType::Unicode(name),
        })
    }
}

impl From<Emoji> for ReactionType {
    fn from(emoji: Emoji) -> ReactionType {
        ReactionType::Custom {
            id: emoji.id,
            name: emoji.name,
        }
    }
}

impl From<String> for ReactionType {
    fn from(unicode: String) -> ReactionType {
        ReactionType::Unicode(unicode)
    }
}

impl Display for ReactionType {
    /// Formats the reaction type, displaying the associated emoji in a
    /// way that clients can understand.
    ///
    /// If the type is a [custom][`ReactionType::Custom`] emoji, then refer to
    /// the documentation for [emoji's formatter][`Emoji::fmt`] on how this is
    /// displayed. Otherwise, if the type is a
    /// [unicode][`ReactionType::Unicode`], then the inner unicode is displayed.
    ///
    /// [`Emoji::fmt`]: struct.Emoji.html#method.fmt
    /// [`ReactionType::Custom`]: enum.ReactionType.html#variant.Custom
    /// [`ReactionType::Unicode`]: enum.ReactionType.html#variant.Unicode
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            ReactionType::Custom { id, ref name } => {
                f.write_char('<')?;
                f.write_char(':')?;
                f.write_str(name)?;
                f.write_char(':')?;
                Display::fmt(&id, f)?;
                f.write_char('>')
            },
            ReactionType::Unicode(ref unicode) => f.write_str(unicode),
        }
    }
}
