//! The builder to create a new reply

use std::borrow::Cow;

use crate::serenity_prelude as serenity;

/// Message builder that abstracts over prefix and application command responses
#[derive(Default, Clone)]
#[allow(clippy::missing_docs_in_private_items)] // docs on setters
pub struct CreateReply<'a> {
    content: Option<Cow<'a, str>>,
    embeds: Vec<serenity::CreateEmbed<'a>>,
    attachments: Vec<serenity::CreateAttachment<'a>>,
    pub(crate) ephemeral: Option<bool>,
    #[cfg(feature = "unstable")]
    components: Option<Cow<'a, [serenity::CreateComponent<'a>]>>,
    #[cfg(not(feature = "unstable"))]
    components: Option<Cow<'a, [serenity::CreateActionRow<'a>]>>,
    pub(crate) allowed_mentions: Option<serenity::CreateAllowedMentions<'a>>,
    poll: Option<serenity::CreatePoll<'a, serenity::builder::create_poll::Ready>>,
    reply: bool,
    flags: Option<serenity::MessageFlags>,
}

impl<'a> CreateReply<'a> {
    /// Creates a blank CreateReply. Equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the content of the message.
    pub fn content(mut self, content: impl Into<Cow<'a, str>>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Adds an embed to the message.
    ///
    /// Existing embeds are kept.
    pub fn embed(mut self, embed: serenity::CreateEmbed<'a>) -> Self {
        self.embeds.push(embed);
        self
    }

    /// Sets the flags for the message.
    pub fn flags(mut self, flags: serenity::MessageFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    #[cfg(feature = "unstable")]
    pub fn components(
        mut self,
        components: impl Into<Cow<'a, [serenity::CreateComponent<'a>]>>,
    ) -> Self {
        self.components = Some(components.into());
        self
    }

    /// Set components (buttons and select menus) for this message.
    ///
    /// Any previously set components will be overwritten.
    #[cfg(not(feature = "unstable"))]
    pub fn components(
        mut self,
        components: impl Into<Cow<'a, [serenity::CreateActionRow<'a>]>>,
    ) -> Self {
        self.components = Some(components.into());
        self
    }

    /// Add an attachment.
    pub fn attachment(mut self, attachment: serenity::CreateAttachment<'a>) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Toggles whether the message is an ephemeral response (only invoking user can see it).
    ///
    /// This only has an effect in slash commands!
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = Some(ephemeral);
        self
    }

    /// Set the allowed mentions for the message.
    ///
    /// See [`serenity::CreateAllowedMentions`] for more information.
    pub fn allowed_mentions(
        mut self,
        allowed_mentions: serenity::CreateAllowedMentions<'a>,
    ) -> Self {
        self.allowed_mentions = Some(allowed_mentions);
        self
    }

    /// Adds a poll to the message. Only one poll can be added per message.
    ///
    /// See [`serenity::CreatePoll`] for more information on creating and configuring a poll.
    pub fn poll(
        mut self,
        poll: serenity::CreatePoll<'a, serenity::builder::create_poll::Ready>,
    ) -> Self {
        self.poll = Some(poll);
        self
    }

    /// Makes this message an inline reply to another message like [`serenity::Message::reply`]
    /// (prefix-only, because slash commands are always inline replies anyways).
    ///
    /// To disable the ping, set [`Self::allowed_mentions`] with
    /// [`serenity::CreateAllowedMentions::replied_user`] set to false.
    pub fn reply(mut self, reply: bool) -> Self {
        self.reply = reply;
        self
    }
}

/// Methods to create a message builder from any type from this [`CreateReply`]. Used by lumi
/// internally to actually send a response to Discord
impl<'a> CreateReply<'a> {
    /// Serialize this response builder to a [`serenity::CreateInteractionResponseMessage`]
    pub fn to_slash_initial_response(
        self,
        mut builder: serenity::CreateInteractionResponseMessage<'a>,
    ) -> serenity::CreateInteractionResponseMessage<'a> {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            poll,
            flags,
            reply: _, // can't reply to a message in interactions
        } = self;

        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }
        if let Some(ephemeral) = ephemeral {
            builder = builder.ephemeral(ephemeral);
        }
        if let Some(poll) = poll {
            builder = builder.poll(poll);
        }
        if let Some(flags) = flags {
            builder = builder.flags(flags);
        }

        builder.add_files(attachments).embeds(embeds)
    }

    /// Serialize this response builder to a [`serenity::CreateInteractionResponseFollowup`]
    pub fn to_slash_followup_response(
        self,
        mut builder: serenity::CreateInteractionResponseFollowup<'a>,
    ) -> serenity::CreateInteractionResponseFollowup<'a> {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral,
            allowed_mentions,
            poll,
            flags,
            reply: _,
        } = self;

        if let Some(content) = content {
            builder = builder.content(content);
        }
        builder = builder.embeds(embeds);
        if let Some(components) = components {
            builder = builder.components(components)
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(ephemeral) = ephemeral {
            builder = builder.ephemeral(ephemeral);
        }
        if let Some(poll) = poll {
            builder = builder.poll(poll);
        }
        if let Some(flags) = flags {
            builder = builder.flags(flags);
        }

        builder.add_files(attachments)
    }

    /// Serialize this response builder to a [`serenity::EditInteractionResponse`]
    pub fn to_slash_initial_response_edit(
        self,
        mut builder: serenity::EditInteractionResponse<'a>,
    ) -> serenity::EditInteractionResponse<'a> {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // can't edit ephemerality in retrospect
            allowed_mentions,
            // cannot edit polls.
            poll: _,
            reply: _,
            flags: _,
        } = self;

        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        for attachment in attachments {
            builder = builder.new_attachment(attachment);
        }

        builder.embeds(embeds)
    }

    /// Serialize this response builder to a [`serenity::EditMessage`]
    pub fn to_prefix_edit(
        self,
        mut builder: serenity::EditMessage<'a>,
    ) -> serenity::EditMessage<'a> {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            // cannot edit polls.
            poll: _,
            reply: _, // can't edit reference message afterwards
            flags,
        } = self;

        let mut attachments_builder = serenity::EditAttachments::new();
        for attachment in attachments {
            attachments_builder = attachments_builder.add(attachment);
        }

        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }

        if let Some(flags) = flags {
            builder = builder.flags(flags);
        }

        builder.embeds(embeds).attachments(attachments_builder)
    }

    /// Serialize this response builder to a [`serenity::CreateMessage`]
    pub fn to_prefix(
        self,
        invocation_message: serenity::MessageReference,
    ) -> serenity::CreateMessage<'a> {
        let crate::CreateReply {
            content,
            embeds,
            attachments,
            components,
            ephemeral: _, // not supported in prefix
            allowed_mentions,
            poll,
            reply,
            flags,
        } = self;

        let mut builder = serenity::CreateMessage::new();
        if let Some(content) = content {
            builder = builder.content(content);
        }
        if let Some(allowed_mentions) = allowed_mentions {
            builder = builder.allowed_mentions(allowed_mentions);
        }
        if let Some(components) = components {
            builder = builder.components(components);
        }
        if reply {
            builder = builder.reference_message(invocation_message);
        }
        if let Some(poll) = poll {
            builder = builder.poll(poll);
        }

        if let Some(flags) = flags {
            builder = builder.flags(flags);
        }

        for attachment in attachments {
            builder = builder.add_file(attachment);
        }

        builder.embeds(embeds)
    }
}
