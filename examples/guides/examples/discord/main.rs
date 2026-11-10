mod rig_agent;


use anyhow::Result;
use rig::prelude::*;
use serenity::all::{
    async_trait, Command, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, EditInteractionResponse, EventHandler, GatewayIntents, Interaction,
    Message, Ready,
};


/// Discord rejects messages longer than 2000 characters.
fn truncate(text: String) -> String {
    match text.char_indices().nth(1990) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text,
    }
}


struct Handler {
    agent: Agent,
}


impl Handler {
    async fn answer(&self, question: &str) -> String {
        match self.agent.prompt(question).await {
            Ok(response) => truncate(response.output()),
            Err(e) => {
                eprintln!("agent error: {e}");
                "Sorry, something went wrong while answering.".to_string()
            }
        }
    }


    async fn ask(&self, ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
        command.defer(&ctx.http).await?;
        let question = command
            .data
            .options
            .first()
            .and_then(|option| option.value.as_str())
            .unwrap_or_default();
        let answer = self.answer(question).await;
        command
            .edit_response(&ctx.http, EditInteractionResponse::new().content(answer))
            .await?;
        Ok(())
    }
}


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
        let ask = CreateCommand::new("ask")
            .description("Ask the bot a question")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "query", "Your question")
                    .required(true),
            );
        if let Err(e) = Command::create_global_command(&ctx.http, ask).await {
            eprintln!("failed to register /ask: {e}");
        }
    }


    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.data.name == "ask" {
                if let Err(e) = self.ask(&ctx, &command).await {
                    eprintln!("failed to answer /ask: {e}");
                }
            }
        }
    }


    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || !msg.mentions_me(&ctx).await.unwrap_or(false) {
            return;
        }
        let mention = format!("<@{}>", ctx.cache.current_user().id);
        let question = msg.content.replace(&mention, "");
        let _ = msg.channel_id.broadcast_typing(&ctx.http).await;
        let answer = self.answer(question.trim()).await;
        if let Err(e) = msg.channel_id.say(&ctx.http, answer).await {
            eprintln!("failed to reply: {e}");
        }
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let token = std::env::var("DISCORD_TOKEN")?;
    let agent = rig_agent::build_agent(std::path::Path::new("documents")).await?;


    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler { agent })
        .await?;
    client.start().await?;
    Ok(())
}
async fn history_example(agent: &Agent,channel_id:u64)-> Result<()> {
use std::collections::HashMap; use rig::message::Message;
let mut histories: HashMap<u64, Vec<Message>> = HashMap::new();


let history = histories.entry(channel_id).or_default();
let answer = agent.chat("And how do I configure it?", history).await?.output();
Ok(())
}
async fn prompt_example()-> Result<()> {
let agent = rig_agent::build_agent(std::path::Path::new("documents")).await?;
let answer = agent.prompt("What is this project about?").await?.output();
println!("{answer}");
Ok(())
}