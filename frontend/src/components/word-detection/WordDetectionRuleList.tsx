import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { WordDetectionRule } from '../../types';
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

const RulesTable = styled.table`
  width: 100%;
  border-collapse: collapse;
  background-color: #2f3136;
  border-radius: 8px;
  overflow: hidden;
`;

const TableHead = styled.thead`
  background-color: #202225;
`;

const TableRow = styled.tr`
  border-bottom: 1px solid #40444b;
  
  &:last-child {
    border-bottom: none;
  }

  &:hover {
    background-color: #36393f;
  }
`;

const TableHeader = styled.th`
  padding: 12px 16px;
  text-align: left;
  color: #b9bbbe;
  font-weight: 600;
  font-size: 14px;
`;

const TableCell = styled.td`
  padding: 12px 16px;
  color: #dcddde;
  font-size: 14px;
`;

const PatternCell = styled(TableCell)`
  font-family: monospace;
  background-color: rgba(114, 137, 218, 0.1);
  border-radius: 4px;
`;

const ActionCell = styled(TableCell)`
  text-align: right;
`;

const ActionBadge = styled.span<{ type: string }>`
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  background-color: ${({ type }) => {
    switch (type.toLowerCase()) {
      case 'delete': return 'rgba(237, 66, 69, 0.1)';
      case 'warn': return 'rgba(250, 166, 26, 0.1)';
      case 'mute': return 'rgba(114, 137, 218, 0.1)';
      case 'kick': return 'rgba(250, 166, 26, 0.2)';
      case 'ban': return 'rgba(237, 66, 69, 0.2)';
      default: return 'rgba(79, 84, 92, 0.2)';
    }
  }};
  color: ${({ type }) => {
    switch (type.toLowerCase()) {
      case 'delete': return '#ED4245';
      case 'warn': return '#FAA61A';
      case 'mute': return '#7289DA';
      case 'kick': return '#FAA61A';
      case 'ban': return '#ED4245';
      default: return '#dcddde';
    }
  }};
`;

const ActionButtonGroup = styled.div`
  display: flex;
  gap: 8px;
  justify-content: flex-end;
`;

const ActionIconButton = styled.button<{ danger?: boolean }>`
  background-color: ${({ danger }) => 
    danger ? 'rgba(237, 66, 69, 0.1)' : 'rgba(114, 137, 218, 0.1)'};
  border: none;
  border-radius: 4px;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: ${({ danger }) => 
    danger ? '#ED4245' : '#7289DA'};
  
  &:hover {
    background-color: ${({ danger }) => 
      danger ? 'rgba(237, 66, 69, 0.2)' : 'rgba(114, 137, 218, 0.2)'};
  }
`;

const EmptyState = styled.div`
  text-align: center;
  padding: 40px 0;
  color: #b9bbbe;
  background-color: #2f3136;
  border-radius: 8px;
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

const PaginationContainer = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
`;

const PageInfo = styled.div`
  color: #b9bbbe;
  font-size: 14px;
`;

const PageButtons = styled.div`
  display: flex;
  gap: 8px;
`;

