import React, { useState, useEffect, useCallback } from 'react';
import styled from 'styled-components';
import { WordDetectionRule } from '../../types';

const Modal = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
`;

const ModalContent = styled.div`
  background-color: #36393f;
  border-radius: 8px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.2);
  width: 90%;
  max-width: 600px;
  max-height: 90vh;
  overflow-y: auto;
  padding: 24px;
`;

const Header = styled.div`
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #40444b;
`;

const Title = styled.h2`
  margin: 0;
  color: #ffffff;
  font-size: 20px;
`;

const CloseButton = styled.button`
  background: none;
  border: none;
  color: #dcddde;
  font-size: 20px;
  cursor: pointer;
  
  &:hover {
    color: #ffffff;
  }
`;

const FormGroup = styled.div`
  margin-bottom: 20px;
`;

const Label = styled.label`
  display: block;
  margin-bottom: 8px;
  color: #b9bbbe;
  font-size: 14px;
`;

const Textarea = styled.textarea`
  width: 100%;
  padding: 10px 12px;
  background-color: #40444b;
  border: 1px solid #202225;
  border-radius: 4px;
  color: #dcddde;
  font-size: 14px;
  min-height: 100px;
  resize: vertical;
  
  &:focus {
    outline: none;
    border-color: #5865F2;
  }
`;

const InfoCard = styled.div`
  background-color: #2f3136;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
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

const PatternDisplay = styled.div`
  background-color: #202225;
  padding: 12px;
  margin: 8px 0;
  border-radius: 4px;
  font-family: monospace;
  color: #dcddde;
  display: flex;
  align-items: center;
`;

const PatternLabel = styled.span`
  background-color: #5865F2;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  margin-right: 12px;
  font-size: 12px;
  font-weight: 600;
`;

const PatternText = styled.code`
  font-size: 14px;
  word-break: break-all;
`;

const ActionBadge = styled.span<{ type: string }>`
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  margin-right: 8px;
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

const ResultsContainer = styled.div`
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #40444b;
`;

const ResultTitle = styled.h3`
  margin: 0 0 16px 0;
  color: #ffffff;
  font-size: 16px;
`;

const MatchResult = styled.div<{ matched: boolean }>`
  background-color: ${({ matched }) => 
    matched ? 'rgba(59, 165, 93, 0.1)' : 'rgba(237, 66, 69, 0.1)'};
  border-left: 4px solid ${({ matched }) => 
    matched ? '#3ba55d' : '#ED4245'};
  padding: 12px;
  border-radius: 4px;
  margin-bottom: 16px;
  color: #ffffff;
`;

const HighlightedText = styled.div`
  background-color: #2f3136;
  padding: 12px;
  border-radius: 4px;
  margin-top: 12px;
  font-family: monospace;
  white-space: pre-wrap;
  word-break: break-word;
`;

const ButtonContainer = styled.div`
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
`;

const Button = styled.button<{ primary?: boolean }>`
  padding: 10px 16px;
  background-color: ${({ primary }) => (primary ? '#5865F2' : '#4f545c')};
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  
  &:hover {
    background-color: ${({ primary }) => (primary ? '#4752c4' : '#5d6269')};
  }
`;

interface WordDetectionRuleTesterProps {
  rule: WordDetectionRule;
  onClose: () => void;
}

interface MatchResult {
  matched: boolean;
  highlightedText?: string;
}

const WordDetectionRuleTester: React.FC<WordDetectionRuleTesterProps> = ({
  rule,
  onClose,
}) => {
  const [sampleText, setSampleText] = useState('');
  const [matchResult, setMatchResult] = useState<MatchResult | null>(null);
  
  // Escape regex special characters if needed
  const escapeRegex = (string: string) => {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  };
  
  // Convert pattern to regex
  const patternToRegex = useCallback(() => {
    try {
      // Try to create a regex directly (in case it's already a valid regex pattern)
      return new RegExp(rule.pattern, 'gi');
    } catch (e) {
      // If that fails, escape the pattern and use it as a literal string
      return new RegExp(escapeRegex(rule.pattern), 'gi');
    }
  }, [rule.pattern]);
  
  // Test the pattern against the sample text
  const testPattern = () => {
    if (!sampleText) {
      return;
    }
    
    try {
      const regex = patternToRegex();
      const matches = sampleText.match(regex);
      
      if (matches && matches.length > 0) {
        // Pattern matched
        // Create highlighted version of the text
        const highlightedText = sampleText.replace(regex, (match) => {
          return `<span style="background-color: rgba(237, 66, 69, 0.3); padding: 2px; border-radius: 2px;">${match}</span>`;
        });
        
        setMatchResult({
          matched: true,
          highlightedText,
        });
      } else {
        // No matches
        setMatchResult({
          matched: false,
        });
      }
    } catch (e) {
      // Regex error
      setMatchResult({
        matched: false,
      });
    }
  };
  
  // Test the pattern when sample text changes
  useEffect(() => {
    if (sampleText) {
      testPattern();
    } else {
      setMatchResult(null);
    }
  }, [sampleText]);
  
  return (
    <Modal onClick={onClose}>
      <ModalContent onClick={(e) => e.stopPropagation()}>
        <Header>
          <Title>Test Word Detection Rule</Title>
          <CloseButton onClick={onClose}>×</CloseButton>
        </Header>
        
        <InfoCard>
          <InfoTitle>Rule Information:</InfoTitle>
          <PatternDisplay>
            <PatternLabel>Pattern</PatternLabel>
            <PatternText>{rule.pattern}</PatternText>
          </PatternDisplay>
          <div style={{ display: 'flex', alignItems: 'center', marginTop: '8px' }}>
            <ActionBadge type={rule.action}>
              {rule.action.toUpperCase()}
            </ActionBadge>
            <InfoText>Will be applied when this pattern is detected</InfoText>
          </div>
        </InfoCard>
        
        <FormGroup>
          <Label htmlFor="sampleText">Test Text</Label>
          <Textarea
            id="sampleText"
            value={sampleText}
            onChange={(e) => setSampleText(e.target.value)}
            placeholder="Type or paste text to test against the pattern..."
          />
        </FormGroup>
        
        {matchResult && (
          <ResultsContainer>
            <ResultTitle>Test Results:</ResultTitle>
            <MatchResult matched={matchResult.matched}>
              {matchResult.matched 
                ? 'Pattern matched! The action would be triggered for this message.' 
                : 'No match. The action would not be triggered for this message.'}
              
              {matchResult.matched && matchResult.highlightedText && (
                <HighlightedText 
                  dangerouslySetInnerHTML={{ __html: matchResult.highlightedText }}
                />
              )}
            </MatchResult>
          </ResultsContainer>
        )}
        
        <ButtonContainer>
          <Button primary onClick={testPattern}>
            Test Again
          </Button>
          <Button onClick={onClose}>
            Close
          </Button>
        </ButtonContainer>
      </ModalContent>
    </Modal>
  );
};

export default WordDetectionRuleTester;
