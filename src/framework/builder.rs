//! A builder struct that allows easy and readable creation of a [`crate::Framework`]

/// A builder to configure a framework.
///
/// If [`Self::options`] is missing, the builder will panic on start.
pub struct FrameworkBuilder<T, E> {
    /// Framework options
    options: Option<crate::FrameworkOptions<T, E>>,
    /// List of framework commands
    commands: Vec<crate::Command<T, E>>,
    /// See [`Self::initialize_owners()`]
    initialize_owners: bool,
}

impl<T, E> Default for FrameworkBuilder<T, E> {
    fn default() -> Self {
        Self {
            options: Default::default(),
            commands: Default::default(),
            initialize_owners: true,
        }
    }
}

impl<T, E> FrameworkBuilder<T, E> {
    /// Configure framework options
    #[must_use]
    pub fn options(mut self, options: crate::FrameworkOptions<T, E>) -> Self {
        self.options = Some(options);
        self
    }

    /// Whether to add this bot application's owner and team members to
    /// [`crate::FrameworkOptions::owners`] automatically
    ///
    /// `true` by default
    pub fn initialize_owners(mut self, initialize_owners: bool) -> Self {
        self.initialize_owners = initialize_owners;
        self
    }

    /// Build the framework with the specified configuration.
    ///
    /// For more information, see [`FrameworkBuilder`]
    pub fn build(self) -> crate::Framework<T, E>
    where
        T: Send + Sync + 'static + 'static,
        E: Send + 'static,
    {
        let mut options = self.options.expect("No framework options provided");

        // Build framework options by concatenating user-set options with commands and owners
        options.commands.extend(self.commands);
        options.initialize_owners = self.initialize_owners;

        // Create framework with specified settings
        crate::Framework::new(options)
    }
}
