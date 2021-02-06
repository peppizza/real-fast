use std::{collections::HashMap, sync::Arc};

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    model::prelude::{Activity, GuildId, Ready, ResumedEvent, VoiceState},
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

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        _: Option<VoiceState>,
        new: VoiceState,
    ) {
        let guild = guild_id
            .unwrap()
            .to_guild_cached(ctx.clone())
            .await
            .unwrap();
        let role = guild.role_by_name("Late Night Crew").unwrap();
        let user = new.user_id.to_user(ctx.clone()).await.unwrap();

        if !user.has_role(ctx.clone(), guild.id, role).await.unwrap()
            && new.channel_id.unwrap() == 807382613076738048
        {
            let _ = guild
                .edit_member(ctx, user.id, |e| e.disconnect_member())
                .await;
        }
    }
}
