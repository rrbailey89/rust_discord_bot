import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import styled from 'styled-components';
import { useQuery } from '@tanstack/react-query';
import { Guild } from '../../types';
import GuildCard from './GuildCard';
import api from '../../services/api';

const Container = styled.div`
  padding: 16px;
`;

const Header = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
`;

const Title = styled.h2`
  color: #ffffff;
  margin: 0;
`;

const SearchContainer = styled.div`
  position: relative;
  width: 300px;
`;

const SearchInput = styled.input`
  width: 100%;
  padding: 10px 16px;
  background-color: #36393f;
  border: 1px solid #202225;
  border-radius: 4px;
  color: white;
  font-size: 14px;

  &:focus {
    outline: none;
    border-color: #5865F2;
  }

  &::placeholder {
    color: #72767d;
  }
`;

const SearchIcon = styled.span`
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: #72767d;
`;

const EmptyState = styled.div`
  text-align: center;
  padding: 40px 0;
  color: #b9bbbe;
`;

const LoadingContainer = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 40px 0;
`;

const ErrorContainer = styled.div`
  background-color: rgba(237, 66, 69, 0.1);
  border-left: 4px solid #ED4245;
  padding: 16px;
  margin-bottom: 24px;
  color: #ffffff;
`;

const ErrorMessage = styled.div`
  margin-bottom: 8px;
  font-weight: 500;
`;

const RetryButton = styled.button`
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

// Function to fetch guilds from the API
const fetchGuilds = async (): Promise<Guild[]> => {
  const response = await api.get('/guilds');
  return response.data;
};

const GuildList: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const navigate = useNavigate();
  
  // Fetch guilds data
  const { data: guilds, isLoading, isError, error, refetch } = useQuery<Guild[], Error>({
    queryKey: ['guilds'],
    queryFn: fetchGuilds,
  });
  
  // Filter guilds based on search term
  const filteredGuilds = guilds?.filter(guild => 
    guild.name && guild.name.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];
  
  // Navigation handlers
  const handleManage = (guildId: string) => {
    navigate(`/guilds/${guildId}`);
  };
  
  const handleSettings = (guildId: string) => {
    navigate(`/guilds/${guildId}/settings`);
  };
  
  const handleCommands = (guildId: string) => {
    navigate(`/guilds/${guildId}/commands`);
  };
  
  const handleWordDetection = (guildId: string) => {
    navigate(`/guilds/${guildId}/word-detection`);
  };

  // Render loading state
  if (isLoading) {
    return (
      <LoadingContainer>
        <div className="loader"></div>
      </LoadingContainer>
    );
  }

  // Render error state
  if (isError) {
    return (
      <Container>
        <ErrorContainer>
          <ErrorMessage>Failed to load guilds</ErrorMessage>
          <div>{error?.message || 'An unknown error occurred'}</div>
          <RetryButton onClick={() => refetch()}>Retry</RetryButton>
        </ErrorContainer>
      </Container>
    );
  }

  return (
    <Container>
      <Header>
        <Title>Your Servers</Title>
        <SearchContainer>
          <SearchInput
            type="text"
            placeholder="Search servers..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
          <SearchIcon>🔍</SearchIcon>
        </SearchContainer>
      </Header>

      {filteredGuilds.length > 0 ? (
        filteredGuilds.map(guild => (
          <GuildCard
            key={guild.id}
            guild={guild}
            onManage={handleManage}
            onSettings={handleSettings}
            onCommands={handleCommands}
            onWordDetection={handleWordDetection}
          />
        ))
      ) : (
        <EmptyState>
          {searchTerm 
            ? 'No servers found matching your search' 
            : 'No servers available. You need administrator permissions on a server with the bot added to manage it.'}
        </EmptyState>
      )}
    </Container>
  );
};

export default GuildList;
