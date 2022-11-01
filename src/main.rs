use dotenv_codegen::dotenv;

use serenity::{
    async_trait,
    model::prelude::{command::Command, Interaction, Ready},
    prelude::{Context, EventHandler, GatewayIntents},
    Client,
};

mod commands;

struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command.data.name.as_str() {
                "johari" => commands::johari::run(ctx, command).await,
                _ => println!(":( Unimplemented"),
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);

        Command::create_global_application_command(&ctx.http, commands::johari::create())
            .await
            .expect("Error while creating new command");
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::empty();
    let handler = Handler {};
    println!("{:#?}", dotenv!("DISCORD_TOKEN"));
    let mut client = Client::builder(
        "MTAzNjc4NTcwMDgxMDUyMjY2NQ.GnNhpQ.BotCUTMhHZHAbOsjgEIkiypMbj1NClJQhs6EG8",
        intents,
    )
    .event_handler(handler)
    .await
    .expect("Cannot create client");

    client.start().await.expect("Cannot start client");
}
