// src/web/services/mod.rs
//! Service implementations for the web server

pub mod auth;
pub mod discord;
pub mod guild;
pub mod command;
pub mod rule;
pub mod settings;
pub mod analytics;

pub use auth::AuthService;
pub use discord::DiscordService;
pub use guild::GuildService;
pub use command::CommandService;
pub use rule::RuleService;
pub use settings::SettingsService;
pub use analytics::AnalyticsService;
