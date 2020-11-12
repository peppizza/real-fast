use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use super::consts::SONGBIRD_EXPECT;

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            msg.channel_id.say(ctx, format!("Failed: {:?}", e)).await?;
        }

        msg.channel_id.say(ctx, "Unmuted").await?;
    } else {
        msg.channel_id
            .say(ctx, "Not in a voice channel to unmute in")
            .await?;
    }

    Ok(())
}
