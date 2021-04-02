use std::{ collections::{ HashSet }, env, sync::Arc, time::Duration};

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
            Args,
            CommandGroup,
            CommandResult,
            StandardFramework,
            HelpOptions,
            help_commands,
            macros::{ command, group, help }
        }
    },
    model::{
        channel::Message,
        gateway::Ready,
        id::UserId,
    },
    http::Http,
    prelude::*
};
use tokio::time::sleep;

// Shard management
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[help]
#[individual_command_tip =
"Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;

    Ok(())
}

// Commands implementation
#[group]
#[commands(ping, latency)]
struct General;

// Handler impl
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
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
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");

    // Fetch owners
    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Define the framework
    let framework = StandardFramework::new()
        .configure(|c| c
                    .with_whitespace(true)
                    .on_mention(Some(bot_id))
                    .prefix("~")
                    .delimiters(vec![", ", ","])
                    .owners(owners)
        )
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    // Define the client
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

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

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager").await?;

            return Ok(());
        },
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx,  "No shard found").await?;

            return Ok(());
        },
    };

    msg.reply(ctx, &format!("The shard latency is {:?}", runner.latency)).await?;

    Ok(())
}
