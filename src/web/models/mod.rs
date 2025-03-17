// src/web/models/mod.rs
//! Data models for the web server

pub mod auth;
pub mod guild;
pub mod command;
pub mod rule;
pub mod settings;
pub mod analytics;

pub use auth::{
    UserSession, DiscordUser, DiscordTokenResponse,
    TokenClaims, UserInfo, AuthResponse
};

pub use guild::{
    GuildInfo, GuildDetails, ChannelInfo, GuildSettings,
    UpdateGuildSettingsRequest, GuildResponse
};

pub use command::{
    CommandInfo, CommandDetails, ConfigSchema, ConfigOption,
    EnumValue, CommandSettings, UpdateCommandSettingsRequest,
    CommandResponse
};

pub use rule::{
    RuleInfo, CreateRuleRequest, UpdateRuleRequest, RuleResponse,
    TestRuleRequest, TestRuleResponse, RuleAction
};

pub use settings::{
    GlobalSettings, UserPreferences, 
    UpdateGlobalSettingsRequest, UpdateUserPreferencesRequest, 
    SettingsResponse
};

pub use analytics::{
    AnalyticsEvent, GuildAnalyticsSummary, CommandUsage,
    UserActivitySummary, GuildActivity, LogEventRequest,
    AnalyticsQueryParams, AnalyticsResponse
};
