use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Read;
use std::mem;
use ::client::rest;
use ::internal::prelude::*;
use ::model::*;
use ::utils::builder::{CreateInvite, CreateMessage, EditChannel, GetMessages};

#[cfg(feature="cache")]
use ::client::CACHE;

/// Represents a guild's text or voice channel. Some methods are available only
/// for voice channels and some are only available for text channels.
#[derive(Clone, Debug, Deserialize)]
pub struct GuildChannel {
    /// The unique Id of the channel.
    ///
    /// The default channel Id shares the Id of the guild and the default role.
    pub id: ChannelId,
    /// The bitrate of the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub bitrate: Option<u64>,
    /// The Id of the guild the channel is located in.
    ///
    /// If this matches with the [`id`], then this is the default text channel.
    ///
    /// The original voice channel has an Id equal to the guild's Id,
    /// incremented by one.
    pub guild_id: GuildId,
    /// The type of the channel.
    #[serde(rename="type")]
    pub kind: ChannelType,
    /// The Id of the last message sent in the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub last_message_id: Option<MessageId>,
    /// The timestamp of the time a pin was most recently made.
    ///
    /// **Note**: This is only available for text channels.
    pub last_pin_timestamp: Option<String>,
    /// The name of the channel.
    pub name: String,
    /// Permission overwrites for [`Member`]s and for [`Role`]s.
    ///
    /// [`Member`]: struct.Member.html
    /// [`Role`]: struct.Role.html
    pub permission_overwrites: Vec<PermissionOverwrite>,
    /// The position of the channel.
    ///
    /// The default text channel will _almost always_ have a position of `-1` or
    /// `0`.
    pub position: i64,
    /// The topic of the channel.
    ///
    /// **Note**: This is only available for text channels.
    pub topic: Option<String>,
    /// The maximum number of members allowed in the channel.
    ///
    /// **Note**: This is only available for voice channels.
    pub user_limit: Option<u64>,
}

