// User related types
export interface User {
  id: string;
  username: string;
  avatar_url?: string;
  guilds: Guild[];
  // New fields from expanded OAuth scopes
  email?: string;
  verified?: boolean;
  locale?: string;
}

// Guild related types
export interface Guild {
  id: string;
  name: string;
  icon?: string;
  icon_url?: string;
  owner: boolean;
  permissions: number;
  botJoined?: boolean;
  memberCount?: number;
  commandsEnabled?: number;
  wordRules?: number;
}

// Authentication related types
export interface AuthResponse {
  token: string;
  expires_in: number;
  user: User;
}

// Command related types
export interface Command {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  category?: string;
  settings?: Record<string, any>;
}

// Word detection rule types
export interface WordDetectionRule {
  id: number;
  guild_id: string;
  pattern: string;
  action: string;
  action_params: Record<string, any>;
  created_at: string;
  updated_at: string;
}

// Settings types - Updated to match backend structure
export interface GuildSettings {
  guild_id: number; // Changed from string? to number
  prefix?: string | null; // Changed to optional string or null
  mod_role_id?: number | null; // Changed to optional number or null
  admin_role_id?: number | null; // Changed to optional number or null
  settings?: { // Nested settings object
    autoModeration?: { // Optional nested structure
      enabled?: boolean;
      filterLinks?: boolean;
      filterInvites?: boolean;
      filterProfanity?: boolean;
    };
    welcomeMessage?: { // Optional nested structure
      enabled?: boolean;
      channelId?: number | null; // Changed to optional number or null
      message?: string;
    };
    // Add other potential nested settings here if needed
    [key: string]: any; // Allow other arbitrary settings
  } | null;
  emoji_reactions_enabled?: boolean | null; // Added new field
  level_up_channel_id?: number | null; // Added new field, type number
  warn_channel_id?: number | null; // Added new field, type number
  url_rule?: string | null; // Added new field
  delete_log_channel_id?: number | null; // Added new field, type number
  reaction_log_channel_id?: number | null; // Added new field, type number

  // Frontend-specific state (might need adjustment based on how backend sends data)
  logChannelId?: string; // Kept for now, might need removal/update
  moderationEnabled?: boolean; // Kept for now, might need removal/update
}

// Analytics types
export interface AnalyticsEvent {
  id: number;
  event_type: string;
  user_id?: string;
  guild_id?: string;
  event_data: Record<string, any>;
  timestamp: string;
}

// API Error type
export interface ApiError {
  message: string;
  code?: string;
  status?: number;
}
