use crate::{CommandCounter, ShardManagerContainer};
use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::fmt::Write;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
pub async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.channel_id
                .say(&ctx.http, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            msg.channel_id.say(&ctx.http, "No shard found").await?;

            return Ok(());
        }
    };

    match runner.latency {
        Some(duration) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("The shard latency is {}ms", duration.as_millis()),
                )
                .await?;
        }
        None => {
            msg.channel_id
                .say(&ctx.http, "Latency is not available yet")
                .await?;
        }
    };

    Ok(())
}

#[command]
#[bucket = "complicated"]
pub async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter.read().await.iter() {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;
    Ok(())
}
