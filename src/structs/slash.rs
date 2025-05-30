//! Holds application command definition structs.

use std::{borrow::Cow, collections::HashMap};

use crate::{BoxFuture, serenity_prelude as serenity};

use super::{CowStr, CowVec};

/// Specifies if the current invokation is from a Command or Autocomplete.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandInteractionType {
    /// Invoked from an application command
    Command,
    /// Invoked from an autocomplete interaction
    Autocomplete,
}

/// Application command specific context passed to command invocations.
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub struct ApplicationContext<'a, T, E> {
    /// The interaction which triggered this command execution.
    pub interaction: &'a serenity::CommandInteraction,
    /// The type of the interaction which triggered this command execution.
    pub interaction_type: CommandInteractionType,
    /// Slash command arguments
    ///
    /// **Not** equivalent to `self.interaction.data().options`. That one refers to just the
    /// top-level command arguments, whereas [`Self::args`] is the options of the actual
    /// subcommand, if any.
    pub args: &'a [serenity::ResolvedOption<'a>],
    /// Keeps track of whether an initial response has been sent.
    ///
    /// Discord requires different HTTP endpoints for initial and additional responses.
    pub has_sent_initial_response: &'a std::sync::atomic::AtomicBool,
    /// Read-only reference to the framework
    ///
    /// Useful if you need the list of commands, for example for a custom help command
    #[derivative(Debug = "ignore")]
    pub framework: crate::FrameworkContext<'a, T, E>,
    /// If the invoked command was a subcommand, these are the parent commands, ordered top down.
    pub parent_commands: &'a [&'a crate::Command<T, E>],
    /// The command object which is the current command
    pub command: &'a crate::Command<T, E>,
    /// Custom user data carried across a single command invocation
    pub invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    // #[non_exhaustive] forbids struct update syntax for ?? reason
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl<T, E> Clone for ApplicationContext<'_, T, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, E> Copy for ApplicationContext<'_, T, E> {}
impl<T, E> crate::_GetGenerics for ApplicationContext<'_, T, E> {
    type T = T;
    type E = E;
}

impl<T, E> ApplicationContext<'_, T, E> {
    /// See [`crate::Context::defer()`]
    pub async fn defer_response(&self, ephemeral: bool) -> Result<(), serenity::Error> {
        if !self
            .has_sent_initial_response
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            let response = serenity::CreateInteractionResponse::Defer(
                serenity::CreateInteractionResponseMessage::new().ephemeral(ephemeral),
            );

            let http = &self.framework.serenity_context.http;
            self.interaction.create_response(http, response).await?;

            self.has_sent_initial_response
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
}

/// Possible actions that a context menu entry can have
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub enum ContextMenuCommandAction<T, E> {
    /// Context menu entry on a user
    User(
        #[derivative(Debug = "ignore")]
        fn(
            ApplicationContext<'_, T, E>,
            serenity::User,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, T, E>>>,
    ),
    /// Context menu entry on a message
    Message(
        #[derivative(Debug = "ignore")]
        fn(
            ApplicationContext<'_, T, E>,
            serenity::Message,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, T, E>>>,
    ),
    #[doc(hidden)]
    __NonExhaustive,
}
impl<T, E> Copy for ContextMenuCommandAction<T, E> {}
impl<T, E> Clone for ContextMenuCommandAction<T, E> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A single drop-down choice in a slash command choice parameter
#[derive(Debug, Clone)]
pub struct CommandParameterChoice {
    /// Label of this choice
    pub name: CowStr,
    /// Localized labels with locale string as the key (slash-only)
    pub localizations: CowVec<(CowStr, CowStr)>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

/// A single parameter of a [`crate::Command`]
#[derive(Clone, derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub struct CommandParameter<T, E> {
    /// Name of this command parameter
    pub name: CowStr,
    /// Localized names with locale string as the key (slash-only)
    pub name_localizations: CowVec<(CowStr, CowStr)>,
    /// Description of the command. Required for slash commands
    pub description: Option<CowStr>,
    /// Localized descriptions with locale string as the key (slash-only)
    pub description_localizations: CowVec<(CowStr, CowStr)>,
    /// `true` is this parameter is required, `false` if it's optional or variadic
    pub required: bool,
    /// If this parameter is a channel, users can only enter these channel types in a slash command
    ///
    /// Prefix commands are currently unaffected by this
    pub channel_types: Option<CowVec<serenity::ChannelType>>,
    /// If this parameter is a choice parameter, this is the fixed list of options
    pub choices: CowVec<CommandParameterChoice>,
    /// Closure that sets this parameter's type and min/max value in the given builder
    ///
    /// For example a u32 [`CommandParameter`] would store this as the [`Self::type_setter`]:
    /// ```rust
    /// # use lumi::serenity_prelude as serenity;
    /// # let _: fn(serenity::CreateCommandOption) -> serenity::CreateCommandOption =
    /// |b| b.kind(serenity::CommandOptionType::Integer).min_int_value(0).max_int_value(i64::MAX)
    /// # ;
    /// ```
    #[derivative(Debug = "ignore")]
    pub type_setter:
        Option<fn(serenity::CreateCommandOption<'_>) -> serenity::CreateCommandOption<'_>>,
    /// Optionally, a callback that is invoked on autocomplete interactions. This closure should
    /// extract the partial argument from the given JSON value and generate the autocomplete
    /// response which contains the list of autocomplete suggestions.
    #[derivative(Debug = "ignore")]
    pub autocomplete_callback: Option<
        for<'a> fn(
            crate::ApplicationContext<'a, T, E>,
            &'a str,
        ) -> BoxFuture<'a, serenity::CreateAutocompleteResponse<'a>>,
    >,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl<T, E> CommandParameter<T, E> {
    /// Generates a slash command parameter builder from this [`CommandParameter`] instance. This
    /// can be used to register the command on Discord's servers
    pub fn create_as_slash_command_option(&self) -> Option<serenity::CreateCommandOption<'static>> {
        let description = self
            .description
            .clone()
            .unwrap_or(Cow::Borrowed("A slash command parameter"));

        let mut builder = serenity::CreateCommandOption::<'static>::new(
            serenity::CommandOptionType::String,
            self.name.clone(),
            description,
        );

        builder = builder
            .required(self.required)
            .set_autocomplete(self.autocomplete_callback.is_some());

        for (locale, name) in self.name_localizations.iter() {
            builder = builder.name_localized(locale.clone(), name.clone());
        }
        for (locale, description) in self.description_localizations.iter() {
            builder = builder.description_localized(locale.clone(), description.clone());
        }
        if let Some(channel_types) = self.channel_types.as_deref() {
            builder = builder.channel_types(channel_types.to_owned());
        }
        for (i, choice) in self.choices.iter().enumerate() {
            builder = builder.add_int_choice_localized(
                choice.name.clone(),
                i as _,
                choice
                    .localizations
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>(),
            );
        }

        Some((self.type_setter?)(builder))
    }
}
