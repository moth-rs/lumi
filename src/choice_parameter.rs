//! Contains the [`ChoiceParameter`] trait and the blanket [`crate::SlashArgument`] and
//! [`crate::PopArgument`] impl

use crate::{serenity_prelude as serenity, CowVec, PopArgumentResult};

/// This trait is implemented by [`crate::macros::ChoiceParameter`]. See its docs for more
/// information
pub trait ChoiceParameter: Sized {
    /// Returns all possible choices for this parameter, in the order they will appear in Discord.
    fn list() -> CowVec<crate::CommandParameterChoice>;

    /// Returns an instance of [`Self`] corresponding to the given index into [`Self::list()`]
    fn from_index(index: usize) -> Option<Self>;

    /// Parses the name as returned by [`Self::name()`] into an instance of [`Self`]
    fn from_name(name: &str) -> Option<Self>;

    /// Returns the non-localized name of this choice
    fn name(&self) -> &'static str;

    /// Returns the localized name for the given locale, if one is set
    fn localized_name(&self, locale: &str) -> Option<&'static str>;
}

#[async_trait::async_trait]
impl<T: ChoiceParameter> crate::SlashArgument for T {
    async fn extract(
        _: &serenity::Context,
        _: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> ::std::result::Result<Self, crate::SlashArgError> {
        let choice_key = match value {
            serenity::ResolvedValue::Integer(int) => *int as u64,
            _ => {
                return Err(crate::SlashArgError::CommandStructureMismatch {
                    description: "expected u64",
                })
            }
        };

        Self::from_index(choice_key as _).ok_or(crate::SlashArgError::CommandStructureMismatch {
            description: "out of bounds choice key",
        })
    }

    fn create(builder: serenity::CreateCommandOption<'_>) -> serenity::CreateCommandOption<'_> {
        builder.kind(serenity::CommandOptionType::Integer)
    }

    fn choices() -> CowVec<crate::CommandParameterChoice> {
        Self::list()
    }
}

#[async_trait::async_trait]
impl<'a, T: ChoiceParameter> crate::PopArgument<'a> for T {
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> PopArgumentResult<'a, Self> {
        let (args, attachment_index, s) =
            <String as crate::PopArgument<'a>>::pop_from(args, attachment_index, ctx, msg).await?;

        Ok((
            args,
            attachment_index,
            Self::from_name(&s).ok_or((
                Box::new(crate::InvalidChoice {
                    __non_exhaustive: (),
                }) as Box<dyn std::error::Error + Send + Sync>,
                Some(s),
            ))?,
        ))
    }
}