impl GuildChannel {
    /// Broadcasts to the channel that the current user is typing.
    ///
    /// For bots, this is a good indicator for long-running commands.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn broadcast_typing(&self) -> Result<()> {
        self.id.broadcast_typing()
    }

    /// Creates an invite leading to the given channel.
    ///
    /// # Examples
    ///
    /// Create an invite that can only be used 5 times:
    ///
    /// ```rust,ignore
    /// let invite = channel.create_invite(|i| i.max_uses(5));
    /// ```
    pub fn create_invite<F>(&self, f: F) -> Result<RichInvite>
        where F: FnOnce(CreateInvite) -> CreateInvite {
        #[cfg(feature="cache")]
        {
            let req = permissions::CREATE_INVITE;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        rest::create_invite(self.id.0, &f(CreateInvite::default()).0)
    }

    /// Creates a [permission overwrite][`PermissionOverwrite`] for either a
    /// single [`Member`] or [`Role`] within a [`Channel`].
    ///
    /// Refer to the documentation for [`PermissionOverwrite`]s for more
    /// information.
    ///
    /// Requires the [Manage Channels] permission.
    ///
    /// # Examples
    ///
    /// Creating a permission overwrite for a member by specifying the
    /// [`PermissionOverwrite::Member`] variant, allowing it the [Send Messages]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,ignore
    /// use serenity::client::CACHE;
    /// use serenity::model::{ChannelId, PermissionOverwrite, permissions};
    ///
    /// let channel_id = 7;
    /// let user_id = 8;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let cache = CACHE.read().unwrap();
    /// let channel = cache.get_guild_channel(channel_id).unwrap();
    ///
    /// let _ = channel.create_permission(overwrite);
    /// ```
    ///
    /// Creating a permission overwrite for a role by specifying the
    /// [`PermissionOverwrite::Role`] variant, allowing it the [Manage Webhooks]
    /// permission, but denying the [Send TTS Messages] and [Attach Files]
    /// permissions:
    ///
    /// ```rust,ignore
    /// use serenity::client::CACHE;
    /// use serenity::model::{ChannelId, PermissionOverwrite, permissions};
    ///
    /// let channel_id = 7;
    /// let user_id = 8;
    ///
    /// let allow = permissions::SEND_MESSAGES;
    /// let deny = permissions::SEND_TTS_MESSAGES | permissions::ATTACH_FILES;
    /// let overwrite = PermissionOverwrite {
    ///     allow: allow,
    ///     deny: deny,
    ///     kind: PermissionOverwriteType::Member(user_id),
    /// };
    ///
    /// let cache = CACHE.read().unwrap();
    /// let channel = cache.get_guild_channel(channel_id).unwrap();
    ///
    /// let _ = channel.create_permission(overwrite);
    /// ```
    ///
    /// [`Channel`]: enum.Channel.html
    /// [`Member`]: struct.Member.html
    /// [`PermissionOverwrite`]: struct.PermissionOverwrite.html
    /// [`PermissionOverwrite::Member`]: struct.PermissionOverwrite.html#variant.Member
    /// [`PermissionOverwrite::Role`]: struct.PermissionOverwrite.html#variant.Role
    /// [`Role`]: struct.Role.html
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    /// [Send TTS Messages]: permissions/constant.SEND_TTS_MESSAGES.html
    #[inline]
    pub fn create_permission(&self, target: PermissionOverwrite) -> Result<()> {
        self.id.create_permission(target)
    }

    /// Deletes this channel, returning the channel on a successful deletion.
    pub fn delete(&self) -> Result<Channel> {
        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.delete()
    }

    /// Deletes all messages by Ids from the given vector in the channel.
    ///
    /// Refer to [`Channel::delete_messages`] for more information.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// **Note**: This uses bulk delete endpoint which is not available
    /// for user accounts.
    ///
    /// **Note**: Messages that are older than 2 weeks can't be deleted using
    /// this method.
    ///
    /// [`Channel::delete_messages`]: enum.Channel.html#method.delete_messages
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_messages(&self, message_ids: &[MessageId]) -> Result<()> {
        self.id.delete_messages(message_ids)
    }

    /// Deletes all permission overrides in the channel from a member
    /// or role.
    ///
    /// **Note**: Requires the [Manage Channel] permission.
    ///
    /// [Manage Channel]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn delete_permission(&self, permission_type: PermissionOverwriteType) -> Result<()> {
        self.id.delete_permission(permission_type)
    }

    /// Deletes the given [`Reaction`] from the channel.
    ///
    /// **Note**: Requires the [Manage Messages] permission, _if_ the current
    /// user did not perform the reaction.
    ///
    /// [`Reaction`]: struct.Reaction.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn delete_reaction<M, R>(&self, message_id: M, user_id: Option<UserId>, reaction_type: R)
        -> Result<()> where M: Into<MessageId>, R: Into<ReactionType> {
        self.id.delete_reaction(message_id, user_id, reaction_type)
    }

    /// Modifies a channel's settings, such as its position or name.
    ///
    /// Refer to `EditChannel`s documentation for a full list of methods.
    ///
    /// # Examples
    ///
    /// Change a voice channels name and bitrate:
    ///
    /// ```rust,ignore
    /// channel.edit(|c| c.name("test").bitrate(86400));
    /// ```
    pub fn edit<F>(&mut self, f: F) -> Result<()>
        where F: FnOnce(EditChannel) -> EditChannel {

        #[cfg(feature="cache")]
        {
            let req = permissions::MANAGE_CHANNELS;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        let mut map = Map::new();
        map.insert("name".to_owned(), Value::String(self.name.clone()));
        map.insert("position".to_owned(), Value::Number(Number::from(self.position)));
        map.insert("type".to_owned(), Value::String(self.kind.name().to_owned()));

        let edited = f(EditChannel(map)).0;

        match rest::edit_channel(self.id.0, &edited) {
            Ok(channel) => {
                mem::replace(self, channel);

                Ok(())
            },
            Err(why) => Err(why),
        }
    }

    /// Edits a [`Message`] in the channel given its Id.
    ///
    /// Message editing preserves all unchanged message data.
    ///
    /// Refer to the documentation for [`CreateMessage`] for more information
    /// regarding message restrictions and requirements.
    ///
    /// **Note**: Requires that the current user be the author of the message.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the [`the limit`], containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [`CreateMessage`]: ../utils/builder/struct.CreateMessage.html
    /// [`Message`]: struct.Message.html
    /// [`the limit`]: ../utils/builder/struct.CreateMessage.html#method.content
    #[inline]
    pub fn edit_message<F, M>(&self, message_id: M, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, M: Into<MessageId> {
        self.id.edit_message(message_id, f)
    }

    /// Attempts to find this channel's guild in the Cache.
    ///
    /// **Note**: Right now this performs a clone of the guild. This will be
    /// optimized in the future.
    #[cfg(feature="cache")]
    pub fn guild(&self) -> Option<Arc<RwLock<Guild>>> {
        CACHE.read().unwrap().guild(self.guild_id)
    }

    /// Gets all of the channel's invites.
    ///
    /// Requires the [Manage Channels] permission.
    /// [Manage Channels]: permissions/constant.MANAGE_CHANNELS.html
    #[inline]
    pub fn invites(&self) -> Result<Vec<RichInvite>> {
        self.id.invites()
    }

    /// Gets a message from the channel.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.id.message(message_id)
    }

    /// Gets messages from the channel.
    ///
    /// Refer to [`Channel::get_messages`] for more information.
    ///
    /// Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_messages`]: enum.Channel.html#method.get_messages
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    #[inline]
    pub fn messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.id.messages(f)
    }

    /// Pins a [`Message`] to the channel.
    #[inline]
    pub fn pin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id.pin(message_id)
    }

    /// Gets all channel's pins.
    #[inline]
    pub fn pins(&self) -> Result<Vec<Message>> {
        self.id.pins()
    }

    /// Gets the list of [`User`]s who have reacted to a [`Message`] with a
    /// certain [`Emoji`].
    ///
    /// Refer to [`Channel::get_reaction_users`] for more information.
    ///
    /// **Note**: Requires the [Read Message History] permission.
    ///
    /// [`Channel::get_reaction_users`]: enum.Channel.html#variant.get_reaction_users
    /// [`Emoji`]: struct.Emoji.html
    /// [`Message`]: struct.Message.html
    /// [`User`]: struct.User.html
    /// [Read Message History]: permissions/constant.READ_MESSAGE_HISTORY.html
    pub fn reaction_users<M, R, U>(&self,
                                   message_id: M,
                                   reaction_type: R,
                                   limit: Option<u8>,
                                   after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.id.reaction_users(message_id, reaction_type, limit, after)
    }

    /// Sends a message with just the given message content in the channel.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// [`ChannelId`]: ../model/struct.ChannelId.html
    /// [`ClientError::MessageTooLong`]: enum.ClientError.html#variant.MessageTooLong
    #[inline]
    pub fn say(&self, content: &str) -> Result<Message> {
        self.id.say(content)
    }

    /// Sends a file along with optional message contents. The filename _must_
    /// be specified.
    ///
    /// Refer to [`ChannelId::send_file`] for examples and more information.
    ///
    /// The [Attach Files] and [Send Messages] permissions are required.
    ///
    /// **Note**: Message contents must be under 2000 unicode code points.
    ///
    /// # Errors
    ///
    /// If the content of the message is over the above limit, then a
    /// [`ClientError::MessageTooLong`] will be returned, containing the number
    /// of unicode code points over the limit.
    ///
    /// [`ChannelId::send_file`]: struct.ChannelId.html#method.send_file
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [Attach Files]: permissions/constant.ATTACH_FILES.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_file<F, R>(&self, file: R, filename: &str, f: F) -> Result<Message>
        where F: FnOnce(CreateMessage) -> CreateMessage, R: Read {
        self.id.send_file(file, filename, f)
    }

    /// Sends a message to the channel with the given content.
    ///
    /// **Note**: This will only work when a [`Message`] is received.
    ///
    /// **Note**: Requires the [Send Messages] permission.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError::MessageTooLong`] if the content of the message
    /// is over the above limit, containing the number of unicode code points
    /// over the limit.
    ///
    /// Returns a [`ClientError::InvalidPermissions`] if the current user does
    /// not have the required permissions.
    ///
    /// [`ClientError::InvalidPermissions`]: ../client/enum.ClientError.html#variant.InvalidPermissions
    /// [`ClientError::MessageTooLong`]: ../client/enum.ClientError.html#variant.MessageTooLong
    /// [`Message`]: struct.Message.html
    /// [Send Messages]: permissions/constant.SEND_MESSAGES.html
    pub fn send_message<F: FnOnce(CreateMessage) -> CreateMessage>(&self, f: F) -> Result<Message> {
        #[cfg(feature="cache")]
        {
            let req = permissions::SEND_MESSAGES;

            if !utils::user_has_perms(self.id, req)? {
                return Err(Error::Client(ClientError::InvalidPermissions(req)));
            }
        }

        self.id.send_message(f)
    }

    /// Unpins a [`Message`] in the channel given by its Id.
    ///
    /// Requires the [Manage Messages] permission.
    ///
    /// [`Message`]: struct.Message.html
    /// [Manage Messages]: permissions/constant.MANAGE_MESSAGES.html
    #[inline]
    pub fn unpin<M: Into<MessageId>>(&self, message_id: M) -> Result<()> {
        self.id.unpin(message_id)
    }

    /// Retrieves the channel's webhooks.
    ///
    /// **Note**: Requires the [Manage Webhooks] permission.
    ///
    /// [Manage Webhooks]: permissions/constant.MANAGE_WEBHOOKS.html
    #[inline]
    pub fn webhooks(&self) -> Result<Vec<Webhook>> {
        self.id.webhooks()
    }

    /// Alias of [`invites`].
    ///
    /// [`invites`]: #method.invites
    #[deprecated(since="0.1.5", note="Use `invites` instead.")]
    #[inline]
    pub fn get_invites(&self) -> Result<Vec<RichInvite>> {
        self.invites()
    }

    /// Alias of [`message`].
    ///
    /// [`message`]: #method.message
    #[deprecated(since="0.1.5", note="Use `message` instead.")]
    #[inline]
    pub fn get_message<M: Into<MessageId>>(&self, message_id: M) -> Result<Message> {
        self.message(message_id)
    }

    /// Alias of [`messages`].
    ///
    /// [`messages`]: #method.messages
    #[deprecated(since="0.1.5", note="Use `messages` instead.")]
    #[inline]
    pub fn get_messages<F>(&self, f: F) -> Result<Vec<Message>>
        where F: FnOnce(GetMessages) -> GetMessages {
        self.messages(f)
    }

    /// Alias of [`reaction_users`].
    ///
    /// [`reaction_users`]: #method.reaction_users
    #[deprecated(since="0.1.5", note="Use `reaction_users` instead.")]
    #[inline]
    pub fn get_reaction_users<M, R, U>(&self,
                                       message_id: M,
                                       reaction_type: R,
                                       limit: Option<u8>,
                                       after: Option<U>)
        -> Result<Vec<User>> where M: Into<MessageId>, R: Into<ReactionType>, U: Into<UserId> {
        self.reaction_users(message_id, reaction_type, limit, after)
    }

    /// Alias of [`webhooks`].
    ///
    /// [`webhooks`]: #method.webhooks
    #[deprecated(since="0.1.5", note="Use `webhooks` instead.")]
    #[inline]
    pub fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        self.webhooks()
    }
}

impl Display for GuildChannel {
    /// Formats the channel, creating a mention of it.
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.id.mention(), f)
    }
}
