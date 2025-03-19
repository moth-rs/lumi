pub enum Context<'_, T, E> {
    Application(ApplicationContext<'_, T, E>),
}

/// Specifies if the current invokation is from a Command or Autocomplete.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandInteractionType {
    /// Invoked from an application command.
    Command,
    /// Invoked from an autocomplete interaction, from command arguments.
    Autocomplete,
}

pub struct ApplicationContext<'a, T, E> {
    /// Serenity's context, giving you access to HTTP and the cache.
    pub serenity_context: &'a serenity::all::Context,
    /// The interaction object that triggered this command execution.
    pub interaction: &'a serenity::all::Interaction,
    /// The type of the interaction which triggered this command execution.
    pub interaction_type: CommandInteractionType,
    /// A check for if an initial response has been sent as Discord requires a different endpoint
    /// for followup responses.
    pub initial_response_sent: &'a std::sync::atomic::AtomicBool,
}
