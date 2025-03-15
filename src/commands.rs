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
mod set_url_rule;
mod is_alive;
mod set_reaction_log;
mod relay;
mod flux_image;
mod iscaliforniaonfire;
mod whereiscaliforniaonfire;
mod set_unavailability_channel;
mod unavailable;
mod list_unavailable;
mod cancel_unavailable;

use crate::error::Error;
use crate::Data;

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
        purge::purge(),
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
        set_url_rule::seturlrule(),
        is_alive::lifecheck(),
        set_reaction_log::reactionslog(),
        relay::relay(),
        flux_image::fluximage(),
        iscaliforniaonfire::iscaliforniaonfire(),
        whereiscaliforniaonfire::whereiscaliforniaonfire(),
        set_unavailability_channel::setunavailabilitychannel(),
        unavailable::unavailable(),
        list_unavailable::listunavailable(),
        cancel_unavailable::cancelunavailable(),
    ]
}
