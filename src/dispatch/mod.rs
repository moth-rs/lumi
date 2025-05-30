//! Contains all code to dispatch incoming events onto framework commands

mod common;
mod permissions;
mod prefix;
mod slash;

pub use common::*;
pub use prefix::*;
pub use slash::*;

use crate::serenity_prelude as serenity;

/// A view into data stored by [`crate::Framework`]
pub struct FrameworkContext<'a, T, E> {
    /// Serenity's context
    pub serenity_context: &'a serenity::Context,
    /// Framework configuration
    pub options: &'a crate::FrameworkOptions<T, E>,
    // deliberately not non exhaustive because you need to create FrameworkContext from scratch
    // to run your own event loop
}
impl<T, E> Copy for FrameworkContext<'_, T, E> {}
impl<T, E> Clone for FrameworkContext<'_, T, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T: Send + Sync + 'static, E> FrameworkContext<'a, T, E> {
    /// Returns the stored framework options, including commands.
    ///
    /// This function exists for API compatiblity with [`crate::Framework`]. On this type, you can
    /// also just access the public `options` field.
    pub fn options(&self) -> &'a crate::FrameworkOptions<T, E> {
        self.options
    }

    /// Retrieves user data
    pub fn user_data(&self) -> std::sync::Arc<T> {
        self.serenity_context.data::<T>()
    }
}

/// Central event handling function of this library
pub async fn dispatch_event<T: Send + Sync + 'static, E>(
    framework: crate::FrameworkContext<'_, T, E>,
    event: &serenity::FullEvent,
) {
    match event {
        serenity::FullEvent::Message { new_message, .. } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            let mut parent_commands = Vec::new();
            let trigger = crate::MessageDispatchTrigger::MessageCreate;
            if let Err(error) = prefix::dispatch_message(
                framework,
                new_message,
                trigger,
                &invocation_data,
                &mut parent_commands,
            )
            .await
            {
                error.handle(framework.options).await;
            }
        }
        serenity::FullEvent::MessageUpdate { event, .. } => {
            if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
                let result = edit_tracker.write().unwrap().process_message_update(
                    event,
                    framework
                        .options()
                        .prefix_options
                        .ignore_edits_if_not_yet_responded,
                );

                if let Some(previously_tracked) = result {
                    let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
                    let mut parent_commands = Vec::new();
                    let trigger = match previously_tracked {
                        true => crate::MessageDispatchTrigger::MessageEdit,
                        false => crate::MessageDispatchTrigger::MessageEditFromInvalid,
                    };
                    if let Err(error) = prefix::dispatch_message(
                        framework,
                        &event.message,
                        trigger,
                        &invocation_data,
                        &mut parent_commands,
                    )
                    .await
                    {
                        error.handle(framework.options).await;
                    }
                }
            }
        }
        serenity::FullEvent::MessageDelete {
            deleted_message_id, ..
        } => {
            if let Some(edit_tracker) = &framework.options.prefix_options.edit_tracker {
                let bot_response = edit_tracker
                    .write()
                    .unwrap()
                    .process_message_delete(*deleted_message_id);
                if let Some(bot_response) = bot_response {
                    if let Err(e) = bot_response
                        .delete(&framework.serenity_context.http, None)
                        .await
                    {
                        tracing::warn!("failed to delete bot response: {}", e);
                    }
                }
            }
        }
        serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Command(interaction),
            ..
        } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            let mut parent_commands = Vec::new();
            if let Err(error) = slash::dispatch_interaction(
                framework,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
                &invocation_data,
                &interaction.data.options(),
                &mut parent_commands,
            )
            .await
            {
                error.handle(framework.options).await;
            }
        }
        serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Autocomplete(interaction),
            ..
        } => {
            let invocation_data = tokio::sync::Mutex::new(Box::new(()) as _);
            let mut parent_commands = Vec::new();
            if let Err(error) = slash::dispatch_autocomplete(
                framework,
                interaction,
                &std::sync::atomic::AtomicBool::new(false),
                &invocation_data,
                &interaction.data.options(),
                &mut parent_commands,
            )
            .await
            {
                error.handle(framework.options).await;
            }
        }
        _ => {}
    }
}
