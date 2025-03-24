//! Contains a simple trait, implemented for all context menu command compatible parameter types
use crate::BoxFuture;
use crate::serenity_prelude as serenity;

/// Implemented for all types that can be used in a context menu command
pub trait ContextMenuParameter<T, E> {
    /// Convert an action function pointer that takes Self as an argument into the appropriate
    /// [`crate::ContextMenuCommandAction`] variant.
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, T, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, T, E>>>,
    ) -> crate::ContextMenuCommandAction<T, E>;
}

impl<T, E> ContextMenuParameter<T, E> for serenity::User {
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, T, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, T, E>>>,
    ) -> crate::ContextMenuCommandAction<T, E> {
        crate::ContextMenuCommandAction::User(action)
    }
}

impl<T, E> ContextMenuParameter<T, E> for serenity::Message {
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, T, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, T, E>>>,
    ) -> crate::ContextMenuCommandAction<T, E> {
        crate::ContextMenuCommandAction::Message(action)
    }
}
