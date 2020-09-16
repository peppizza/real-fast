mod commands;

use log::{debug, error, info, warn};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{
        standard::{
            macros::{group, hook},
            CommandResult, DispatchError, Reason,
        },
        StandardFramework,
    },
    http::Http,
    model::{
        channel::Message,
        event::ResumedEvent,
        gateway::{Activity, Ready},
    },
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};
use tokio::signal;

use commands::{emoji::*, help::*, math::*, roles::*, util::*};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            "Connected as {}#{} ({})",
            ready.user.name, ready.user.discriminator, ready.user.id
        );

        ctx.set_activity(Activity::playing(
            format!("with {} guilds", ready.guilds.len()).as_str(),
        ))
        .await;
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(multiply, latency, ping)]
struct General;

#[group]
#[commands(new_emoji, remove_emoji, rename_emoji)]
struct Emoji;

#[group]
#[commands(add_role, remove_role)]
struct Role;

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    debug!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => debug!("Processed command '{}'", command_name),
        Err(why) => warn!("Command '{}' returned error {}", command_name, why),
    }
}

#[hook]
async fn unkown_command(_ctx: &Context, _msg: &Message, unkown_command_name: &str) {
    debug!("Could not find command named '{}'", unkown_command_name);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::Ratelimited(duration) => {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    format!("Try this again in {} seconds.", duration.as_secs()),
                )
                .await;
        }
        DispatchError::CheckFailed(check, reason) => {
            if let Reason::User(reason) = reason {
                let _ = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        format!("Check {} failed with error {:?}", check, reason),
                    )
                    .await;
            }
        }

        why => debug!("Command failed with error: {:?}", why),
    }
}

#[tokio::main]
async fn main() {
    kankyo::init().expect("Failed to load .env file");

    env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token to be in the environment");

    let http = Http::new_with_token(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix("~"))
        .group(&GENERAL_GROUP)
        .group(&EMOJI_GROUP)
        .group(&ROLE_GROUP)
        .before(before)
        .after(after)
        .unrecognised_command(unkown_command)
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Error");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }
}
