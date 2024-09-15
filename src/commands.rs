// commands.rs
mod warn;
mod set_warn_channel;
mod random_cat_image;
mod random_capy_image;
mod user_info;
mod anime_hug;
mod update_raid_time;
mod ask;
mod purge;
mod set_delete_log_channel;
mod ping;
mod toggle_emoji_reactions;
mod createimage;
pub(crate) mod reminder;
mod help;
mod weather;
mod blame_serena;
mod rules;
mod set_level_up_channel;
pub(crate) mod add_role_buttons;

use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::Permissions;

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        help::help(),
        warn::warn(),
        set_warn_channel::setwarnchannel(),
        random_cat_image::randomcatimage(),
        random_capy_image::randomcapyimage(),
        user_info::userinfo(),
        anime_hug::animehug(),
        update_raid_time::updateraidtime(),
        ask::ask(),
        {
            let mut cmd = purge::purge();
            cmd.default_member_permissions = Permissions::MANAGE_MESSAGES;
            cmd
        },
        set_delete_log_channel::setdeletemessagechannel(),
        ping::ping(),
        toggle_emoji_reactions::toggleemojireactions(),
        createimage::createimage(),
        reminder::reminder(),
        weather::weather(),
        blame_serena::blame(),
        rules::rule(),
        set_level_up_channel::setlevelupchannel(),
        add_role_buttons::rolebuttons(),
    ]
}