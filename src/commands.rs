// commands.rs
pub mod admin;
pub mod fun;
pub mod utility;
pub mod availability;

use crate::error::Error;
use crate::Data;
use std::collections::HashMap;

// Command scope for registration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandScope {
    Global,  // Available everywhere
    Guild,   // Only available in specific guilds when enabled
}

// Command configuration
#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub scope: CommandScope,
    pub name: &'static str,
    pub cooldown: Option<u64>, // cooldown in seconds
    pub description: &'static str,
}

// Get command configurations
pub fn get_command_config() -> HashMap<&'static str, CommandConfig> {
    let mut configs = HashMap::new();
    
    // Global commands (available everywhere)
    configs.insert("ping", CommandConfig { 
        scope: CommandScope::Global, 
        name: "ping",
        cooldown: None,
        description: "Check bot latency",
    });
    configs.insert("help", CommandConfig { 
        scope: CommandScope::Global, 
        name: "help",
        cooldown: None,
        description: "Show command help",
    });
    
    // "blame" command moved to guild commands (no longer global)
    
    // Admin commands
    configs.insert("warn", CommandConfig {
        scope: CommandScope::Guild,
        name: "warn",
        cooldown: Some(5),
        description: "Warn a user",
    });
    configs.insert("setwarnchannel", CommandConfig {
        scope: CommandScope::Guild,
        name: "setwarnchannel",
        cooldown: None,
        description: "Set the warn channel",
    });
    configs.insert("updateraidtime", CommandConfig {
        scope: CommandScope::Guild,
        name: "updateraidtime",
        cooldown: None,
        description: "Update raid time",
    });
    configs.insert("purge", CommandConfig {
        scope: CommandScope::Guild,
        name: "purge",
        cooldown: Some(10),
        description: "Purge messages",
    });
    configs.insert("setdeletemessagechannel", CommandConfig {
        scope: CommandScope::Guild,
        name: "setdeletemessagechannel",
        cooldown: None,
        description: "Set delete message log channel",
    });
    configs.insert("toggleemojireactions", CommandConfig {
        scope: CommandScope::Guild,
        name: "toggleemojireactions",
        cooldown: None,
        description: "Toggle emoji reactions",
    });
    configs.insert("setlevelupchannel", CommandConfig {
        scope: CommandScope::Guild,
        name: "setlevelupchannel",
        cooldown: None,
        description: "Set level up channel",
    });
    configs.insert("rolebuttons", CommandConfig {
        scope: CommandScope::Guild,
        name: "rolebuttons",
        cooldown: None,
        description: "Create role buttons",
    });
    configs.insert("seturlrule", CommandConfig {
        scope: CommandScope::Guild,
        name: "seturlrule",
        cooldown: None,
        description: "Set URL rule",
    });
    configs.insert("reactionslog", CommandConfig {
        scope: CommandScope::Guild,
        name: "reactionslog",
        cooldown: None,
        description: "Set reactions log channel",
    });
    
    // Fun commands
    configs.insert("randomcatimage", CommandConfig {
        scope: CommandScope::Guild,
        name: "randomcatimage",
        cooldown: Some(5),
        description: "Get a random cat image",
    });
    configs.insert("randomcapyimage", CommandConfig {
        scope: CommandScope::Guild,
        name: "randomcapyimage",
        cooldown: Some(5),
        description: "Get a random capybara image",
    });
    configs.insert("animehug", CommandConfig {
        scope: CommandScope::Guild,
        name: "animehug",
        cooldown: Some(5),
        description: "Send an anime hug",
    });
    configs.insert("createimage", CommandConfig {
        scope: CommandScope::Guild,
        name: "createimage",
        cooldown: Some(30),
        description: "Create an AI image",
    });
    configs.insert("blame", CommandConfig { 
        scope: CommandScope::Guild, 
        name: "blame",
        cooldown: None,
        description: "Blame Serena for something",
    });
    configs.insert("fluximage", CommandConfig {
        scope: CommandScope::Guild,
        name: "fluximage",
        cooldown: Some(5),
        description: "Get a flux image",
    });
    
    // Utility commands
    configs.insert("userinfo", CommandConfig {
        scope: CommandScope::Guild,
        name: "userinfo",
        cooldown: Some(5),
        description: "Get user information",
    });
    configs.insert("ask", CommandConfig {
        scope: CommandScope::Guild,
        name: "ask",
        cooldown: Some(10),
        description: "Ask a question",
    });
    configs.insert("reminder", CommandConfig {
        scope: CommandScope::Guild,
        name: "reminder",
        cooldown: Some(5),
        description: "Set a reminder",
    });
    configs.insert("weather", CommandConfig {
        scope: CommandScope::Guild,
        name: "weather",
        cooldown: Some(10),
        description: "Get weather information",
    });
    configs.insert("rule", CommandConfig {
        scope: CommandScope::Guild,
        name: "rule",
        cooldown: None,
        description: "Display server rules",
    });
    configs.insert("lifecheck", CommandConfig {
        scope: CommandScope::Guild,
        name: "lifecheck",
        cooldown: None,
        description: "Check if the bot is alive",
    });
    configs.insert("relay", CommandConfig {
        scope: CommandScope::Guild,
        name: "relay",
        cooldown: None,
        description: "Relay a message",
    });
    configs.insert("dbhealth", CommandConfig {
        scope: CommandScope::Guild,
        name: "dbhealth",
        cooldown: None,
        description: "Check database health",
    });
    configs.insert("dbschema", CommandConfig {
        scope: CommandScope::Guild,
        name: "dbschema",
        cooldown: None,
        description: "Show database schema",
    });
    
    // Availability commands
    configs.insert("setunavailabilitychannel", CommandConfig {
        scope: CommandScope::Guild,
        name: "setunavailabilitychannel",
        cooldown: None,
        description: "Set unavailability channel",
    });
    configs.insert("unavailable", CommandConfig {
        scope: CommandScope::Guild,
        name: "unavailable",
        cooldown: None,
        description: "Mark yourself as unavailable",
    });
    configs.insert("listunavailable", CommandConfig {
        scope: CommandScope::Guild,
        name: "listunavailable",
        cooldown: None,
        description: "List unavailable users",
    });
    configs.insert("cancelunavailable", CommandConfig {
        scope: CommandScope::Guild,
        name: "cancelunavailable",
        cooldown: None,
        description: "Cancel unavailability",
    });
    
    // Additional Utility commands
    configs.insert("iscaliforniaonfire", CommandConfig {
        scope: CommandScope::Guild,
        name: "iscaliforniaonfire",
        cooldown: Some(60),
        description: "Is California on fire?",
    });
    configs.insert("whereiscaliforniaonfire", CommandConfig {
        scope: CommandScope::Guild,
        name: "whereiscaliforniaonfire",
        cooldown: Some(60),
        description: "Where is California on fire?",
    });
    
    configs
}

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
