use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::parse_mention,
};

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_ROLES")]
pub async fn add_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = parse_mention(args.single::<String>().unwrap()).unwrap();
    let mut member = ctx.http.get_member(msg.guild_id.unwrap().0, member).await?;

    let role = parse_mention(args.single::<String>().unwrap()).unwrap();

    member.add_role(&ctx.http, RoleId(role)).await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "Gave role `{}` to `{}`",
                RoleId(role).to_role_cached(&ctx.cache).await.unwrap().name,
                member.display_name()
            ),
        )
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_ROLES")]
pub async fn remove_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = parse_mention(args.single::<String>().unwrap()).unwrap();
    let mut member = ctx.http.get_member(msg.guild_id.unwrap().0, member).await?;

    let role = parse_mention(args.single::<String>().unwrap()).unwrap();

    member.remove_role(&ctx.http, RoleId(role)).await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!(
                "Removed role `{}` from `{}`",
                RoleId(role).to_role_cached(&ctx.cache).await.unwrap().name,
                member.display_name()
            ),
        )
        .await?;

    Ok(())
}
