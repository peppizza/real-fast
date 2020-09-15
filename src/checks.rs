use serenity::{
    framework::standard::{macros::check, Args, CheckResult, CommandOptions},
    model::prelude::*,
    prelude::*,
};

#[check]
#[name = "owner"]
pub async fn owner_check(
    _: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> CheckResult {
    if msg.author.id != 253290704384557057 {
        return CheckResult::new_user("You lack admin permissions");
    }

    CheckResult::Success
}
