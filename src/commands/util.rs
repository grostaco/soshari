use serenity::{
    builder::{CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage},
    http::Http,
    model::prelude::CommandInteraction,
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
