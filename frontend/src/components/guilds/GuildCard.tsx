import React from 'react';
import styled from 'styled-components';
import { Guild } from '../../types';

const Card = styled.div`
  background-color: #2f3136;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s, box-shadow 0.2s;
  
  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
`;

const Header = styled.div`
  display: flex;
  align-items: center;
  margin-bottom: 12px;
`;

const GuildIcon = styled.div<{ iconUrl?: string }>`
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background-color: #5865F2;
  background-image: ${({ iconUrl }) => (iconUrl ? `url(${iconUrl})` : 'none')};
  background-size: cover;
  background-position: center;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: bold;
  margin-right: 16px;
  flex-shrink: 0;
`;

const GuildInfo = styled.div`
  flex: 1;
`;

const GuildName = styled.h3`
  margin: 0 0 4px 0;
  font-size: 18px;
  color: #ffffff;
`;

const GuildMeta = styled.div`
  font-size: 14px;
  color: #b9bbbe;
`;

const Stats = styled.div`
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
`;

const StatItem = styled.div`
  flex: 1;
  background-color: #36393f;
  padding: 12px;
  border-radius: 4px;
  text-align: center;
`;

const StatValue = styled.div`
  font-size: 16px;
  font-weight: bold;
  color: #ffffff;
  margin-bottom: 4px;
`;

const StatLabel = styled.div`
  font-size: 12px;
  color: #b9bbbe;
`;

const Actions = styled.div`
  display: flex;
  gap: 8px;
`;

const ActionButton = styled.button<{ variant?: 'primary' | 'secondary' | 'danger' }>`
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
  border: none;
  
  background-color: ${({ variant }) => {
    switch (variant) {
      case 'primary': return '#5865F2';
      case 'secondary': return '#4f545c';
      case 'danger': return '#ED4245';
      default: return '#4f545c';
    }
  }};
  
  color: white;
  
  &:hover {
    background-color: ${({ variant }) => {
      switch (variant) {
        case 'primary': return '#4752c4';
        case 'secondary': return '#5d6269';
        case 'danger': return '#c9383a';
        default: return '#5d6269';
      }
    }};
  }
`;

interface GuildCardProps {
  guild: Guild;
  onManage: (guildId: string) => void;
  onSettings: (guildId: string) => void;
  onCommands: (guildId: string) => void;
  onWordDetection: (guildId: string) => void;
}

// Helper function to convert Discord icon hash to URL
const getIconUrl = (guildId: string, iconHash?: string) => {
  if (!iconHash) return undefined;
  return `https://cdn.discordapp.com/icons/${guildId}/${iconHash}.png`;
};

// Get initial letter of guild name for avatar fallback
const getInitial = (name: string) => name && name.length > 0 ? name.charAt(0).toUpperCase() : '?';

const GuildCard: React.FC<GuildCardProps> = ({ 
  guild, 
  onManage, 
  onSettings, 
  onCommands,
  onWordDetection 
}) => {
  return (
    <Card>
      <Header>
        <GuildIcon iconUrl={getIconUrl(guild.id, guild.icon)}>
          {!guild.icon && getInitial(guild.name)}
        </GuildIcon>
        <GuildInfo>
          <GuildName>{guild.name || 'Unnamed Server'}</GuildName>
          <GuildMeta>{guild.botJoined ? 'Bot is active' : 'Bot not joined'}</GuildMeta>
        </GuildInfo>
      </Header>
      
      <Stats>
        <StatItem>
          <StatValue>{guild.memberCount || '?'}</StatValue>
          <StatLabel>Members</StatLabel>
        </StatItem>
        <StatItem>
          <StatValue>{guild.commandsEnabled || 0}</StatValue>
          <StatLabel>Commands</StatLabel>
        </StatItem>
        <StatItem>
          <StatValue>{guild.wordRules || 0}</StatValue>
          <StatLabel>Word Rules</StatLabel>
        </StatItem>
      </Stats>
      
      <Actions>
        <ActionButton 
          variant="primary" 
          onClick={() => onManage(guild.id)}
        >
          Manage
        </ActionButton>
        <ActionButton 
          variant="secondary" 
          onClick={() => onCommands(guild.id)}
        >
          Commands
        </ActionButton>
        <ActionButton 
          variant="secondary" 
          onClick={() => onWordDetection(guild.id)}
        >
          Word Rules
        </ActionButton>
        <ActionButton 
          variant="secondary" 
          onClick={() => onSettings(guild.id)}
        >
          Settings
        </ActionButton>
      </Actions>
    </Card>
  );
};

export default GuildCard;
