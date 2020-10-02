use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::{parse_emoji, read_image},
};
use std::{fs::File, io::copy};
use tempfile::Builder;

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_EMOJIS")]
pub async fn new_emoji(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single::<String>()?;
    let image = args.single::<String>()?;

    let tmp_dir = Builder::new().tempdir()?;
    let resp = reqwest::get(&image).await?;

    let mut dest = {
        let fname = resp
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        let path = tmp_dir.path().join(fname);
        let file = File::create(&path)?;
        (path, file)
    };

    copy(&mut resp.bytes().await?.as_ref(), &mut dest.1)?;

    let emoji = msg
        .guild_id
        .unwrap()
        .create_emoji(&ctx.http, &name, &read_image(&dest.0)?)
        .await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!("Created emoji {} with name {}", emoji.mention(), emoji.name),
        )
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_EMOJIS")]
pub async fn remove_emoji(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single::<String>()?;
    let emoji = match parse_emoji(name) {
        Some(e) => e,
        None => {
            msg.channel_id
                .say(&ctx.http, "That emoji was not found")
                .await?;
            return Ok(());
        }
    };

    msg.guild_id
        .unwrap()
        .delete_emoji(&ctx.http, emoji.id)
        .await?;

    msg.channel_id.say(&ctx.http, "Deleted emoji").await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_EMOJIS")]
pub async fn rename_emoji(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single::<String>()?;
    let new_name = args.single::<String>()?;
    let emoji = match parse_emoji(name) {
        Some(e) => e,
        None => {
            msg.channel_id
                .say(&ctx.http, "That emoji was not found")
                .await?;
            return Ok(());
        }
    };

    let emoji = msg
        .guild_id
        .unwrap()
        .edit_emoji(&ctx.http, emoji.id, &new_name)
        .await?;

    msg.channel_id
        .say(&ctx.http, format!("Renamed emoji to {}", emoji.name))
        .await?;

    Ok(())
}
