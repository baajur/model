use ::*;

#[cfg(feature = "model")]
use std::borrow::Cow;
#[cfg(feature = "model")]
use std::fmt::Write as FmtWrite;

impl From<Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: Channel) -> ChannelId {
        match channel {
            Channel::Group(group) => group.channel_id,
            Channel::Guild(channel) => channel.id,
            Channel::Private(channel) => channel.id,
            Channel::Category(channel) => channel.id,
        }
    }
}

impl<'a> From<&'a Channel> for ChannelId {
    /// Gets the Id of a `Channel`.
    fn from(channel: &Channel) -> ChannelId {
        match *channel {
            Channel::Group(ref group) => group.channel_id,
            Channel::Guild(ref channel) => channel.id,
            Channel::Private(ref channel) => channel.id,
            Channel::Category(ref channel) => channel.id,
        }
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId { private_channel.id }
}

impl<'a> From<&'a PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: &PrivateChannel) -> ChannelId { private_channel.id }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId { public_channel.id }
}
impl<'a> From<&'a GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId { public_channel.id }
}
