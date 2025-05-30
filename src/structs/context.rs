//! Just contains Context and `PartialContext` structs

use std::borrow::Cow;

use crate::{CommandInteractionType, serenity_prelude as serenity};

// needed for proc macro
#[doc(hidden)]
pub trait _GetGenerics {
    type T;
    type E;
}
impl<T, E> _GetGenerics for Context<'_, T, E> {
    type T = T;
    type E = E;
}

/// Wrapper around either [`crate::ApplicationContext`] or [`crate::PrefixContext`]
#[derive(Debug)]
pub enum Context<'a, T, E> {
    /// Application command context
    Application(crate::ApplicationContext<'a, T, E>),
    /// Prefix command context
    Prefix(crate::PrefixContext<'a, T, E>),
    // Not non_exhaustive.. adding a whole new category of commands would justify breakage lol
}
impl<T, E> Clone for Context<'_, T, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, E> Copy for Context<'_, T, E> {}
impl<'a, T, E> From<crate::ApplicationContext<'a, T, E>> for Context<'a, T, E> {
    fn from(x: crate::ApplicationContext<'a, T, E>) -> Self {
        Self::Application(x)
    }
}
impl<'a, T, E> From<crate::PrefixContext<'a, T, E>> for Context<'a, T, E> {
    fn from(x: crate::PrefixContext<'a, T, E>) -> Self {
        Self::Prefix(x)
    }
}
/// Macro to generate Context methods and also PrefixContext and ApplicationContext methods that
/// delegate to Context
macro_rules! context_methods {
    ( $(
        $( #[$($attrs:tt)*] )*
        // pub $(async $($dummy:block)?)? fn $fn_name:ident $()
        // $fn_name:ident ($($sig:tt)*) $body:block
        $($await:ident)? ( $fn_name:ident $self:ident $($arg:ident)* )
        ( $($sig:tt)* ) $(where $b1:lifetime : $b2:lifetime)? $body:block
    )* ) => {
        impl<'a, T: Send + Sync + 'static, E> Context<'a, T, E> { $(
            $( #[$($attrs)*] )*
            $($sig)* $(where $b1:$b2)* $body
        )* }

        impl<'a, T: Send + Sync + 'static, E> crate::PrefixContext<'a, T, E> { $(
            $( #[$($attrs)*] )*
            $($sig)* $(where $b1:$b2)* {
                $crate::Context::Prefix($self).$fn_name($($arg)*) $(.$await)?
            }
        )* }

        impl<'a, T: Send + Sync + 'static, E> crate::ApplicationContext<'a, T, E> { $(
            $( #[$($attrs)*] )*
            $($sig)* $(where $b1:$b2)* {
                $crate::Context::Application($self).$fn_name($($arg)*) $(.$await)?
            }
        )* }
    };
}
// Note how you have to surround the function signature in parentheses, and also add a line before
// the signature with the function name, parameter names and maybe `await` token
context_methods! {
    /// Defer the response, giving the bot multiple minutes to respond without the user seeing an
    /// "interaction failed error".
    ///
    /// Also sets the [`crate::ApplicationContext::has_sent_initial_response`] flag so subsequent
    /// responses will be sent in the correct manner.
    ///
    /// No-op if this is an autocomplete context
    ///
    /// This will make the response public; to make it ephemeral, use [`Self::defer_ephemeral()`].
    await (defer self)
    (pub async fn defer(self) -> Result<(), serenity::Error>) {
        if let Self::Application(ctx) = self {
            ctx.defer_response(false).await?;
        }
        Ok(())
    }

    /// See [`Self::defer()`]
    ///
    /// This will make the response ephemeral; to make it public, use [`Self::defer()`].
    await (defer_ephemeral self)
    (pub async fn defer_ephemeral(self) -> Result<(), serenity::Error>) {
        if let Self::Application(ctx) = self {
            ctx.defer_response(true).await?;
        }
        Ok(())
    }

    /// If this is an application command, [`Self::defer()`] is called
    ///
    /// If this is a prefix command, a typing broadcast is started until the return value is
    /// dropped.
    // #[must_use = "The typing broadcast will only persist if you store it"] // currently doesn't work
    await (defer_or_broadcast self)
    (pub async fn defer_or_broadcast(self) -> Result<Option<serenity::Typing>, serenity::Error>) {
        Ok(match self {
            Self::Application(ctx) => {
                ctx.defer_response(false).await?;
                None
            }
            Self::Prefix(ctx) => Some(
                ctx.msg
                    .channel_id
                    .start_typing(ctx.serenity_context().http.clone()),
            ),
        })
    }

    /// Shorthand of [`crate::say_reply`]
    ///
    /// Note: panics when called in an autocomplete context!
    await (say self text)
    (pub async fn say<'arg>(self, text: impl Into<Cow<'arg, str>>) -> Result<crate::ReplyHandle<'a>, serenity::Error>) {
        crate::say_reply(self, text).await
    }

    /// Like [`Self::say`], but formats the message as a reply to the user's command
    /// message.
    ///
    /// Equivalent to `.send(CreateReply::default().content("...").reply(true))`.
    ///
    /// Only has an effect in prefix context, because slash command responses are always
    /// formatted as a reply.
    ///
    /// Note: panics when called in an autocomplete context!
    await (reply self text)
    (pub async fn reply(
        self,
        text: impl Into<Cow<'_, str>>,
    ) -> Result<crate::ReplyHandle<'a>, serenity::Error>) {
        self.send(crate::CreateReply::default().content(text).reply(true)).await
    }

    /// Shorthand of [`crate::send_reply`]
    ///
    /// Note: panics when called in an autocomplete context!
    await (send self builder)
    (pub async fn send(
        self,
        builder: crate::CreateReply<'_>,
    ) -> Result<crate::ReplyHandle<'a>, serenity::Error>) {
        crate::send_reply(self, builder).await
    }

    /// Return the stored [`serenity::Context`] within the underlying context type.
    (serenity_context self)
    (pub fn serenity_context(self) -> &'a serenity::Context) {
        self.framework().serenity_context
    }

    /// Create a [`crate::CooldownContext`] based off the underlying context type.
    (cooldown_context self)
    (pub fn cooldown_context(self) -> crate::CooldownContext) {
        crate::CooldownContext {
            user_id: self.author().id,
            channel_id: self.channel_id(),
            guild_id: self.guild_id()
        }
    }

    /// Returns a view into data stored by the framework, like configuration
    (framework self)
    (pub fn framework(self) -> crate::FrameworkContext<'a, T, E>) {
        match self {
            Self::Application(ctx) => ctx.framework,
            Self::Prefix(ctx) => ctx.framework,
        }
    }

    /// Return a reference to your custom user data
    (data self)
    (pub fn data(self) -> std::sync::Arc<T>) {
        self.framework().user_data()
    }

    /// Return the channel ID of this context
    (channel_id self)
    (pub fn channel_id(self) -> serenity::GenericChannelId) {
        match self {
            Self::Application(ctx) => ctx.interaction.channel_id,
            Self::Prefix(ctx) => ctx.msg.channel_id,
        }
    }

    /// Returns the guild ID of this context, if we are inside a guild
    (guild_id self)
    (pub fn guild_id(self) -> Option<serenity::GuildId>) {
        match self {
            Self::Application(ctx) => ctx.interaction.guild_id,
            Self::Prefix(ctx) => ctx.msg.guild_id,
        }
    }

    /// Return the guild channel of this context, if we are inside a guild.
    await (channel self)
    (pub async fn channel(self) -> Option<serenity::Channel>) {
        self.channel_id().to_channel(self.serenity_context(), self.guild_id()).await.ok()
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Return the guild of this context, if we are inside a guild.
    (guild self)
    (pub fn guild(self) -> Option<serenity::GuildRef<'a>>) {
        self.guild_id()?.to_guild_cached(self.cache())
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Return the partial guild of this context, if we are inside a guild.
    ///
    /// Attempts to find the guild in cache, if cache feature is enabled. Otherwise, falls back to
    /// an HTTP request
    ///
    /// Returns None if in DMs, or if the guild HTTP request fails
    await (partial_guild self)
    (pub async fn partial_guild(self) -> Option<serenity::PartialGuild>) {
        if let Some(guild) = self.guild() {
            return Some(guild.clone().into());
        }

        self.guild_id()?.to_partial_guild(self.serenity_context()).await.ok()
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Returns the author of the invoking message or interaction, as a [`serenity::Member`]
    ///
    /// Returns a reference to the inner member object if in an [`crate::ApplicationContext`], otherwise
    /// clones the member out of the cache, or fetches from the discord API.
    ///
    /// Returns None if this command was invoked in DMs, or if the member cache lookup or HTTP
    /// request failed
    ///
    /// Warning: can clone the entire Member instance out of the cache
    await (author_member self)
    (pub async fn author_member(self) -> Option<Cow<'a, serenity::Member>>) {
        if let Self::Application(ctx) = self {
            ctx.interaction.member.as_deref().map(Cow::Borrowed)
        } else {
            self.guild_id()?
                .member(self.serenity_context(), self.author().id)
                .await
                .ok()
                .map(Cow::Owned)
        }
    }

    /// Return the datetime of the invoking message or interaction
    (created_at self)
    (pub fn created_at(self) -> serenity::Timestamp) {
        match self {
            Self::Application(ctx) => ctx.interaction.id.created_at(),
            Self::Prefix(ctx) => ctx.msg.timestamp,
        }
    }

    /// Get the author of the command message or application command.
    (author self)
    (pub fn author(self) -> &'a serenity::User) {
        match self {
            Self::Application(ctx) => &ctx.interaction.user,
            Self::Prefix(ctx) => &ctx.msg.author,
        }
    }

    /// Return a ID that uniquely identifies this command invocation.
    (id self)
    (pub fn id(self) -> u64) {
        match self {
            Self::Application(ctx) => ctx.interaction.id.get(),
            Self::Prefix(ctx) => {
                let mut id = ctx.msg.id.get();
                if let Some(edited_timestamp) = ctx.msg.edited_timestamp {
                    // We replace the 42 datetime bits with msg.timestamp_edited so that the ID is
                    // unique even after edits

                    // Set existing datetime bits to zero
                    id &= !0 >> 42;

                    // Calculate Discord's datetime representation (millis since Discord epoch) and
                    // insert those bits into the ID
                    let timestamp_millis = edited_timestamp.timestamp_millis();

                    id |= ((timestamp_millis - 1420070400000) as u64) << 22;
                }
                id
            }
        }
    }

    /// If the invoked command was a subcommand, these are the parent commands, ordered top-level
    /// downwards.
    (parent_commands self)
    (pub fn parent_commands(self) -> &'a [&'a crate::Command<T, E>]) {
        match self {
            Self::Prefix(x) => x.parent_commands,
            Self::Application(x) => x.parent_commands,
        }
    }

    /// Returns a reference to the command.
    (command self)
    (pub fn command(self) -> &'a crate::Command<T, E>) {
        match self {
            Self::Prefix(x) => x.command,
            Self::Application(x) => x.command,
        }
    }

    /// Returns the prefix this command was invoked with, or a slash (`/`), if this is an
    /// application command.
    (prefix self)
    (pub fn prefix(self) -> &'a str) {
        match self {
            Context::Prefix(ctx) => ctx.prefix,
            Context::Application(_) => "/",
        }
    }

    /// Returns the command name that this command was invoked with
    ///
    /// Mainly useful in prefix context, for example to check whether a command alias was used.
    ///
    /// In slash contexts, the given command name will always be returned verbatim, since there are
    /// no slash command aliases and the user has no control over spelling
    (invoked_command_name self)
    (pub fn invoked_command_name(self) -> &'a str) {
        match self {
            Self::Prefix(ctx) => ctx.invoked_command_name,
            Self::Application(ctx) => &ctx.interaction.data.name,
        }
    }

    /// Re-runs this entire command invocation
    ///
    /// Permission checks are omitted; the command code is directly executed as a function. The
    /// result is returned by this function
    await (rerun self)
    (pub async fn rerun(self) -> Result<(), E>) {
        match self.rerun_inner().await {
            Ok(()) => Ok(()),
            Err(crate::FrameworkError::Command { error, ctx: _ }) => Err(error),
            // The only code that runs before the actual user code (which would trigger Command
            // error) is argument parsing. And that's pretty much deterministic. So, because the
            // current command invocation parsed successfully, we can always expect that a command
            // rerun will still parse successfully.
            // Also: can't debug print error because then we need U: Debug + E: Debug bound arghhhhh
            Err(_other) => panic!("unexpected error before entering command"),
        }
    }

    /// Returns the string with which this command was invoked.
    ///
    /// For example `"/slash_command subcommand arg1:value1 arg2:value2"`.
    (invocation_string self)
    (pub fn invocation_string(self) -> String) {
        match self {
            Context::Application(ctx) => {
                let mut string = String::from("/");
                for parent_command in ctx.parent_commands {
                    string += &parent_command.name;
                    string += " ";
                }
                string += &ctx.command.name;
                for arg in ctx.args {
                    use std::fmt::Write as _;

                    string += " ";
                    string += arg.name;
                    string += ":";

                    let _ = match arg.value {
                        // This was verified to match Discord behavior when copy-pasting a not-yet
                        // sent slash command invocation
                        serenity::ResolvedValue::Attachment(_) => write!(string, ""),
                        serenity::ResolvedValue::Boolean(x) => write!(string, "{}", x),
                        serenity::ResolvedValue::Integer(x) => write!(string, "{}", x),
                        serenity::ResolvedValue::Number(x) => write!(string, "{}", x),
                        serenity::ResolvedValue::String(x) => write!(string, "{}", x),
                        serenity::ResolvedValue::Channel(x) => {
                            write!(string, "#{}", x.base().name.as_deref().unwrap_or(""))
                        }
                        serenity::ResolvedValue::Role(x) => write!(string, "@{}", x.name),
                        serenity::ResolvedValue::User(x, _) => {
                            string.push('@');
                            string.push_str(&x.name);
                            if let Some(discrim) = x.discriminator {
                                let _ = write!(string, "#{discrim:04}");
                            }
                            Ok(())
                        }

                        serenity::ResolvedValue::Unresolved(_)
                        | serenity::ResolvedValue::SubCommand(_)
                        | serenity::ResolvedValue::SubCommandGroup(_)
                        | serenity::ResolvedValue::Autocomplete { .. } => {
                            tracing::warn!("unexpected interaction option type");
                            Ok(())
                        }
                        // We need this because ResolvedValue is #[non_exhaustive]
                        _ => {
                            tracing::warn!("newly-added unknown interaction option type");
                            Ok(())
                        }
                    };
                }
                string
            }
            Context::Prefix(ctx) => ctx.msg.content.to_string(),
        }
    }

    /// Stores the given value as the data for this command invocation
    ///
    /// This data is carried across the `pre_command` hook, checks, main command execution, and
    /// `post_command`. It may be useful to cache data or pass information to later phases of command
    /// execution.
    await (set_invocation_data self data)
    (pub async fn set_invocation_data<U: 'static + Send + Sync>(self, data: U)) {
        *self.invocation_data_raw().lock().await = Box::new(data);
    }

    /// Attempts to get the invocation data with the requested type
    ///
    /// If the stored invocation data has a different type than requested, None is returned
    await (invocation_data self)
    (pub async fn invocation_data<U: 'static>(
        self,
    ) -> Option<impl std::ops::DerefMut<Target = U> + 'a>) {
        tokio::sync::MutexGuard::try_map(self.invocation_data_raw().lock().await, |any| {
            any.downcast_mut()
        })
        .ok()
    }

    /// If available, returns the locale (selected language) of the invoking user
    (locale self)
    (pub fn locale(self) -> Option<&'a str>) {
        match self {
            Context::Application(ctx) => Some(&ctx.interaction.locale),
            Context::Prefix(_) => None,
        }
    }

    /// Builds a [`crate::CreateReply`] by combining the builder closure with the defaults that were
    /// pre-configured in lumi.
    ///
    /// This is primarily an internal function and only exposed for people who want to manually
    /// convert [`crate::CreateReply`] instances into Discord requests.
    #[allow(unused_mut)] // side effect of how macro works
    (reply_builder self builder)
    (pub fn reply_builder<'args>(self, mut builder: crate::CreateReply<'args>) -> crate::CreateReply<'args>) {
        let fw_options = self.framework().options();
        builder.ephemeral = builder.ephemeral.or(Some(self.command().ephemeral));
        builder.allowed_mentions = builder.allowed_mentions.or_else(|| fw_options.allowed_mentions.clone());

        if let Some(callback) = fw_options.reply_callback {
            builder = callback(self, builder);
        }

        builder
    }

    /// Returns serenity's cache which stores various useful data received from the gateway
    ///
    /// Shorthand for [`.serenity_context().cache`](serenity::Context::cache)
    (cache self)
    (pub fn cache(self) -> &'a serenity::Cache) {
        &self.serenity_context().cache
    }

    /// Returns serenity's raw Discord API client to make raw API requests, if needed.
    ///
    /// Shorthand for [`.serenity_context().http`](serenity::Context::http)
    (http self)
    (pub fn http(self) -> &'a serenity::Http) {
        &self.serenity_context().http
    }

    /// Returns the current gateway heartbeat latency ([`::serenity::gateway::Shard::latency()`]).
    ///
    /// If the shard has just connected, this value is zero.
    await (ping self)
    (pub async fn ping(self) -> std::time::Duration) {
        let zero = std::time::Duration::ZERO;
        let Some(runner) = self.serenity_context().runners.get(&self.serenity_context().shard_id) else { return zero };
        runner.0.latency.unwrap_or(zero)
    }
}

