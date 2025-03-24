//! Utilities for registering application commands

use crate::serenity_prelude::{self as serenity, CollectComponentInteractions};

/// Collects all commands into a [`Vec<serenity::CreateCommand>`] builder, which can be used
/// to register the commands on Discord
///
/// Also see [`register_application_commands_buttons`] for a ready to use register command
///
/// ```rust,no_run
/// # use lumi::serenity_prelude as serenity;
/// # async fn foo(ctx: lumi::Context<'_, (), ()>) -> Result<(), serenity::Error> {
/// let commands = &ctx.framework().options().commands;
/// let create_commands = lumi::builtins::create_application_commands(commands);
///
/// serenity::Command::set_global_commands(ctx.http(), &create_commands).await?;
/// # Ok(()) }
/// ```
pub fn create_application_commands<T, E>(
    commands: &[crate::Command<T, E>],
) -> Vec<serenity::CreateCommand<'static>> {
    /// We decided to extract context menu commands recursively, despite the subcommand hierarchy
    /// not being preserved. Because it's more confusing to just silently discard context menu
    /// commands if they're not top-level commands.
    /// https://discord.com/channels/381880193251409931/919310428344029265/947970605985189989
    fn recursively_add_context_menu_commands<T, E>(
        builder: &mut Vec<serenity::CreateCommand<'static>>,
        command: &crate::Command<T, E>,
    ) {
        if let Some(context_menu_command) = command.create_as_context_menu_command() {
            builder.push(context_menu_command);
        }
        for subcommand in &command.subcommands {
            recursively_add_context_menu_commands(builder, subcommand);
        }
    }

    let mut commands_builder = Vec::with_capacity(commands.len());
    for command in commands {
        if let Some(slash_command) = command.create_as_slash_command() {
            commands_builder.push(slash_command);
        }
        recursively_add_context_menu_commands(&mut commands_builder, command);
    }
    commands_builder
}

/// Registers the given list of application commands to Discord as global commands.
///
/// Thin wrapper around [`create_application_commands`] that funnels the returned builder into
/// [`serenity::Command::set_global_commands`].
pub async fn register_globally<T, E>(
    http: &serenity::Http,
    commands: &[crate::Command<T, E>],
) -> Result<(), serenity::Error> {
    let builder = create_application_commands(commands);
    serenity::Command::set_global_commands(http, &builder).await?;
    Ok(())
}

/// Registers the given list of application commands to Discord as guild-specific commands.
///
/// Thin wrapper around [`create_application_commands`] that funnels the returned builder into
/// [`serenity::GuildId::set_commands`].
pub async fn register_in_guild<T, E>(
    http: &serenity::Http,
    commands: &[crate::Command<T, E>],
    guild_id: serenity::GuildId,
) -> Result<(), serenity::Error> {
    let builder = create_application_commands(commands);
    guild_id.set_commands(http, &builder).await?;
    Ok(())
}

/// _Note: you probably want [`register_application_commands_buttons`] instead; it's easier and more
/// powerful_
///
/// Wraps [`create_application_commands`] and adds a bot owner permission check and status messages.
///
/// This function is supposed to be a ready-to-use implementation for a `~register` command of your
/// bot. So if you want, you can copy paste this help message for the command:
///
/// ```text
/// Registers application commands in this guild or globally
///
/// Run with no arguments to register in guild, run with argument "global" to register globally.
/// ```
pub async fn register_application_commands<T: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, T, E>,
    global: bool,
) -> Result<(), serenity::Error> {
    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    let commands_builder = create_application_commands(&ctx.framework().options().commands);
    let num_commands = commands_builder.len();

    if global {
        ctx.say(format!("Registering {num_commands} commands...",))
            .await?;
        serenity::Command::set_global_commands(ctx.http(), &commands_builder).await?;
    } else {
        let guild_id = match ctx.guild_id() {
            Some(x) => x,
            None => {
                ctx.say("Must be called in guild").await?;
                return Ok(());
            }
        };

        ctx.say(format!("Registering {num_commands} commands..."))
            .await?;
        guild_id.set_commands(ctx.http(), &commands_builder).await?;
    }

    ctx.say("Done!").await?;

    Ok(())
}

