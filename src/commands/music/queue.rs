use crate::state::VoiceQueueManager;

use super::consts::{SONGBIRD_EXPECT, VOICEQUEUEMANAGER_NOT_FOUND};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

use songbird::input;

use tracing::error;

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .say(ctx, "Mut provide a URL to a video or audio")
                .await?;

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        msg.channel_id.say(ctx, "Mut provide a valid URL").await?;

        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();
    let queues_lock = ctx
        .data
        .read()
        .await
        .get::<VoiceQueueManager>()
        .cloned()
        .expect(VOICEQUEUEMANAGER_NOT_FOUND);

    let mut track_queues = queues_lock.lock().await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let source = match input::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);

                msg.channel_id.say(ctx, "Error sourcing ffmpeg").await?;

                return Ok(());
            }
        };

        let queue = track_queues.entry(guild_id).or_default();

        queue.add_source(source, &mut handler);

        msg.channel_id
            .say(
                ctx,
                format!("Added song to queue: position {}", queue.len()),
            )
            .await?;
    } else {
        msg.channel_id
            .say(ctx, "Not in a voice channel to play in")
            .await?;
    }

    Ok(())
}
