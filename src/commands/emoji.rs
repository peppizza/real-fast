use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::parse_emoji,
    Result as SerenityResult,
};

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_EMOJIS")]
pub async fn new_emoji(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single_quoted::<String>()?;
    let image = args.single_quoted::<String>()?;

    let resp = reqwest::get(&image).await?;

    let last_segment = image
        .split('/')
        .last()
        .ok_or(SerenityError::Other("Could not get image type"))?;

    let emoji = msg
        .guild_id
        .unwrap()
        .create_emoji(
            &ctx.http,
            &name,
            &read_image(&last_segment, &resp.bytes().await?)?,
        )
        .await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!("Created emoji {} with name {}", emoji.mention(), emoji.name),
        )
        .await?;

    Ok(())
}

fn read_image(last_segment: &str, bytes: &bytes::Bytes) -> SerenityResult<String> {
    let b64 = base64::encode(&bytes);
    if last_segment.contains("png") {
        Ok(format!("data:image/png;base64,{}", b64))
    } else if last_segment.contains("jpg") || last_segment.contains("jpeg") {
        Ok(format!("data:image/jpeg;base64,{}", b64))
    } else if last_segment.contains("gif") {
        Ok(format!("data:image/gif;base64,{}", b64))
    } else {
        Err(SerenityError::Other("Image is not valid"))
    }
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_EMOJIS")]
pub async fn remove_emoji(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single_quoted::<String>()?;
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
    let name = args.single_quoted::<String>()?;
    let new_name = args.single_quoted::<String>()?;
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
