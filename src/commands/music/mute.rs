use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

use super::consts::SONGBIRD_EXPECT;

#[command]
#[only_in(guilds)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx).await.expect(SONGBIRD_EXPECT).clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            msg.reply(ctx, "Not in a voice channel").await?;

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        msg.channel_id.say(ctx, "Already muted").await?;
    } else {
        if let Err(e) = handler.mute(true).await {
            msg.channel_id.say(ctx, format!("Failed: {:?}", e)).await?;
        }

        msg.channel_id.say(ctx, "Now muted").await?;
    }

    Ok(())
}
