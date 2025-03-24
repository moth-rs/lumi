//! Dispatches incoming messages and message edits onto framework commands

use crate::serenity_prelude as serenity;

/// Checks if this message is a bot invocation by attempting to strip the prefix
///
/// Returns tuple of stripped prefix and rest of the message, if any prefix matches
async fn strip_prefix<'a, T: Send + Sync + 'static, E>(
    framework: crate::FrameworkContext<'a, T, E>,
    msg: &'a serenity::Message,
) -> Option<(&'a str, &'a str)> {
    let partial_ctx = crate::PartialContext {
        guild_id: msg.guild_id,
        channel_id: msg.channel_id,
        author: &msg.author,
        framework,
        __non_exhaustive: (),
    };

    if let Some(dynamic_prefix) = framework.options.prefix_options.dynamic_prefix {
        match dynamic_prefix(partial_ctx).await {
            Ok(prefix) => {
                if let Some(prefix) = prefix {
                    if msg.content.starts_with(prefix.as_ref()) {
                        return Some(msg.content.split_at(prefix.len()));
                    }
                }
            }
            Err(error) => {
                (framework.options.on_error)(crate::FrameworkError::DynamicPrefix {
                    error,
                    ctx: partial_ctx,
                    msg,
                })
                .await;
            }
        }
    }

    if let Some(prefix) = framework.options.prefix_options.prefix.as_deref() {
        if let Some(content) = msg.content.strip_prefix(prefix) {
            return Some((prefix, content));
        }
    }

    if let Some((prefix, content)) = framework
        .options
        .prefix_options
        .additional_prefixes
        .iter()
        .find_map(|prefix| match prefix {
            &crate::Prefix::Literal(prefix) => Some((prefix, msg.content.strip_prefix(prefix)?)),
            crate::Prefix::Regex(prefix) => {
                let regex_match = prefix.find(&msg.content)?;
                if regex_match.start() == 0 {
                    Some(msg.content.split_at(regex_match.end()))
                } else {
                    None
                }
            }
            crate::Prefix::__NonExhaustive => unreachable!(),
        })
    {
        return Some((prefix, content));
    }

    if let Some(dynamic_prefix) = framework.options.prefix_options.stripped_dynamic_prefix {
        match dynamic_prefix(framework.serenity_context, msg, framework.user_data()).await {
            Ok(result) => {
                if let Some((prefix, content)) = result {
                    return Some((prefix, content));
                }
            }
            Err(error) => {
                (framework.options.on_error)(crate::FrameworkError::DynamicPrefix {
                    error,
                    ctx: partial_ctx,
                    msg,
                })
                .await;
            }
        }
    }

    if framework.options.prefix_options.mention_as_prefix {
        // Mentions are either <@USER_ID> or <@!USER_ID>
        if let Some(stripped_content) = (|| {
            msg.content
                .strip_prefix("<@")?
                .trim_start_matches('!')
                .strip_prefix(
                    &framework
                        .serenity_context
                        .cache
                        .current_user()
                        .id
                        .to_string(),
                )?
                .strip_prefix('>')
        })() {
            let mention_prefix =
                &msg.content[..(msg.content.len() as usize - stripped_content.len())];
            return Some((mention_prefix, stripped_content));
        }
    }

    None
}

