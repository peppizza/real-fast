use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::parse_mention,
};
use tokio::time::Duration;

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_ROLES")]
pub async fn add_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = parse_mention(args.single::<String>()?).ok_or(UserIdParseError::InvalidFormat)?;
    let mut member = ctx.http.get_member(msg.guild_id.unwrap().0, member).await?;

    let role = parse_mention(args.single::<String>()?).ok_or(RoleIdParseError::InvalidFormat)?;

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
    let member = parse_mention(args.single::<String>()?).ok_or(UserIdParseError::InvalidFormat)?;
    let mut member = ctx.http.get_member(msg.guild_id.unwrap().0, member).await?;

    let role = parse_mention(args.single::<String>()?).ok_or(RoleIdParseError::InvalidFormat)?;

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

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_ROLES")]
pub async fn create_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = {
        if args.len() >= 1 {
            args.single::<String>()?
        } else {
            msg.channel_id
                .say(&ctx.http, "Enter the name of the new role")
                .await?;
            match msg
                .author
                .await_reply(&ctx)
                .timeout(Duration::from_secs(10))
                .await
            {
                Some(name) => name.content.clone(),
                None => {
                    msg.channel_id
                        .say(&ctx.http, "No answer within 10 seconds")
                        .await?;
                    return Ok(());
                }
            }
        }
    };

    let role = msg
        .guild_id
        .unwrap()
        .create_role(&ctx.http, |r| r.name(&name))
        .await?;

    msg.channel_id
        .say(&ctx.http, format!("Created role {}", role.mention()))
        .await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[required_permissions("MANAGE_ROLES")]
pub async fn delete_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let name = {
        if args.len() >= 1 {
            args.single::<String>()?
        } else {
            msg.channel_id
                .say(&ctx.http, "Mention the role to delete")
                .await?;
            match msg
                .author
                .await_reply(&ctx)
                .timeout(Duration::from_secs(10))
                .await
            {
                Some(name) => name.content.clone(),
                None => {
                    msg.channel_id
                        .say(&ctx.http, "No answer within 10 seconds")
                        .await?;
                    return Ok(());
                }
            }
        }
    };

    let role = match parse_mention(&name) {
        Some(mention) => mention,
        None => {
            msg.channel_id
                .say(&ctx.http, "Could not find that role")
                .await?;
            return Ok(());
        }
    };

    msg.guild_id.unwrap().delete_role(&ctx.http, role).await?;
    msg.channel_id
        .say(&ctx.http, format!("Deleted role"))
        .await?;

    Ok(())
}