impl<'a, T, E> Context<'a, T, E> {
    /// Actual implementation of rerun() that returns `FrameworkError` for implementation convenience
    async fn rerun_inner(self) -> Result<(), crate::FrameworkError<'a, T, E>> {
        match self {
            Self::Application(ctx) => {
                // Skip autocomplete interactions
                if ctx.interaction_type == CommandInteractionType::Autocomplete {
                    return Ok(());
                }

                // Check slash command
                if ctx.interaction.data.kind == serenity::CommandType::ChatInput {
                    return if let Some(action) = ctx.command.slash_action {
                        action(ctx).await
                    } else {
                        Ok(())
                    };
                }

                // Check context menu command
                if let (Some(action), Some(target)) = (
                    ctx.command.context_menu_action,
                    &ctx.interaction.data.target(),
                ) {
                    return match action {
                        crate::ContextMenuCommandAction::User(action) => {
                            if let serenity::ResolvedTarget::User(user, _) = target {
                                action(ctx, (*user).clone()).await
                            } else {
                                Ok(())
                            }
                        }
                        crate::ContextMenuCommandAction::Message(action) => {
                            if let serenity::ResolvedTarget::Message(message) = target {
                                action(ctx, (*message).clone()).await
                            } else {
                                Ok(())
                            }
                        }
                        crate::ContextMenuCommandAction::__NonExhaustive => unreachable!(),
                    };
                }
            }
            Self::Prefix(ctx) => {
                if let Some(action) = ctx.command.prefix_action {
                    return action(ctx).await;
                }
            }
        }

