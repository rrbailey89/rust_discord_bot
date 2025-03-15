mod help;
mod is_alive;
mod ping;
mod rules;
mod user_info;
mod weather;
pub mod reminder;
mod relay;
mod ask;
mod health;
mod iscaliforniaonfire;
mod whereiscaliforniaonfire;

use crate::error::Error;
use crate::Data;

// Re-export with category = "Utility" explicitly set
pub fn help() -> poise::Command<Data, Error> {
    // Help command already has category defined in the file, but we'll set it here for consistency
    let mut cmd = help::help();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn lifecheck() -> poise::Command<Data, Error> {
    let mut cmd = is_alive::lifecheck();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn ping() -> poise::Command<Data, Error> {
    let mut cmd = ping::ping();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn rule() -> poise::Command<Data, Error> {
    let mut cmd = rules::rule();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn userinfo() -> poise::Command<Data, Error> {
    let mut cmd = user_info::userinfo();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn weather() -> poise::Command<Data, Error> {
    let mut cmd = weather::weather();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn reminder() -> poise::Command<Data, Error> {
    let mut cmd = reminder::reminder();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn relay() -> poise::Command<Data, Error> {
    let mut cmd = relay::relay();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn ask() -> poise::Command<Data, Error> {
    let mut cmd = ask::ask();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn dbhealth() -> poise::Command<Data, Error> {
    let mut cmd = health::dbhealth();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn dbschema() -> poise::Command<Data, Error> {
    let mut cmd = health::dbschema();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn iscaliforniaonfire() -> poise::Command<Data, Error> {
    let mut cmd = iscaliforniaonfire::iscaliforniaonfire();
    cmd.category = Some("Utility".to_string());
    cmd
}

pub fn whereiscaliforniaonfire() -> poise::Command<Data, Error> {
    let mut cmd = whereiscaliforniaonfire::whereiscaliforniaonfire();
    cmd.category = Some("Utility".to_string());
    cmd
}
