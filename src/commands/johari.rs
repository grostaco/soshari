use itertools::Itertools;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    path::Path,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serenity::prelude::*;
use serenity::{
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
    },
    collector::ComponentInteractionCollectorBuilder,
    futures::StreamExt,
    model::prelude::{
        command::CommandOptionType, component::ButtonStyle, CommandDataOptionValue,
        CommandInteraction, ComponentInteractionDataKind, UserId,
    },
};
use soshari_macros::generate_adjectives;

use super::util::respond_embed_error;

generate_adjectives! { accepting, adaptable, bold, brave, calm, caring, cheerful, confident, dependable, dignified, energetic, extroverted,
friendly, giving, happy, helpful, idealistic, independent, ingenious, intelligent,
introverted, kind, knowledgeable, logical, loving, mature, modest, nervous, observant,
organised, patient, proud, quiet, reflective, relaxed, responsive,
 self_assertive, self_conscious, sensible, sentimental, shy, silly, spontaneous,
sympathetic, tense, trustworthy, warm, witty }

#[derive(Serialize, Deserialize)]
pub struct Johari {
    id: UserId,
    adjectives: Adjectives,
    others: Vec<Johari>,
}

pub struct JohariGroup(Vec<Johari>);

impl JohariGroup {
    fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(JohariGroup(
            serde_json::from_str(&fs::read_to_string(path)?).unwrap_or(Vec::new()),
        ))
    }

    fn dump<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        serde_json::to_writer_pretty(
            OpenOptions::new().write(true).truncate(true).open(path)?,
            &self.0,
        )?;
        Ok(())
    }

    fn get(&self, id: UserId) -> Option<&Johari> {
        self.0.iter().find(|johari| johari.id == id)
    }

    fn get_mut(&mut self, id: UserId) -> Option<&mut Johari> {
        self.0.iter_mut().find(|johari| johari.id == id)
    }

    fn push(&mut self, johari: Johari) {
        match self.get_mut(johari.id) {
            Some(s) => s.adjectives = johari.adjectives,
            None => self.0.push(johari),
        }
    }
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
    //println!("{:#?}", command.data);
    let mut johari_group = JohariGroup::load("johari.json").unwrap();
    let id = command.user.id;

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
                    if let Some(target) = johari_group.get_mut(target_id) {
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

            let selected = get_johari(&ctx, &command).await;

            if let Some(target) = target {
                target.others.push(Johari {
                    id,
                    adjectives: adjectives_to_bitflags(selected),
                    others: Vec::new(),
                });
            } else {
                johari_group.push(Johari {
                    id,
                    adjectives: adjectives_to_bitflags(selected),
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
            if let Some(johari) = johari_group.get(target_id) {
                let mut arena = Adjectives::empty();
                let mut blind = Adjectives::empty();
                let mut facade = Adjectives::empty();
                let mut unknown = Adjectives::all();
                for other in &johari.others {
                    arena |= johari.adjectives & other.adjectives;
                    blind |= !johari.adjectives & other.adjectives;
                    facade |= johari.adjectives & !other.adjectives;
                    unknown &= !(arena | blind | facade);
                }

                let empty_or = |s: String| if s.is_empty() { "N/A".to_string() } else { s };

                let embed = CreateEmbed::new()
                    .title("Johari window")
                    .description("The overall johari window")
                    .field(
                        "Arena",
                        empty_or(bitflags_to_adjectives(arena).join("\n")),
                        true,
                    )
                    .field(
                        "Blind",
                        empty_or(bitflags_to_adjectives(blind).join("\n")),
                        true,
                    )
                    .field("\u{200b}", "\u{200b}", false)
                    .field(
                        "Facade",
                        empty_or(bitflags_to_adjectives(facade).join("\n")),
                        true,
                    )
                    .field(
                        "Unknown",
                        empty_or(bitflags_to_adjectives(unknown).join("\n")),
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

async fn get_johari(ctx: &Context, command: &CommandInteraction) -> Vec<String> {
    let embed = |selected: &Vec<String>| {
        CreateEmbed::new()
            .title("The Johari window test")
            .description("The Johari Window was invented by Joseph Luft and Harrington Ingham in the 1950s as a model for mapping personality awareness")
            .field("Selected", if selected.is_empty() { "Nothing selected yet".into() } else { selected.join("\n") }, true)
            .color((0xFF, 0x5C, 0x5C))
            .footer(CreateEmbedFooter::new("This johari window is modified; see the original at https://kevan.org/johari"))
    };

    let mut selection: HashMap<String, bool> = ADJECTIVES
        .iter()
        .map(|adjective| (adjective.to_string(), false))
        .collect();

    let mut last_selected: Vec<String> = Vec::new();
    let mut selected: Vec<String> = Vec::new();

    let select_menu = |index, selected: &Vec<String>| {
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "select_menu",
                CreateSelectMenuKind::String {
                    options: ADJECTIVES
                        .iter()
                        .skip(25 * index)
                        .take(25)
                        .map(|adjective| {
                            CreateSelectMenuOption::new(adjective.clone(), adjective.clone())
                                .default_selection(
                                    selected
                                        .iter()
                                        .map(AsRef::<str>::as_ref)
                                        .contains(adjective),
                                )
                        })
                        .collect(),
                },
            )
            .min_values(0)
            .max_values(25),
        )
    };
    let buttons = |selected: &Vec<String>| {
        CreateActionRow::Buttons(vec![
            CreateButton::new("Prev", "prev").style(ButtonStyle::Primary),
            CreateButton::new("Next", "next").style(ButtonStyle::Primary),
            CreateButton::new("Submit", "submit")
                .style(ButtonStyle::Success)
                .disabled(selected.len() < 5),
        ])
    };

    let mut menu_index: usize = 0;

    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::default()
                    .embed(embed(&selected))
                    .components(vec![
                        select_menu(menu_index, &selected).clone(),
                        buttons(&selected),
                    ]),
            ),
        )
        .await
        .unwrap();

    let message = command.get_response(&ctx.http).await.unwrap();
    let mut collector = ComponentInteractionCollectorBuilder::new(&ctx.shard)
        .author_id(command.user.id)
        .message_id(message.id)
        .timeout(Duration::from_secs(600))
        .build();

    while let Some(interaction) = collector.next().await {
        match interaction.data.custom_id.as_str() {
            "select_menu" => {
                if let ComponentInteractionDataKind::StringSelect { values } =
                    interaction.data.kind.clone()
                {
                    for value in &values {
                        *selection.get_mut(value.as_str()).unwrap() = true;
                    }

                    if !last_selected.is_empty() {
                        for value in &last_selected {
                            if !values.contains(value) {
                                *selection.get_mut(value.as_str()).unwrap() = false;
                            }
                        }
                    }

                    last_selected = values.clone();
                    selected = selection
                        .iter()
                        .filter(|(_, set)| **set)
                        .map(|(k, _)| k.to_string())
                        .collect();
                }
            }
            button_selection @ ("next" | "prev") => {
                if button_selection == "next" {
                    menu_index = (menu_index + 1).min(ADJECTIVES.len() / 25);
                } else {
                    menu_index = menu_index.saturating_sub(1);
                }
                last_selected.clear();
            }
            "submit" => {
                interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::UpdateMessage(
                            CreateInteractionResponseMessage::new()
                                .embed(
                                    CreateEmbed::new()
                                        .title("Submission recorded")
                                        .description("Your submission has been recorded"),
                                )
                                .components(Vec::new()),
                        ),
                    )
                    .await
                    .unwrap();

                collector.stop();
                break;
            }

            _ => {}
        }
        interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .embed(embed(&selected))
                        .components(vec![
                            select_menu(menu_index, &selected).clone(),
                            buttons(&selected),
                        ]),
                ),
            )
            .await
            .unwrap();
    }

    selected
}