/// Find a command or subcommand within `&[Command]`, given a command invocation without a prefix.
/// Returns the verbatim command name string as well as the command arguments (i.e. the remaining
/// string).
///
/// The API must be like this (as opposed to just taking the command name upfront) because of
/// subcommands.
pub fn find_command<'a, T, E>(
    commands: &'a [crate::Command<T, E>],
    remaining_message: &'a str,
    case_insensitive: bool,
    parent_commands: &mut Vec<&'a crate::Command<T, E>>,
) -> Option<(&'a crate::Command<T, E>, &'a str, &'a str, &'a str)> {
    let string_equal = if case_insensitive {
        |a: &str, b: &str| a.eq_ignore_ascii_case(b)
    } else {
        |a: &str, b: &str| a == b
    };

    let (command_name, remaining_message) = {
        let mut iter = remaining_message.splitn(2, char::is_whitespace);
        (iter.next().unwrap(), iter.next().unwrap_or("").trim_start())
    };

    for command in commands {
        let (primary_name_matches, alias_matches, mod_chars) =
            if command.has_modifier && command.subcommands.is_empty() {
                let (primary_match, primary_mod) =
                    starts_with(&command.name, command_name, case_insensitive);

                if primary_match {
                    (true, false, primary_mod)
                } else {
                    let alias_match = command.aliases.iter().find_map(|alias| {
                        let (matches, mod_str) = starts_with(alias, command_name, case_insensitive);
                        if matches { Some(mod_str) } else { None }
                    });

                    (false, alias_match.is_some(), alias_match.unwrap_or(""))
                }
            } else {
                let primary_name_matches = string_equal(&command.name, command_name);
                let alias_matches = command
                    .aliases
                    .iter()
                    .any(|alias| string_equal(alias, command_name));

                (primary_name_matches, alias_matches, "")
            };

        if !primary_name_matches && !alias_matches {
            continue;
        }

        parent_commands.push(command);
        return Some(
            find_command(
                &command.subcommands,
                remaining_message,
                case_insensitive,
                parent_commands,
            )
            .unwrap_or_else(|| {
                parent_commands.pop();
                (command, mod_chars, command_name, remaining_message)
            }),
        );
    }

    None
}

/// starts with function, but handles case insensitity when needed.
fn starts_with<'a>(needle: &'a str, haystack: &'a str, case_insensitive: bool) -> (bool, &'a str) {
    if case_insensitive {
        return starts_with_ignore_ascii_case(needle, haystack);
    }

    if haystack.starts_with(needle) {
        (true, &needle[haystack.len()..])
    } else {
        (false, "")
    }
}

/// starts_with function, but case insensitive.
fn starts_with_ignore_ascii_case<'a>(needle: &str, haystack: &'a str) -> (bool, &'a str) {
    let mut h_chars = haystack.chars();
    let n_chars = needle.chars();

    for nc in n_chars {
        match h_chars.next() {
            Some(hc) if hc.eq_ignore_ascii_case(&nc) => continue,
            _ => return (false, ""),
        }
    }

    (true, h_chars.as_str())
}

/// Manually dispatches a message with the prefix framework
pub async fn dispatch_message<'a, T: Send + Sync + 'static, E>(
    framework: crate::FrameworkContext<'a, T, E>,
    msg: &'a serenity::Message,
    trigger: crate::MessageDispatchTrigger,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    parent_commands: &'a mut Vec<&'a crate::Command<T, E>>,
) -> Result<(), crate::FrameworkError<'a, T, E>> {
    if let Some(ctx) =
        parse_invocation(framework, msg, trigger, invocation_data, parent_commands).await?
    {
        crate::catch_unwind_maybe(run_invocation(ctx))
            .await
            .map_err(|payload| crate::FrameworkError::CommandPanic {
                payload,
                ctx: ctx.into(),
            })??;
    } else if let Some(non_command_message) = framework.options.prefix_options.non_command_message {
        non_command_message(&framework, msg).await.map_err(|e| {
            crate::FrameworkError::NonCommandMessage {
                error: e,
                framework,
                msg,
            }
        })?;
    }
    Ok(())
}

