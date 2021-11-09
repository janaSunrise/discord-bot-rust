// Modules
mod commands;
mod utils;

// Imports
use std::{collections::HashSet, env, sync::Arc};

use serenity::{
    async_trait,
    client::bridge::gateway::{GatewayIntents, ShardManager},
    framework::standard::{
        help_commands,
        macros::{group, help, hook},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::UserId},
    prelude::*,
};

use commands::info::*;
use utils::uptime;

// Handler
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("Using API v{}", ready.version);
        println!("ID: {}", ready.session_id);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("Resumed connection.");
    }
}

// Shard management
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Help command
#[help]
#[individual_command_tip = "Use `{prefix}{command} {subcommand}` for more info on a command."]
#[command_not_found_text = "Could not find the command `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Strike"]
#[wrong_channel = "Hide"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;

    Ok(())
}

// Hooks
#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => {}
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

// Command groups
#[group]
#[description = "Get info about the bot."]
#[commands(ping, uptime, latency, about)]
struct General;

// Main function
#[tokio::main]
async fn main() {
    // Get token
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create the instance of bot
    let http = Http::new_with_token(&token);

    // Fetch owners and ID
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Framework
    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("~")
                .delimiters(vec![", ", ","])
                .owners(owners)
        })
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    // Client building
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .await
        .expect("Error creating client");

    // Global ctx data
    {
        let mut data = client.data.write().await;

        // Shard manager
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));

        // Uptime
        data.insert::<uptime::Uptime>(uptime::Uptime::new());
    }

    let shard_manager = client.shard_manager.clone();

    // Spawn service
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("CTRL + C not registered.");

        shard_manager.lock().await.shutdown_all().await;
    });

    // Start client
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
