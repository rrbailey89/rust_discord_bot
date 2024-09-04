// commands.rs
mod warn;
mod set_warn_channel;
mod random_cat_image;
mod random_capy_image;
mod user_info;
mod anime_hug;
mod update_raid_time;
mod ask;

use crate::error::Error;
use crate::Data;

type Context<'a> = poise::Context<'a, Data, Error>;

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        warn::warn(),
        set_warn_channel::set_warn_channel(),
        random_cat_image::random_cat_image(),
        random_capy_image::random_capy_image(),
        user_info::user_info(),
        anime_hug::anime_hug(),
        update_raid_time::update_raid_time(),
        ask::ask(),
    ]
}