const PageButton = styled.button<{ active?: boolean }>`
  padding: 6px 12px;
  background-color: ${({ active }) => 
    active ? '#5865F2' : '#4f545c'};
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  
  &:hover {
    background-color: ${({ active }) => 
      active ? '#4752c4' : '#5d6269'};
  }
  
  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

// Function to format date
const formatDate = (dateString: string) => {
  const date = new Date(dateString);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
};

// Function to fetch word detection rules
const fetchWordDetectionRules = async (guildId: string): Promise<WordDetectionRule[]> => {
  try {
    // Note: The 'api' instance already has '/api' as its baseURL,
    // so we don't need to include it in the path
    const response = await api.get(`/word_detection/rules/${guildId}`);
    return response.data;
  } catch (error) {
    console.error("Error fetching word detection rules:", error);
    // Return empty array instead of propagating error
    return [];
  }
};

// Function to delete a rule
const deleteWordDetectionRule = async (params: {
  guildId: string;
  ruleId: number;
}): Promise<void> => {
  const { guildId, ruleId } = params;
  try {
    await api.delete(`/word_detection/rules/${guildId}/${ruleId}`);
  } catch (error) {
    console.error("Error deleting rule:", error);
    throw error; // We still throw here since the UI handles this error
  }
};

interface WordDetectionRuleListProps {
  onEdit: (rule: WordDetectionRule) => void;
  onNew: () => void;
  onTest: (rule: WordDetectionRule) => void;
}

const WordDetectionRuleList: React.FC<WordDetectionRuleListProps> = ({
  onEdit,
  onNew,
  onTest,
}) => {
  const { guildId } = useParams<{ guildId: string }>();
  const [searchTerm, setSearchTerm] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const rulesPerPage = 10;
  const queryClient = useQueryClient();
  
  // Fetch rules data
  const { 
    data: rules, 
    isLoading, 
    isError, 
    error, 
    refetch 
  } = useQuery<WordDetectionRule[], Error>({
    queryKey: ['word-detection-rules', guildId],
    queryFn: () => fetchWordDetectionRules(guildId!),
    enabled: !!guildId,
  });
  
  // Delete rule mutation
  const deleteMutation = useMutation({
    mutationFn: deleteWordDetectionRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['word-detection-rules', guildId] });
    },
  });
  
  // Handle delete rule
  const handleDeleteRule = (ruleId: number) => {
    if (window.confirm('Are you sure you want to delete this rule?')) {
      deleteMutation.mutate({
        guildId: guildId!,
        ruleId,
      });
    }
  };
  
  // Filter rules based on search term
  const filteredRules = rules?.filter(rule => 
    rule.pattern.toLowerCase().includes(searchTerm.toLowerCase()) ||
    rule.action.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];
  
  // Paginate rules
  const indexOfLastRule = currentPage * rulesPerPage;
  const indexOfFirstRule = indexOfLastRule - rulesPerPage;
  const currentRules = filteredRules.slice(indexOfFirstRule, indexOfLastRule);
  const totalPages = Math.ceil(filteredRules.length / rulesPerPage);
  
  // Render loading state
  if (isLoading) {
    return (
      <LoadingContainer>
        <div>Loading word detection rules...</div>
      </LoadingContainer>
    );
  }
  
  // Render error state
  if (isError) {
    return (
      <Container>
        <ErrorContainer>
          <div>Failed to load word detection rules: {error?.message || 'Unknown error'}</div>
          <RetryButton onClick={() => refetch()}>Retry</RetryButton>
        </ErrorContainer>
      </Container>
    );
  }
  
  return (
    <Container>
      <Header>
        <Title>Word Detection Rules</Title>
        <div style={{ display: 'flex', gap: '12px' }}>
          <SearchBox>
            <SearchInput
              type="text"
              placeholder="Search rules..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
            <SearchIcon>🔍</SearchIcon>
          </SearchBox>
          <ActionButton variant="primary" onClick={() => onNew()}>
            <ButtonIcon>+</ButtonIcon>
            New Rule
          </ActionButton>
        </div>
      </Header>
      
      {currentRules.length === 0 ? (
        <EmptyState>
          {searchTerm 
            ? 'No rules found matching your search' 
            : 'No word detection rules created yet. Click "New Rule" to create one.'}
        </EmptyState>
      ) : (
        <>
          <RulesTable>
            <TableHead>
              <TableRow>
                <TableHeader>Pattern</TableHeader>
                <TableHeader>Action</TableHeader>
                <TableHeader>Created</TableHeader>
                <TableHeader>Updated</TableHeader>
                <TableHeader style={{ textAlign: 'right' }}>Actions</TableHeader>
              </TableRow>
            </TableHead>
            <tbody>
              {currentRules.map(rule => (
                <TableRow key={rule.id}>
                  <PatternCell>{rule.pattern}</PatternCell>
                  <TableCell>
                    <ActionBadge type={rule.action}>
                      {rule.action.toUpperCase()}
                    </ActionBadge>
                  </TableCell>
                  <TableCell>{formatDate(rule.created_at)}</TableCell>
                  <TableCell>{formatDate(rule.updated_at)}</TableCell>
                  <ActionCell>
                    <ActionButtonGroup>
                      <ActionIconButton onClick={() => onTest(rule)}>
                        🧪
                      </ActionIconButton>
                      <ActionIconButton onClick={() => onEdit(rule)}>
                        ✏️
                      </ActionIconButton>
                      <ActionIconButton 
                        danger 
                        onClick={() => handleDeleteRule(rule.id)}
                        disabled={deleteMutation.isPending}
                      >
                        🗑️
                      </ActionIconButton>
                    </ActionButtonGroup>
                  </ActionCell>
                </TableRow>
              ))}
            </tbody>
          </RulesTable>
          
          {totalPages > 1 && (
            <PaginationContainer>
              <PageInfo>
                Showing {indexOfFirstRule + 1}-{Math.min(indexOfLastRule, filteredRules.length)} of {filteredRules.length} rules
              </PageInfo>
              <PageButtons>
                <PageButton 
                  onClick={() => setCurrentPage(1)} 
                  disabled={currentPage === 1}
                >
                  &laquo;
                </PageButton>
                <PageButton 
                  onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))} 
                  disabled={currentPage === 1}
                >
                  &lt;
                </PageButton>
                
                {Array.from({ length: totalPages }, (_, i) => i + 1)
                  .filter(page => 
                    page === 1 || 
                    page === totalPages || 
                    Math.abs(page - currentPage) <= 1
                  )
                  .map((page, index, array) => {
                    // Add ellipsis
                    if (index > 0 && page - array[index - 1] > 1) {
                      return (
                        <React.Fragment key={`ellipsis-${page}`}>
                          <PageButton disabled>...</PageButton>
                          <PageButton 
                            active={page === currentPage}
                            onClick={() => setCurrentPage(page)}
                          >
                            {page}
                          </PageButton>
                        </React.Fragment>
                      );
                    }
                    
                    return (
                      <PageButton 
                        key={page}
                        active={page === currentPage}
                        onClick={() => setCurrentPage(page)}
                      >
                        {page}
                      </PageButton>
                    );
                  })}
                
                <PageButton 
                  onClick={() => setCurrentPage(prev => Math.min(prev + 1, totalPages))}
                  disabled={currentPage === totalPages}
                >
                  &gt;
                </PageButton>
                <PageButton 
                  onClick={() => setCurrentPage(totalPages)}
                  disabled={currentPage === totalPages}
                >
                  &raquo;
                </PageButton>
              </PageButtons>
            </PaginationContainer>
          )}
        </>
      )}
    </Container>
  );
};

export default WordDetectionRuleList;
