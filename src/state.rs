use std::{collections::HashMap, sync::Arc};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    model::prelude::{Activity, GuildId, Ready, ResumedEvent},
    prelude::*,
};
use tracing::info;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = Arc<RwLock<HashMap<String, u64>>>;
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!(
            "Connected as {}#{} ({})",
            ready.user.name, ready.user.discriminator, ready.user.id
        );
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        ctx.set_activity(Activity::playing(
            format!("with {} guilds", guilds.len()).as_str(),
        ))
        .await;
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}