/// Spawns four buttons to register or delete application commands globally or in the current guild
///
/// Upgraded version of [`register_application_commands`]
///
/// ![Screenshot of output](https://imgur.com/rTbTaDs.png)
///
/// You probably want to use this by wrapping it in a small `register` command:
/// ```rust
/// # type Error = Box<dyn std::error::Error + Send + Sync>;
/// # type Context<'a> = lumi::Context<'a, (), Error>;
/// #[lumi::command(prefix_command)]
/// pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
///     lumi::builtins::register_application_commands_buttons(ctx).await?;
///     Ok(())
/// }
///
/// // ...
/// lumi::FrameworkOptions {
///     commands: vec![
///         // ...
///         register(),
///     ],
/// #   ..Default::default()
/// };
/// ```
///
/// Which you can call like any prefix command, for example `@your_bot register`.
pub async fn register_application_commands_buttons<T: Send + Sync + 'static, E>(
    ctx: crate::Context<'_, T, E>,
) -> Result<(), serenity::Error> {
    let create_commands = create_application_commands(&ctx.framework().options().commands);
    let num_commands = create_commands.len();

    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    let buttons = [
        serenity::CreateButton::new("register.guild")
            .label("Register in guild")
            .style(serenity::ButtonStyle::Primary)
            .emoji('📋'),
        serenity::CreateButton::new("unregister.guild")
            .label("Delete in guild")
            .style(serenity::ButtonStyle::Danger)
            .emoji('🗑'),
        serenity::CreateButton::new("register.global")
            .label("Register globally")
            .style(serenity::ButtonStyle::Primary)
            .emoji('📋'),
        serenity::CreateButton::new("unregister.global")
            .label("Unregister globally")
            .style(serenity::ButtonStyle::Danger)
            .emoji('🗑'),
    ];

    let components = [serenity::CreateActionRow::buttons(&buttons)];
    let builder = crate::CreateReply::default()
        .content("Choose what to do with the commands:")
        .components(&components);

    let reply = ctx.send(builder).await?;
    let reply_message = reply.message().await?;

    let interaction = reply_message
        .id
        .collect_component_interactions(ctx.serenity_context())
        .author_id(ctx.author().id)
        .await;

    reply
        .edit(
            ctx,
            crate::CreateReply::default()
                .components(vec![])
                .content("Processing... Please wait."),
        )
        .await?; // remove buttons after button press and edit message
    let pressed_button_id = match &interaction {
        Some(m) => &m.data.custom_id,
        None => {
            ctx.say(":warning: You didn't interact in time - please run the command again.")
                .await?;
            return Ok(());
        }
    };

    let (register, global) = match &**pressed_button_id {
        "register.global" => (true, true),
        "unregister.global" => (false, true),
        "register.guild" => (true, false),
        "unregister.guild" => (false, false),
        other => {
            tracing::warn!("unknown register button ID: {:?}", other);
            return Ok(());
        }
    };

    let start_time = std::time::Instant::now();

    if global {
        if register {
            ctx.say(format!(
                ":gear: Registering {num_commands} global commands...",
            ))
            .await?;
            serenity::Command::set_global_commands(ctx.http(), &create_commands).await?;
        } else {
            ctx.say(":gear: Unregistering global commands...").await?;
            serenity::Command::set_global_commands(ctx.http(), &[]).await?;
        }
    } else {
        let guild_id = match ctx.guild_id() {
            Some(x) => x,
            None => {
                ctx.say(":x: Must be called in guild").await?;
                return Ok(());
            }
        };
        if register {
            ctx.say(format!(
                ":gear: Registering {num_commands} guild commands...",
            ))
            .await?;
            guild_id.set_commands(ctx.http(), &create_commands).await?;
        } else {
            ctx.say(":gear: Unregistering guild commands...").await?;
            guild_id.set_commands(ctx.http(), &[]).await?;
        }
    }

    // Calulate time taken and send message
    let time_taken = start_time.elapsed();
    ctx.say(format!(
        ":white_check_mark: Done! Took {}ms",
        time_taken.as_millis()
    ))
    .await?;

    Ok(())
}
