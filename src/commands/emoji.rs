use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::read_image,
};
use std::{fs::File, io::copy};
use tempfile::Builder;

#[command]
async fn new_emoji(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = args.single::<String>().unwrap();
    let image = args.single::<String>().unwrap();

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

    msg.guild_id
        .unwrap()
        .create_emoji(&ctx.http, &name, &read_image(&dest.0).unwrap())
        .await?;

    Ok(())
}
