use crate::state::TrackEndNotifier;

use super::consts::SONGBIRD_EXPECT;
use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::{Event as SongbirdEvent, TrackEvent};
use std::time::Duration;

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();

    let (handle_lock, success) = manager.join(guild_id, connect_to).await;

    if success.is_ok() {
        msg.channel_id
            .say(ctx, &format!("Joined {}", connect_to.mention()))
            .await?;

        let chan_id = msg.channel_id;

        let send_http = ctx.http.clone();

        let mut handle = handle_lock.lock().await;

        handle.add_global_event(
            SongbirdEvent::Track(TrackEvent::End),
            TrackEndNotifier {
                chan_id,
                http: send_http,
            },
        );
    } else {
        msg.channel_id.say(ctx, "Error joining the channel").await?;
    }

    Ok(())
}
