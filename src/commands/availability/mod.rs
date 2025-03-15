mod unavailable;
mod cancel_unavailable;
mod list_unavailable;
mod set_unavailability_channel;
mod iscaliforniaonfire;
mod whereiscaliforniaonfire;

use crate::error::Error;
use crate::Data;

// Re-export with category = "Availability" explicitly set
pub fn unavailable() -> poise::Command<Data, Error> {
    let mut cmd = unavailable::unavailable();
    cmd.category = Some("Availability".to_string());
    cmd
}

pub fn cancelunavailable() -> poise::Command<Data, Error> {
    let mut cmd = cancel_unavailable::cancelunavailable();
    cmd.category = Some("Availability".to_string());
    cmd
}

pub fn listunavailable() -> poise::Command<Data, Error> {
    let mut cmd = list_unavailable::listunavailable();
    cmd.category = Some("Availability".to_string());
    cmd
}

pub fn setunavailabilitychannel() -> poise::Command<Data, Error> {
    let mut cmd = set_unavailability_channel::setunavailabilitychannel();
    cmd.category = Some("Availability".to_string());
    cmd
}

pub fn iscaliforniaonfire() -> poise::Command<Data, Error> {
    let mut cmd = iscaliforniaonfire::iscaliforniaonfire();
    cmd.category = Some("Utility".to_string());  // Changed from "Availability" to "Utility"
    cmd
}

pub fn whereiscaliforniaonfire() -> poise::Command<Data, Error> {
    let mut cmd = whereiscaliforniaonfire::whereiscaliforniaonfire();
    cmd.category = Some("Utility".to_string());  // Changed from "Availability" to "Utility"
    cmd
}
