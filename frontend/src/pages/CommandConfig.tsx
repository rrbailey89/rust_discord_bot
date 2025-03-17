import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useQuery } from '@tanstack/react-query';
import { Command, Guild } from '../types';
import CommandList from '../components/commands/CommandList';
import CommandSettings from '../components/commands/CommandSettings';
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

const HeaderInfo = styled.div`
  display: flex;
  flex-direction: column;
`;

const Title = styled.h1`
  color: #ffffff;
  margin: 0 0 8px 0;
  font-size: 24px;
`;

const Subtitle = styled.p`
  color: #b9bbbe;
  margin: 0;
  font-size: 16px;
`;

const RefreshButton = styled.button`
  padding: 8px 16px;
  background-color: #4f545c;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  
  &:hover {
    background-color: #5d6269;
  }
  
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;

const ButtonIcon = styled.span`
  margin-right: 8px;
`;

const InfoCard = styled.div`
  background-color: #2f3136;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
  border-left: 4px solid #5865F2;
`;

const InfoTitle = styled.h3`
  margin: 0 0 8px 0;
  color: #ffffff;
  font-size: 16px;
`;

const InfoText = styled.p`
  margin: 0;
  color: #dcddde;
  font-size: 14px;
  line-height: 1.5;
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

// Fetch guild details
const fetchGuildDetails = async (guildId: string): Promise<Guild> => {
  const response = await api.get(`/api/guilds/${guildId}`);
  return response.data;
};

const CommandConfig: React.FC = () => {
  const { guildId } = useParams<{ guildId: string }>();
  const [selectedCommand, setSelectedCommand] = useState<Command | null>(null);
  
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
  
  // Handle command settings open
  const handleOpenSettings = (command: Command) => {
    setSelectedCommand(command);
  };
  
  // Handle command settings close
  const handleCloseSettings = () => {
    setSelectedCommand(null);
  };
  
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
          <div>{error instanceof Error ? error.message : 'An unknown error occurred'}</div>
          <RetryButton onClick={() => refetch()}>Retry</RetryButton>
        </ErrorContainer>
      </Container>
    );
  }
  
  return (
    <Container>
      <Header>
        <HeaderInfo>
          <Title>Command Configuration</Title>
          <Subtitle>Manage bot commands for {guild.name}</Subtitle>
        </HeaderInfo>
        <RefreshButton onClick={() => refetch()}>
          <ButtonIcon>🔄</ButtonIcon>
          Refresh
        </RefreshButton>
      </Header>
      
      <InfoCard>
        <InfoTitle>About Commands</InfoTitle>
        <InfoText>
          Enable or disable specific commands for your server, and configure their settings.
          Commands can be grouped by category, and each command can have its own specific settings.
        </InfoText>
      </InfoCard>
      
      <CommandList onOpenSettings={handleOpenSettings} />
      
      {selectedCommand && (
        <CommandSettings
          command={selectedCommand}
          guildId={guildId!}
          onClose={handleCloseSettings}
        />
      )}
    </Container>
  );
};

export default CommandConfig;
