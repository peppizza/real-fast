mod commands;

use serenity::{
    async_trait,
    client::{bridge::gateway::ShardManager, validate_token},
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
use std::{
    collections::{HashMap, HashSet},
    env,
    sync::Arc,
};
use tokio::{signal, sync::RwLock};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{emoji::*, help::*, math::*, roles::*, util::*};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = Arc<RwLock<HashMap<String, u64>>>;
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
#[commands(multiply, latency, ping, commands)]
struct General;

#[group]
#[commands(new_emoji, remove_emoji, rename_emoji)]
struct Emoji;

#[group]
#[commands(add_role, remove_role, create_role, delete_role)]
struct Role;

#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    debug!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    let counter_lock = {
        let data_read = ctx.data.read().await;

        data_read
            .get::<CommandCounter>()
            .expect("Expected CommandCounter in TypeMap.")
            .clone()
    };

    {
        let mut counter = counter_lock.write().await;

        let entry = counter.entry(command_name.to_string()).or_insert(0);
        *entry += 1;
    }

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
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let token = env::var("DISCORD_TOKEN")?;

    validate_token(&token)?;

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
        .bucket("complicated", |b| b.delay(5).time_span(30).limit(2))
        .await
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
        .await?;

    {
        let mut data = client.data.write().await;
        data.insert::<CommandCounter>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}
