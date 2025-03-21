import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useQuery } from '@tanstack/react-query';
import { Command } from '../../types';
import CommandToggleSimple from './CommandToggleSimple';
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

const SearchBox = styled.div`
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

const CategorySection = styled.div`
  margin-bottom: 24px;
`;

const CategoryHeader = styled.div`
  padding: 12px 16px;
  background-color: #2f3136;
  border-radius: 4px;
  margin-bottom: 12px;
  font-weight: 600;
  color: #ffffff;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
`;

const CategoryIcon = styled.span`
  margin-right: 8px;
`;

const CommandGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
`;

const CommandCard = styled.div`
  background-color: #36393f;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #202225;
  transition: transform 0.2s, box-shadow 0.2s;
  
  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
`;

const CommandHeader = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
`;

const CommandName = styled.h3`
  margin: 0;
  font-size: 16px;
  color: #ffffff;
`;

const CommandDescription = styled.p`
  margin: 0 0 16px 0;
  font-size: 14px;
  color: #b9bbbe;
  line-height: 1.4;
`;

const CommandActions = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

const SettingsButton = styled.button`
  background-color: #4f545c;
  color: white;
  border: none;
  border-radius: 4px;
  padding: 6px 12px;
  font-size: 12px;
  cursor: pointer;
  
  &:hover {
    background-color: #5d6269;
  }
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

const RetryButton = styled.button`
  margin-top: 8px;
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

// Function to fetch commands from the API
const fetchCommands = async (guildId: string): Promise<Command[]> => {
  try {
    console.log(`Fetching commands for guild ${guildId}`);
    
    // Call the direct endpoint instead of using the api service
    // to avoid the double /api prefix issue
    const response = await fetch(`/api/commands/guild/${guildId}`, {
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bot ${localStorage.getItem('auth_token')}`,
      },
    });
    
    if (!response.ok) {
      throw new Error(`API request failed with status ${response.status}`);
    }
    
    const data = await response.json();
    console.log("Commands data received:", data);
    return data;
  } catch (error) {
    console.error("Error fetching commands:", error);
    // Return empty array instead of propagating error
    return [];
  }
};

// Group commands by category
const groupCommandsByCategory = (commands: Command[]) => {
  const categories: Record<string, Command[]> = {};
  
  commands.forEach(command => {
    const category = command.category || 'Uncategorized';
    if (!categories[category]) {
      categories[category] = [];
    }
    categories[category].push(command);
  });
  
  return categories;
};

interface CommandListProps {
  onOpenSettings: (command: Command) => void;
}

const CommandList: React.FC<CommandListProps> = ({ onOpenSettings }) => {
  const { guildId } = useParams<{ guildId: string }>();
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedCategories, setExpandedCategories] = useState<Record<string, boolean>>({});
  
  // Fetch commands data
  const { 
    data: commands, 
    isLoading, 
    isError, 
    error, 
    refetch 
  } = useQuery<Command[], Error>({
    queryKey: ['commands', guildId],
    queryFn: () => fetchCommands(guildId!),
    enabled: !!guildId,
  });
  
  // Filter commands based on search term
  const filteredCommands = commands?.filter(command => 
    (command.name?.toLowerCase() || '').includes(searchTerm.toLowerCase()) ||
    (command.description?.toLowerCase() || '').includes(searchTerm.toLowerCase())
  ) || [];
  
  // Handle toggle click
  const handleToggleCategory = (category: string) => {
    setExpandedCategories(prev => ({
      ...prev,
      [category]: !prev[category]
    }));
  };
  
  // Get initial state for categories - all expanded by default
  React.useEffect(() => {
    if (commands) {
      const categories = Object.keys(groupCommandsByCategory(commands));
      const initialState: Record<string, boolean> = {};
      categories.forEach(category => {
        initialState[category] = true; // Set all to expanded by default
      });
      setExpandedCategories(initialState);
    }
  }, [commands]);
  
  // Render loading state
  if (isLoading) {
    return (
      <LoadingContainer>
        <div>Loading commands...</div>
      </LoadingContainer>
    );
  }
  
  // Render error state
  if (isError) {
    return (
      <Container>
        <ErrorContainer>
          <div>Failed to load commands: {error instanceof Error ? error.message : 'Unknown error'}</div>
          <RetryButton onClick={() => refetch()}>Retry</RetryButton>
        </ErrorContainer>
      </Container>
    );
  }
  
  // Group commands by category for display
  const commandsByCategory = groupCommandsByCategory(filteredCommands);
  const categories = Object.keys(commandsByCategory);
  
  return (
    <Container>
      <Header>
        <Title>Server Commands</Title>
        <SearchBox>
          <SearchInput
            type="text"
            placeholder="Search commands..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
          <SearchIcon>🔍</SearchIcon>
        </SearchBox>
      </Header>
      
      {categories.length === 0 ? (
        <EmptyState>
          {searchTerm 
            ? 'No commands found matching your search' 
            : 'No commands available for this server.'}
        </EmptyState>
      ) : (
        categories.map(category => (
          <CategorySection key={category}>
            <CategoryHeader onClick={() => handleToggleCategory(category)}>
              <div>
                <CategoryIcon>{expandedCategories[category] ? '▼' : '►'}</CategoryIcon>
                {category}
              </div>
              <div>{commandsByCategory[category].length} commands</div>
            </CategoryHeader>
            
            {expandedCategories[category] && (
              <CommandGrid>
                {commandsByCategory[category].map(command => (
                  <CommandCard key={command.id}>
                    <CommandHeader>
                      <CommandName>{command.name || 'Unnamed Command'}</CommandName>
                      <CommandToggleSimple
                        commandId={command.id}
                        guildId={guildId!}
                        initialEnabled={command.enabled}
                      />
                    </CommandHeader>
                    <CommandDescription>
                      {command.description || 'No description available'}
                    </CommandDescription>
                    <CommandActions>
                      <div></div>
                      <SettingsButton onClick={() => onOpenSettings(command)}>
                        Settings
                      </SettingsButton>
                    </CommandActions>
                  </CommandCard>
                ))}
              </CommandGrid>
            )}
          </CategorySection>
        ))
      )}
    </Container>
  );
};

export default CommandList;
