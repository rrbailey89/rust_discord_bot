// User related types
export interface User {
  id: string;
  username: string;
  avatar_url?: string;
  guilds: Guild[];
}

// Guild related types
export interface Guild {
  id: string;
  name: string;
  icon?: string;
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

// Settings types
export interface GuildSettings {
  guild_id?: string;
  id?: string;
  name?: string;
  settings?: Record<string, any>;
  prefix: string;
  logChannelId: string;
  moderationEnabled: boolean;
  autoModeration: {
    enabled: boolean;
    filterLinks: boolean;
    filterInvites: boolean;
    filterProfanity: boolean;
  };
  welcomeMessage: {
    enabled: boolean;
    channelId: string;
    message: string;
  };
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