        // Fallback if the Command doesn't have the action it needs to execute this context
        // (This should never happen, because if this context cannot be executed, how could this
        // method have been called)
        Ok(())
    }

    /// Returns the raw type erased invocation data
    fn invocation_data_raw(self) -> &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>> {
        match self {
            Context::Application(ctx) => ctx.invocation_data,
            Context::Prefix(ctx) => ctx.invocation_data,
        }
    }
}

/// Forwards for serenity::Context's impls. With these, lumi's Context types can be passed in as-is
/// to serenity API functions.
macro_rules! context_trait_impls {
    ($($type:tt)*) => {
        impl<T: Send + Sync + 'static, E> AsRef<serenity::Cache> for $($type)*<'_, T, E> {
            fn as_ref(&self) -> &serenity::Cache {
                &self.serenity_context().cache
            }
        }
        impl<T: Send + Sync + 'static, E> AsRef<serenity::Http> for $($type)*<'_, T, E> {
            fn as_ref(&self) -> &serenity::Http {
                &self.serenity_context().http
            }
        }
        // Originally added as part of component interaction modals; not sure if this impl is really
        // required by anything else... It makes sense to have though imo
        impl<T: Send + Sync + 'static, E> AsRef<serenity::Context> for $($type)*<'_, T, E> {
            fn as_ref(&self) -> &serenity::Context {
                self.serenity_context()
            }
        }
        impl<T: Send + Sync + 'static, E> serenity::CacheHttp for $($type)*<'_, T, E> {
            fn http(&self) -> &serenity::Http {
                &self.serenity_context().http
            }

            fn cache(&self) -> Option<&std::sync::Arc<serenity::Cache>> {
                Some(&self.serenity_context().cache)
            }
        }
    };
}
context_trait_impls!(Context);
context_trait_impls!(crate::ApplicationContext);
context_trait_impls!(crate::PrefixContext);

/// Trimmed down, more general version of [`Context`]
pub struct PartialContext<'a, T, E> {
    /// ID of the guild, if not invoked in DMs
    pub guild_id: Option<serenity::GuildId>,
    /// ID of the invocation channel
    pub channel_id: serenity::GenericChannelId,
    /// ID of the invocation author
    pub author: &'a serenity::User,
    /// Useful if you need the list of commands, for example for a custom help command
    pub framework: crate::FrameworkContext<'a, T, E>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl<T, E> Copy for PartialContext<'_, T, E> {}
impl<T, E> Clone for PartialContext<'_, T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: Send + Sync + 'static, E> From<Context<'a, T, E>> for PartialContext<'a, T, E> {
    fn from(ctx: Context<'a, T, E>) -> Self {
        Self {
            guild_id: ctx.guild_id(),
            channel_id: ctx.channel_id(),
            author: ctx.author(),
            framework: ctx.framework(),
            __non_exhaustive: (),
        }
    }
}
