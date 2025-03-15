// commands.rs
pub mod admin;
pub mod fun;
pub mod utility;
pub mod availability;

use crate::error::Error;
use crate::Data;

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        // Admin commands
        admin::warn(),
        admin::setwarnchannel(),
        admin::updateraidtime(),
        admin::purge(),
        admin::setdeletemessagechannel(),
        admin::toggleemojireactions(),
        admin::setlevelupchannel(),
        admin::rolebuttons(),
        admin::seturlrule(),
        admin::reactionslog(),
        
        // Fun commands
        fun::randomcatimage(),
        fun::randomcapyimage(),
        fun::animehug(),
        fun::createimage(),
        fun::blame(),
        fun::fluximage(),
        
        // Utility commands
        utility::help(),
        utility::userinfo(),
        utility::ask(),
        utility::ping(),
        utility::reminder(),
        utility::weather(),
        utility::rule(),
        utility::lifecheck(),
        utility::relay(),
        utility::dbhealth(),
        utility::dbschema(),
        
        // Availability commands
        availability::setunavailabilitychannel(),
        availability::unavailable(),
        availability::listunavailable(),
        availability::cancelunavailable(),
        
        // Additional Utility commands
        utility::iscaliforniaonfire(),
        utility::whereiscaliforniaonfire(),
    ]
}
