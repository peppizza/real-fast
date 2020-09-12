use serenity::framework::standard::{macros::check, Args, CheckResult, CommandOptions};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[check]
#[name = "owner"]
async fn owner_check(_: &Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if msg.author.id != 253290704384557057 {
        return CheckResult::new_user("You lack admin permissions");
    }

    CheckResult::Success
}
