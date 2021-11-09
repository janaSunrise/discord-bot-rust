use std::time::Duration;

use crate::{utils::uptime, ShardManagerContainer};

use serenity::{
    builder::{CreateEmbed, CreateEmbedAuthor},
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::Colour,
};

// Ping command
#[command]
#[description = "Ping command"]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, format!("Pong! Shard: {}", ctx.shard_id))
        .await?;

    Ok(())
}

// Latency
#[command]
#[description = "Get the latency of the bot"]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    msg.reply(
        ctx,
        &format!(
            "The shard latency is {:?}",
            runner.latency.unwrap_or(Duration::from_secs(0))
        ),
    )
    .await?;

    Ok(())
}

// Uptime
#[command]
#[description = "Get the uptime of the bot"]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let uptime = {
        let data = ctx.data.read().await;

        // Get the running duration
        let uptime = data
            .get::<uptime::Uptime>()
            .expect("Couldn't fetch uptime.");

        // Convert to string
        uptime.to_str()
    };

    // Send message
    msg.channel_id
        .send_message(&ctx, |cm| {
            cm.embed(|ce| {
                ce.title("Uptime")
                    .description(&uptime)
                    .color(Colour::BLURPLE)
            })
        })
        .await
        .unwrap();

    // OK
    Ok(())
}

// About
#[command]
#[description("About the bot")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    // Create embed
    let mut embed = CreateEmbed::default();

    embed.color(Colour::BLURPLE);
    embed.title("About this bot");
    embed.description("A simple discord bot. Built using Rust!");
    embed.field(
        "Info",
        format!("Bot v{}. Serenity: {}", env!("CARGO_PKG_VERSION"), "0.10"),
        true,
    );

    // Get author
    let mut auth = CreateEmbedAuthor::default();
    auth.name(&msg.author.name);
    auth.url(
        &msg.author
            .avatar_url()
            .unwrap_or(String::from(&msg.author.default_avatar_url())),
    );

    // Send message
    msg.channel_id
        .send_message(&ctx, |f| {
            f.content("").embed(|e| {
                e.0 = embed.0;
                e
            })
        })
        .await
        .unwrap();

    // OK
    Ok(())
}
