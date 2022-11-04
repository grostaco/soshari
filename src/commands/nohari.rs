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

use super::util::{respond_embed_error, menu_get};


#[adjectives(incompetent, intolerant, inflexible, timid, cowardly, violent, aloof, glum, stupid, simple,
insecure, irresponsible, vulgar, lethargic, withdrawn, hostile, selfish, unhappy, unhelpful,
cynical, needy, unimaginative, inane, brash, cruel, ignorant, irrational, distant, childish, boastful,
blase, imperceptive, chaotic, impatient, weak, embarrassed, loud, vacuous, panicky, unethical, insensitive,
self_satisfied, passive, smug, rash, dispassionate, overdramatic, dull, predictable, callous, inattentive, unreliable, cold, foolish, humourless)]
#[derive(Serialize, Deserialize)]
pub struct Nohari {
    id: u64,
    adjectives: NohariAdjectives,
    others: Vec<Nohari>,
}

pub fn create() -> CreateCommand {
    CreateCommand::new("nohari")
        .description("The nohari window test")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "start",
                "Start the nohari window test",
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
                "Query for a user by id in the nohari database",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::User, "user", "User to query")
                    .required(true),
            ),
        )
}

pub async fn run(ctx: Context, command: CommandInteraction) {
    let mut nohari_group = NohariGroup::load("nohari.json").unwrap();
    let id = command.user.id;
    let embed = 
        CreateEmbed::new()
            .title("The nohari window test")
            .description("The Nohari is a darker version of the Johari Window, invented by Joseph Luft and Harrington Ingham in the 1950s as a model for mapping personality awareness")
            .color((0xFF, 0x5C, 0x5C))
            .footer(CreateEmbedFooter::new("This nohari window is modified; see the original at https://kevan.org/nohari"));

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
                    if let Some(target) = nohari_group.get_mut(target_id) {
                        Some(target)
                    } else {
                        respond_embed_error(
                            &ctx.http,
                            command,
                            "Cannot find user in the nohari database",
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

            let selected = menu_get(embed, &ctx, &command, NohariAdjectives::adjectives(), 3).await;

            if let Some(target) = target {
                target.others.push(Nohari {
                    id: id.into(),
                    adjectives: selected.into(),
                    others: Vec::new(),
                });
            } else {
                nohari_group.push(Nohari {
                    id: id.into(),
                    adjectives: selected.into(),
                    others: Vec::new(),
                });
            }

            nohari_group.dump("nohari.json").unwrap();
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
            if let Some(nohari) = nohari_group.get(target_id) {
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

                let mut arena: HashMap<&&str, usize> = NohariAdjectives::adjectives()
                    .iter()
                    .map(|adjective| (adjective, 0))
                    .collect();
                let mut blind = arena.clone();
                let mut facade = nohari.adjectives;
                let mut unknown = nohari.adjectives.complement();

                for other in &nohari.others {
                    let arena_bitflags = nohari.adjectives & other.adjectives;
                    let blind_bitflags = !nohari.adjectives & other.adjectives;

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
                    .title("nohari window")
                    .description("The overall nohari window")
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
                    "Cannot find the user in the nohari database",
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
            panic!("Unreachable state (nohari matching error)");
        }
    }
}
