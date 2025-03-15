mod purge;
mod warn;
mod set_warn_channel;
mod set_delete_log_channel;
mod set_url_rule;
mod update_raid_time;
mod toggle_emoji_reactions;
mod set_reaction_log;
mod set_level_up_channel;
pub mod add_role_buttons;

use crate::error::Error;
use crate::Data;

// Re-export with category = "Admin" explicitly set
pub fn warn() -> poise::Command<Data, Error> {
    let mut cmd = warn::warn();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn setwarnchannel() -> poise::Command<Data, Error> {
    let mut cmd = set_warn_channel::setwarnchannel();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn setdeletemessagechannel() -> poise::Command<Data, Error> {
    let mut cmd = set_delete_log_channel::setdeletemessagechannel();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn seturlrule() -> poise::Command<Data, Error> {
    let mut cmd = set_url_rule::seturlrule();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn updateraidtime() -> poise::Command<Data, Error> {
    let mut cmd = update_raid_time::updateraidtime();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn toggleemojireactions() -> poise::Command<Data, Error> {
    let mut cmd = toggle_emoji_reactions::toggleemojireactions();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn reactionslog() -> poise::Command<Data, Error> {
    let mut cmd = set_reaction_log::reactionslog();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn setlevelupchannel() -> poise::Command<Data, Error> {
    let mut cmd = set_level_up_channel::setlevelupchannel();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn rolebuttons() -> poise::Command<Data, Error> {
    let mut cmd = add_role_buttons::rolebuttons();
    cmd.category = Some("Admin".to_string());
    cmd
}

pub fn purge() -> poise::Command<Data, Error> {
    let mut cmd = purge::purge();
    cmd.category = Some("Admin".to_string());
    cmd
}
