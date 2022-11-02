use std::{collections::HashMap, time::Duration};

use itertools::Itertools;
use serenity::{
    builder::{
        CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateSelectMenu, CreateSelectMenuKind,
        CreateSelectMenuOption,
    },
    collector::ComponentInteractionCollectorBuilder,
    futures::StreamExt,
    http::Http,
    model::prelude::{component::ButtonStyle, CommandInteraction, ComponentInteractionDataKind},
    prelude::*,
};

pub async fn respond_embed_error(
    http: impl AsRef<Http>,
    interaction: CommandInteraction,
    message: &str,
) {
    let embed = CreateEmbed::new()
        .title("Error")
        .color((255, 0, 0))
        .description(message);
    interaction
        .create_response(
            http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().embed(embed),
            ),
        )
        .await
        .unwrap();
}

pub async fn menu_get(
    embed: CreateEmbed,
    ctx: &Context,
    command: &CommandInteraction,
    adjectives: &[&str],
    min: usize,
) -> Vec<String> {
    let embed = |selected: &Vec<String>| {
        embed.clone().field(
            "Selected",
            if selected.is_empty() {
                "Nothing selected yet".into()
            } else {
                selected.join("\n")
            },
            true,
        )
    };

    let mut selection: HashMap<String, bool> = adjectives
        .iter()
        .map(|adjective| (adjective.to_string(), false))
        .collect();

    let mut last_selected: Vec<String> = Vec::new();
    let mut selected: Vec<String> = Vec::new();

    let select_menu = |index, selected: &Vec<String>| {
        let adjectives: Vec<_> = adjectives
            .iter()
            .skip(25 * index)
            .take(25)
            .map(|adjective| {
                CreateSelectMenuOption::new(adjective.clone(), adjective.clone()).default_selection(
                    selected
                        .iter()
                        .map(AsRef::<str>::as_ref)
                        .contains(adjective),
                )
            })
            .collect();
        let n = adjectives.len() as u64;
        CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                "select_menu",
                CreateSelectMenuKind::String {
                    options: adjectives,
                },
            )
            .min_values(0)
            .max_values(n),
        )
    };
    let buttons = |selected: &Vec<String>| {
        CreateActionRow::Buttons(vec![
            CreateButton::new("Prev", "prev").style(ButtonStyle::Primary),
            CreateButton::new("Next", "next").style(ButtonStyle::Primary),
            CreateButton::new("Submit", "submit")
                .style(ButtonStyle::Success)
                .disabled(selected.len() < min),
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
                    menu_index = (menu_index + 1).min(adjectives.len() / 25);
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
