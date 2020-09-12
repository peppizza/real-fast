mod commands;

use log::{error, info};
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{
        event::ResumedEvent,
        gateway::{Activity, Ready},
    },
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};
use tokio::signal;

use commands::{math::*, util::*};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);

        ctx.set_activity(Activity::playing("name")).await;
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[group]
#[commands(multiply, add_role, remove_role, latency)]
struct General;

#[tokio::main]
async fn main() {
    kankyo::load(false).expect("Failed to load .env file");

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
        .configure(|c| c.owners(owners).prefix("!"))
        .group(&GENERAL_GROUP);

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
