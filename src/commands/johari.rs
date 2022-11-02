use itertools::Itertools;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serenity::prelude::*;
use serenity::{
    builder::{CreateCommand, CreateCommandOption, CreateEmbed,
        CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    model::prelude::{
        command::CommandOptionType, CommandDataOptionValue,
        CommandInteraction
    },
};
use soshari_macros::adjectives;

use super::util::{menu_get, respond_embed_error};

#[adjectives(
    accepting, adaptable, bold, brave, calm, caring, cheerful, confident, dependable, dignified, 
    energetic, extroverted, friendly, giving, happy, helpful, idealistic, independent, ingenious, 
    intelligent, introverted, kind, knowledgeable, logical, loving, mature, modest, nervous, observant, 
    organised, patient, proud, quiet, reflective, relaxed, responsive, self_assertive, self_conscious, sensible, 
    sentimental, shy, silly, spontaneous, sympathetic, tense, trustworthy, warm, witty, wise
)]
#[derive(Serialize, Deserialize)]
pub struct Johari {
    id: u64,
    adjectives: JohariAdjectives,
    others: Vec<Johari>,
}

pub fn create() -> CreateCommand {
    CreateCommand::new("johari")
        .description("The johari window test")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "start",
                "Start the johari window test",
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "User to contribute to",
            )),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "query",
                "Query for a user by id in the johari database",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "User to query")
                    .required(true),
            ),
        )
}

pub async fn run(ctx: Context, command: CommandInteraction) {
    let mut johari_group = JohariGroup::load("johari.json").unwrap();
    let id = command.user.id;
    let embed = 
        CreateEmbed::new()
            .title("The Johari window test")
            .description("The Johari Window was invented by Joseph Luft and Harrington Ingham in the 1950s as a model for mapping personality awareness")
            .color((0xFF, 0x5C, 0x5C))
            .footer(CreateEmbedFooter::new("This johari window is modified; see the original at https://kevan.org/johari"))
    ;

    match &command.data.options[..] {
        [start] if start.name == "start" => {
            let target = if let CommandDataOptionValue::SubCommand(subcommand) = &start.value {
                if !subcommand.is_empty() {
                    let target_id = subcommand.first().unwrap().value.as_user_id().unwrap();
                    if target_id == id {
                        respond_embed_error(
                            &ctx.http,
                            command,
                            "You cannot contribute to yourself!",
                        )
                        .await;
                        return;
                    }
                    if let Some(target) = johari_group.get_mut(target_id.into()) {
                        Some(target)
                    } else {
                        respond_embed_error(
                            &ctx.http,
                            command,
                            "Cannot find user in the johari database",
                        )
                        .await;
                        return;
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let selected = menu_get(embed, &ctx, &command, JohariAdjectives::adjectives(), 5).await;

            if let Some(target) = target {
                target.others.push(Johari {
                    id: id.into(),
                    adjectives: selected.into(),
                    others: Vec::new(),
                });
            } else {
                johari_group.push(Johari {
                    id: id.into(),
                    adjectives: selected.into(),
                    others: Vec::new(),
                });
            }

            johari_group.dump("johari.json").unwrap();
        }
        [query] if query.name == "query" => {
            let target_id = if let CommandDataOptionValue::SubCommand(subcommand) = &query.value {
                subcommand.first().unwrap().value.as_user_id().unwrap()
            } else {
                respond_embed_error(
                    &ctx.http,
                    command,
                    "The program reached a (supposedly) unreachable state. Something went wrong",
                )
                .await;
                return;
            };
            if let Some(johari) = johari_group.get(target_id.into()) {
                let guild_id = command.guild_id.unwrap();
                let color = match guild_id
                    .member(&ctx.http, target_id)
                    .await
                    .unwrap()
                    .roles(&ctx.cache)
                {
                    Some(role) => role
                        .last()
                        .map(|role| role.colour)
                        .unwrap_or((0, 0, 0).into()),
                    None => (0, 0, 0).into(),
                };

                let mut arena: HashMap<&&str, usize> = JohariAdjectives::adjectives()
                    .iter()
                    .map(|adjective| (adjective, 0))
                    .collect();
                let mut blind = arena.clone();
                let mut facade = johari.adjectives;
                let mut unknown = johari.adjectives.complement();

                for other in &johari.others {
                    let arena_bitflags = johari.adjectives & other.adjectives;
                    let blind_bitflags = !johari.adjectives & other.adjectives;

                    for adjective in arena_bitflags.as_adjectives() {
                        *arena.get_mut(&adjective).unwrap() += 1;
                    }
                    for adjective in blind_bitflags.as_adjectives() {
                        *blind.get_mut(&adjective).unwrap() += 1;
                    }
                    facade &= !(arena_bitflags | blind_bitflags);
                    unknown &= !other.adjectives;
                }

                let empty_or = |s: String| {
                    if s.is_empty() {
                        "N/A".to_string()
                    } else {
                        format!("```fix\n{s}\n```")
                    }
                };

                let embed = CreateEmbed::new()
                    .title("Johari window")
                    .description("The overall johari window")
                    .color(color)
                    .field(
                        "Arena",
                        empty_or(
                            arena
                                .iter()
                                .filter_map(|(adj, n)| {
                                    if *n > 1 {
                                        Some(format!("{adj} ({n})"))
                                    } else if *n == 1 {
                                        Some(format!("{adj}"))
                                    } else {
                                        None
                                    }
                                })
                                .join("\n"),
                        ),
                        true,
                    )
                    .field(
                        "Blind",
                        empty_or(
                            blind
                                .iter()
                                .filter_map(|(adj, n)| {
                                    if *n > 1 {
                                        Some(format!("{adj} ({n})"))
                                    } else if *n == 1 {
                                        Some(format!("{adj}"))
                                    } else {
                                        None
                                    }
                                })
                                .join("\n"),
                        ),
                        true,
                    )
                    .field("Facade", empty_or(facade.as_adjectives().join("\n")), true)
                    .field(
                        "Unknown",
                        empty_or(unknown.as_adjectives().join("\n")),
                        true,
                    );
                command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().embed(embed),
                        ),
                    )
                    .await
                    .unwrap();
            } else {
                respond_embed_error(
                    &ctx.http,
                    command,
                    "Cannot find the user in the johari database",
                )
                .await;
            }
        }
        _ => {
            respond_embed_error(
                &ctx.http,
                command,
                "The program reached a (supposedly) unreachable state. Something went wrong",
            )
            .await;
            panic!("Unreachable state (johari matching error)");
        }
    }
}
