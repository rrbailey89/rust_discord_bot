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

use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::Permissions;

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
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
    ]
}