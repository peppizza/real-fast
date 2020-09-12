use crate::util::checks::*;
use crate::ShardManagerContainer;
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::parse_mention;

#[command]
#[checks(owner)]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(owner)]
pub async fn add_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = parse_mention(args.single::<String>().unwrap()).unwrap();
    let mut member = ctx
        .http
        .get_member(msg.guild_id.unwrap().as_u64().clone(), member)
        .await?;

    let role = parse_mention(args.single::<String>().unwrap()).unwrap();

    member.add_role(&ctx.http, RoleId(role)).await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "Gave role `{}` to `{}`",
                RoleId(role).to_role_cached(&ctx.cache).await.unwrap().name,
                msg.author.name
            ),
        )
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(owner)]
pub async fn remove_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = parse_mention(args.single::<String>().unwrap()).unwrap();
    let mut member = ctx
        .http
        .get_member(msg.guild_id.unwrap().as_u64().clone(), member)
        .await?;

    let role = parse_mention(args.single::<String>().unwrap()).unwrap();

    member.remove_role(&ctx.http, RoleId(role)).await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "Removed role `{}` from `{}`",
                RoleId(role).to_role_cached(&ctx.cache).await.unwrap().name,
                msg.author.name
            ),
        )
        .await?;

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
        Some(_) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "The shard latency is {}ms",
                        runner.latency.unwrap().as_millis()
                    ),
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
