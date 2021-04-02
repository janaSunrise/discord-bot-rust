use std::{ env, sync::Arc, time::Duration};

use serenity::{
    async_trait,
    client::{
        Client, Context, EventHandler,
        bridge::{
            gateway::{
                ShardId, ShardManager
            }
        },
    },
    framework::{
        standard::{
            CommandResult,
            StandardFramework,
            macros::{ command, group }
        }
    },
    model::{
        channel::{ Message },
        gateway::{ Ready },
    },
    prelude::*
};
use tokio::time::sleep;

// Shard management
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Commands implementation
#[group]
#[commands(ping)]
struct General;

// Handler impl
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        if let Some(shard) = ready.shard {
            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name,
                shard[0],
                shard[1],
            );
        }
    }
}

// Main section
#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // Shard management
    let manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;

            let lock = manager.lock().await;
            let shard_runners = lock.runners.lock().await;

            for (id, runner) in shard_runners.iter() {
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id,
                    runner.stage,
                    runner.latency,
                );
            }
        }
    });

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

// Commands
#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, format!("Pong! Shard: {}", ctx.shard_id)).await?;

    Ok(())
}