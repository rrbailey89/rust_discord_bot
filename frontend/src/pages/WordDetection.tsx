import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useQuery } from '@tanstack/react-query';
import { Guild, WordDetectionRule } from '../types';
import WordDetectionRuleList from '../components/word-detection/WordDetectionRuleList';
import WordDetectionRuleEditor from '../components/word-detection/WordDetectionRuleEditor';
import WordDetectionRuleTester from '../components/word-detection/WordDetectionRuleTester';
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
  margin: 0 0 8px 0;
  color: #dcddde;
  font-size: 14px;
  line-height: 1.5;
  
  &:last-child {
    margin-bottom: 0;
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

// Fetch guild details
const fetchGuildDetails = async (guildId: string): Promise<Guild> => {
  const response = await api.get(`/api/guilds/${guildId}`);
  return response.data;
};

const WordDetection: React.FC = () => {
  const { guildId } = useParams<{ guildId: string }>();
  const [editingRule, setEditingRule] = useState<WordDetectionRule | null>(null);
  const [isCreatingRule, setIsCreatingRule] = useState(false);
  const [testingRule, setTestingRule] = useState<WordDetectionRule | null>(null);
  
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
  
  // Handle edit rule
  const handleEditRule = (rule: WordDetectionRule) => {
    setEditingRule(rule);
  };
  
  // Handle create new rule
  const handleNewRule = () => {
    setIsCreatingRule(true);
  };
  
  // Handle test rule
  const handleTestRule = (rule: WordDetectionRule) => {
    setTestingRule(rule);
  };
  
  // Handle close editors/testers
  const handleCloseEditor = () => {
    setEditingRule(null);
    setIsCreatingRule(false);
  };
  
  const handleCloseTester = () => {
    setTestingRule(null);
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
          <div>{error?.message || 'An unknown error occurred'}</div>
          <RetryButton onClick={() => refetch()}>Retry</RetryButton>
        </ErrorContainer>
      </Container>
    );
  }
  
  return (
    <Container>
      <Header>
        <HeaderInfo>
          <Title>Word Detection</Title>
          <Subtitle>Configure word detection rules for {guild.name}</Subtitle>
        </HeaderInfo>
        <RefreshButton onClick={() => refetch()}>
          <ButtonIcon>🔄</ButtonIcon>
          Refresh
        </RefreshButton>
      </Header>
      
      <InfoCard>
        <InfoTitle>About Word Detection</InfoTitle>
        <InfoText>
          Word detection rules allow you to automatically detect and take action on messages
          containing specific words or patterns. Use this to moderate your Discord server more effectively.
        </InfoText>
        <InfoText>
          Each rule consists of a pattern (what to detect) and an action (what to do when detected).
          Patterns can be simple text or advanced regular expressions for more complex matching.
        </InfoText>
      </InfoCard>
      
      <WordDetectionRuleList
        onEdit={handleEditRule}
        onNew={handleNewRule}
        onTest={handleTestRule}
      />
      
      {/* Edit rule modal */}
      {editingRule && (
        <WordDetectionRuleEditor
          guildId={guildId!}
          rule={editingRule}
          onClose={handleCloseEditor}
        />
      )}
      
      {/* New rule modal */}
      {isCreatingRule && (
        <WordDetectionRuleEditor
          guildId={guildId!}
          onClose={handleCloseEditor}
        />
      )}
      
      {/* Test rule modal */}
      {testingRule && (
        <WordDetectionRuleTester
          rule={testingRule}
          onClose={handleCloseTester}
        />
      )}
    </Container>
  );
};

export default WordDetection;