/// Given a Message and some context data, parses prefix, command etc. out of the message and
/// returns the resulting [`crate::PrefixContext`]. To run the command, see [`run_invocation`].
///
/// Returns `Ok(None)` if the message does not look like a command invocation.
/// Returns `Err(...)` if the message _does_ look like a command invocation, but cannot be
/// fully parsed.
pub async fn parse_invocation<'a, T: Send + Sync + 'static, E>(
    framework: crate::FrameworkContext<'a, T, E>,
    msg: &'a serenity::Message,
    trigger: crate::MessageDispatchTrigger,
    invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    parent_commands: &'a mut Vec<&'a crate::Command<T, E>>,
) -> Result<Option<crate::PrefixContext<'a, T, E>>, crate::FrameworkError<'a, T, E>> {
    // Check if we're allowed to invoke from bot messages
    if msg.author.bot() && framework.options.prefix_options.ignore_bots {
        return Ok(None);
    }

    // Check if we're allowed to execute our own messages
    if framework.serenity_context.cache.current_user().id == msg.author.id
        && !framework.options.prefix_options.execute_self_messages
    {
        return Ok(None);
    }

    // Check if we can execute commands contained in thread creation messages
    if msg.kind == serenity::MessageType::ThreadCreated
        && framework.options.prefix_options.ignore_thread_creation
    {
        return Ok(None);
    }

    // Strip prefix, trim whitespace between prefix and rest, split rest into command name and args
    let (prefix, msg_content) = match strip_prefix(framework, msg).await {
        Some(x) => x,
        None => return Ok(None),
    };
    let msg_content = msg_content.trim_start();

    let (command, mod_chars, invoked_command_name, args) = find_command(
        &framework.options.commands,
        msg_content,
        framework.options.prefix_options.case_insensitive_commands,
        parent_commands,
    )
    .ok_or(crate::FrameworkError::UnknownCommand {
        msg,
        prefix,
        msg_content,
        framework,
        invocation_data,
        trigger,
    })?;

    let action = match command.prefix_action {
        Some(x) => x,
        // This command doesn't have a prefix implementation
        None => return Ok(None),
    };

    Ok(Some(crate::PrefixContext {
        msg,
        prefix,
        invoked_command_name,
        mod_chars,
        args,
        framework,
        parent_commands,
        command,
        invocation_data,
        trigger,
        action,
        __non_exhaustive: (),
    }))
}

/// Given an existing parsed command invocation from [`parse_invocation`], run it, including all the
/// before and after code like checks and built in filters from edit tracking
pub async fn run_invocation<T: Send + Sync + 'static, E>(
    ctx: crate::PrefixContext<'_, T, E>,
) -> Result<(), crate::FrameworkError<'_, T, E>> {
    // Check if we should disregard this invocation if it was triggered by an edit
    if ctx.trigger == crate::MessageDispatchTrigger::MessageEdit && !ctx.command.invoke_on_edit {
        return Ok(());
    }
    if ctx.trigger == crate::MessageDispatchTrigger::MessageEditFromInvalid
        && !ctx.framework.options.prefix_options.execute_untracked_edits
    {
        return Ok(());
    }

    if ctx.command.subcommand_required {
        // None of this command's subcommands were invoked, or else we'd have the subcommand in
        // ctx.command and not the parent command
        return Err(crate::FrameworkError::SubcommandRequired {
            ctx: crate::Context::Prefix(ctx),
        });
    }

    super::common::check_permissions_and_cooldown(ctx.into()).await?;

    // Typing is broadcasted as long as this object is alive
    let _typing_broadcaster = if ctx.command.broadcast_typing {
        Some(
            ctx.msg
                .channel_id
                .start_typing(ctx.framework.serenity_context.http.clone()),
        )
    } else {
        None
    };

    (ctx.framework.options.pre_command)(crate::Context::Prefix(ctx)).await;

    // Store that this command is currently running; so that if the invocation message is being
    // edited before a response message is registered, we don't accidentally treat it as an
    // execute_untracked_edits situation and start an infinite loop
    // Reported by vicky5124 https://discord.com/channels/381880193251409931/381912587505500160/897981367604903966
    if let Some(edit_tracker) = &ctx.framework.options.prefix_options.edit_tracker {
        edit_tracker
            .write()
            .unwrap()
            .track_command(ctx.msg, ctx.command.track_deletion);
    }

    // Execute command
    (ctx.action)(ctx).await?;

    (ctx.framework.options.post_command)(crate::Context::Prefix(ctx)).await;

    Ok(())
}
