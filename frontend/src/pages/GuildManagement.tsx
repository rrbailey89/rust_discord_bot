import React, { useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import styled from 'styled-components';
import { useQuery } from '@tanstack/react-query';
import { Guild } from '../types';
import GuildSettings from '../components/guilds/GuildSettings';
import api from '../services/api';

const Container = styled.div`
  padding: 24px;
`;

const Header = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
`;

const GuildInfo = styled.div`
  display: flex;
  align-items: center;
`;

const GuildIcon = styled.div<{ iconUrl?: string }>`
  width: 64px;
  height: 64px;
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

const GuildName = styled.h1`
  margin: 0 0 4px 0;
  color: #ffffff;
  font-size: 24px;
`;

const GuildMeta = styled.div`
  color: #b9bbbe;
  font-size: 14px;
`;

const ActionButtons = styled.div`
  display: flex;
  gap: 12px;
`;

const ActionButton = styled.button<{ variant?: 'primary' | 'secondary' | 'danger' }>`
  padding: 8px 16px;
  background-color: ${({ variant }) => {
    switch (variant) {
      case 'primary': return '#5865F2';
      case 'secondary': return '#4f545c';
      case 'danger': return '#ED4245';
      default: return '#4f545c';
    }
  }};
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  
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

const ButtonIcon = styled.span`
  margin-right: 8px;
`;

const TabMenu = styled.div`
  display: flex;
  border-bottom: 1px solid #40444b;
  margin-bottom: 24px;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  
  &::-webkit-scrollbar {
    height: 4px;
  }
  
  &::-webkit-scrollbar-track {
    background: #2e3136;
  }
  
  &::-webkit-scrollbar-thumb {
    background-color: #202225;
    border-radius: 2px;
  }
`;

const Tab = styled.button<{ active: boolean }>`
  padding: 12px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid ${({ active }) => (active ? '#5865F2' : 'transparent')};
  color: ${({ active }) => (active ? '#ffffff' : '#b9bbbe')};
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  
  &:hover {
    color: #ffffff;
  }
`;

const LoadingContainer = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
`;

const ErrorContainer = styled.div`
  background-color: rgba(237, 66, 69, 0.1);
  border-left: 4px solid #ED4245;
  padding: 16px;
  margin-bottom: 24px;
  color: #ffffff;
`;

const RetryButton = styled.button`
  margin-top: 12px;
  padding: 6px 12px;
  background-color: #ED4245;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  
  &:hover {
    background-color: #c9383a;
  }
`;

// Helper function to convert Discord icon hash to URL or use provided URL
const getIconUrl = (guildId: string, iconHash?: string, iconUrl?: string) => {
  if (iconUrl) return iconUrl;
  if (!iconHash) return undefined;
  return `https://cdn.discordapp.com/icons/${guildId}/${iconHash}.png`;
};

// Get initial letter of guild name for avatar fallback
const getInitial = (name: string) => name && name.length > 0 ? name.charAt(0).toUpperCase() : '?';

// Fetch guild details
const fetchGuildDetails = async (guildId: string): Promise<Guild> => {
  // Note: The 'api' instance already has '/api' as its baseURL,
  // so we don't need to include it in the path
  const response = await api.get(`/guilds/${guildId}`);
  return response.data;
};

const GuildManagement: React.FC = () => {
  const { guildId } = useParams<{ guildId: string }>();
  const [activeTab, setActiveTab] = useState<string>('overview');
  const navigate = useNavigate();
  
  // Fetch guild details
  const { 
    data: guild, 
    isLoading, 
    isError, 
    error,
    refetch 
  } = useQuery<Guild, Error>({
    queryKey: ['guild', guildId],
    queryFn: () => fetchGuildDetails(guildId!),
    enabled: !!guildId,
  });
  
  // Handle navigation
  const handleTabChange = (tab: string) => {
    setActiveTab(tab);
    
    switch (tab) {
      case 'commands':
        navigate(`/guilds/${guildId}/commands`);
        break;
      case 'word-detection':
        navigate(`/guilds/${guildId}/word-detection`);
        break;
      case 'settings':
        navigate(`/guilds/${guildId}/settings`);
        break;
      default:
        // Overview tab - stay on the main guild page
        navigate(`/guilds/${guildId}`);
        break;
    }
  };
  
  // Set active tab based on current path when component mounts or route changes
  React.useEffect(() => {
    const path = window.location.pathname;
    if (path.includes('/commands')) {
      setActiveTab('commands');
    } else if (path.includes('/word-detection')) {
      setActiveTab('word-detection');
    } else if (path.includes('/settings')) {
      setActiveTab('settings');
    } else {
      setActiveTab('overview');
    }
  }, [window.location.pathname]);
  
  // Render loading state
  if (isLoading) {
    return (
      <Container>
        <LoadingContainer>
          <div>Loading guild details...</div>
        </LoadingContainer>
      </Container>
    );
  }
  
  // Render error state
  if (isError || !guild) {
    return (
      <Container>
        <ErrorContainer>
          <h3>Failed to load guild details</h3>
          <div>{error?.message || 'An unknown error occurred'}</div>
          <RetryButton onClick={() => refetch()}>Retry</RetryButton>
        </ErrorContainer>
      </Container>
    );
  }
  
  return (
    <Container>
      <Header>
        <GuildInfo>
          <GuildIcon iconUrl={getIconUrl(guild.id, guild.icon, guild.icon_url)}>
            {!guild.icon && !guild.icon_url && getInitial(guild.name)}
          </GuildIcon>
          <div>
            <GuildName>{guild.name || 'Unnamed Server'}</GuildName>
            <GuildMeta>
              {guild.botJoined ? 'Bot is active' : 'Bot not joined'}
              {guild.memberCount && ` • ${guild.memberCount} members`}
            </GuildMeta>
          </div>
        </GuildInfo>
        
        <ActionButtons>
          <ActionButton variant="primary">
            <ButtonIcon>⚙️</ButtonIcon>
            Configure Bot
          </ActionButton>
          {!guild.botJoined && (
            <ActionButton variant="secondary">
              <ButtonIcon>➕</ButtonIcon>
              Add to Server
            </ActionButton>
          )}
        </ActionButtons>
      </Header>
      
      <TabMenu>
        <Tab
          active={activeTab === 'overview'}
          onClick={() => handleTabChange('overview')}
        >
          Overview
        </Tab>
        <Tab
          active={activeTab === 'commands'}
          onClick={() => handleTabChange('commands')}
        >
          Commands
        </Tab>
        <Tab
          active={activeTab === 'word-detection'}
          onClick={() => handleTabChange('word-detection')}
        >
          Word Detection
        </Tab>
        <Tab
          active={activeTab === 'settings'}
          onClick={() => handleTabChange('settings')}
        >
          Settings
        </Tab>
      </TabMenu>
      
      {/* Tab content */}
      {activeTab === 'overview' && (
        <div>
          <h2>Server Overview</h2>
          {/* Overview content will go here */}
          <p>This is the overview of your Discord server. More detailed statistics and information will be displayed here.</p>
        </div>
      )}
      
      {activeTab === 'settings' && <GuildSettings />}
    </Container>
  );
};

export default GuildManagement;
