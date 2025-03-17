import React, { useState, useEffect } from 'react';
import styled from 'styled-components';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { WordDetectionRule } from '../../types';
import api from '../../services/api';

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

const Input = styled.input`
  width: 100%;
  padding: 10px 12px;
  background-color: #40444b;
  border: 1px solid #202225;
  border-radius: 4px;
  color: #dcddde;
  font-size: 14px;
  
  &:focus {
    outline: none;
    border-color: #5865F2;
  }
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
  font-family: monospace;
  resize: vertical;
  
  &:focus {
    outline: none;
    border-color: #5865F2;
  }
`;

const Select = styled.select`
  width: 100%;
  padding: 10px 12px;
  background-color: #40444b;
  border: 1px solid #202225;
  border-radius: 4px;
  color: #dcddde;
  font-size: 14px;
  
  &:focus {
    outline: none;
    border-color: #5865F2;
  }
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
  
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
`;

const ActionParamsContainer = styled.div`
  background-color: #2f3136;
  border-radius: 4px;
  padding: 16px;
  margin-top: 8px;
`;

const HelpText = styled.div`
  margin-top: 4px;
  font-size: 12px;
  color: #b9bbbe;
  line-height: 1.4;
`;

const PatternExample = styled.code`
  display: block;
  background-color: #202225;
  padding: 8px 12px;
  margin: 8px 0;
  border-radius: 4px;
  color: #dcddde;
  font-family: monospace;
  font-size: 13px;
`;

const StatusMessage = styled.div<{ error?: boolean }>`
  margin-top: 16px;
  padding: 12px;
  border-radius: 4px;
  background-color: ${({ error }) => 
    error ? 'rgba(237, 66, 69, 0.1)' : 'rgba(59, 165, 93, 0.1)'};
  border-left: 4px solid ${({ error }) => 
    error ? '#ED4245' : '#3ba55d'};
  color: #ffffff;
`;

// Common action param fields by action type
type ActionParamField = {
  name: string;
  label: string;
  type: 'text' | 'textarea' | 'number' | 'select';
  placeholder?: string;
  options?: { value: string; label: string }[];
  defaultValue: any;
  helpText?: string;
};

interface WordDetectionRuleEditorProps {
  guildId: string;
  rule?: WordDetectionRule; // If provided, we're editing an existing rule
  onClose: () => void;
}

// Function to create a new rule
const createWordDetectionRule = async (params: {
  guildId: string;
  pattern: string;
  action: string;
  action_params: Record<string, any>;
}): Promise<WordDetectionRule> => {
  const { guildId, pattern, action, action_params } = params;
  const response = await api.post(`/api/guilds/${guildId}/word-detection`, {
    pattern,
    action,
    action_params,
  });
  return response.data;
};

// Function to update an existing rule
const updateWordDetectionRule = async (params: {
  guildId: string;
  ruleId: number;
  pattern: string;
  action: string;
  action_params: Record<string, any>;
}): Promise<WordDetectionRule> => {
  const { guildId, ruleId, pattern, action, action_params } = params;
  const response = await api.put(`/api/guilds/${guildId}/word-detection/${ruleId}`, {
    pattern,
    action,
    action_params,
  });
  return response.data;
};

const WordDetectionRuleEditor: React.FC<WordDetectionRuleEditorProps> = ({
  guildId,
  rule,
  onClose,
}) => {
  const isEditing = !!rule;
  const [pattern, setPattern] = useState(rule?.pattern || '');
  const [action, setAction] = useState(rule?.action || 'delete');
  const [actionParams, setActionParams] = useState<Record<string, any>>(
    rule?.action_params || {}
  );
  const [statusMessage, setStatusMessage] = useState<{ message: string; error: boolean } | null>(null);
  const queryClient = useQueryClient();

  // Available actions
  const actions = [
    { value: 'delete', label: 'Delete Message' },
    { value: 'warn', label: 'Warn User' },
    { value: 'mute', label: 'Mute User' },
    { value: 'kick', label: 'Kick User' },
    { value: 'ban', label: 'Ban User' },
  ];

  // Get action param fields based on selected action
  const getActionParamFields = (actionType: string): ActionParamField[] => {
    const commonFields: ActionParamField[] = [
      {
        name: 'log_to_channel',
        label: 'Log to Channel ID',
        type: 'text',
        placeholder: 'Channel ID for logging actions',
        defaultValue: '',
        helpText: 'Leave empty to use the server\'s default log channel',
      },
    ];

    switch (actionType.toLowerCase()) {
      case 'delete':
        return [
          ...commonFields,
          {
            name: 'notify_user',
            label: 'Notify User',
            type: 'select',
            options: [
              { value: 'yes', label: 'Yes' },
              { value: 'no', label: 'No' },
            ],
            defaultValue: 'yes',
            helpText: 'Send a notification to the user about the deleted message',
          }
        ];
      
      case 'warn':
        return [
          ...commonFields,
          {
            name: 'warning_message',
            label: 'Warning Message',
            type: 'textarea',
            placeholder: 'Warning message to show the user',
            defaultValue: 'Your message was flagged for inappropriate content.',
            helpText: 'The message to send to the user explaining the warning',
          }
        ];
      
      case 'mute':
        return [
          ...commonFields,
          {
            name: 'duration',
            label: 'Mute Duration (minutes)',
            type: 'number',
            defaultValue: 10,
            helpText: 'How long the user should be muted for',
          },
          {
            name: 'reason',
            label: 'Reason',
            type: 'textarea',
            placeholder: 'Reason for muting the user',
            defaultValue: 'Inappropriate language',
            helpText: 'This will be shown in the audit log and to the user',
          }
        ];
      
      case 'kick':
        return [
          ...commonFields,
          {
            name: 'reason',
            label: 'Reason',
            type: 'textarea',
            placeholder: 'Reason for kicking the user',
            defaultValue: 'Inappropriate language',
            helpText: 'This will be shown in the audit log and to the user',
          }
        ];
      
      case 'ban':
        return [
          ...commonFields,
          {
            name: 'delete_message_days',
            label: 'Delete Message History (days)',
            type: 'number',
            defaultValue: 1,
            helpText: 'Number of days of message history to delete',
          },
          {
            name: 'reason',
            label: 'Reason',
            type: 'textarea',
            placeholder: 'Reason for banning the user',
            defaultValue: 'Inappropriate language',
            helpText: 'This will be shown in the audit log and to the user',
          }
        ];
      
      default:
        return commonFields;
    }
  };

  // Get action param fields for the current action
  const actionParamFields = getActionParamFields(action);
  
  // Initialize action params when action changes or on first load
  useEffect(() => {
    // Get param fields for the current action
    const fields = getActionParamFields(action);
    
    // Create an initial params object with default values
    const initialParams: Record<string, any> = {};
    fields.forEach(field => {
      // If we're editing and the param exists, use that value
      if (isEditing && rule?.action_params && field.name in rule.action_params) {
        initialParams[field.name] = rule.action_params[field.name];
      } else {
        // Otherwise use the default value
        initialParams[field.name] = field.defaultValue;
      }
    });
    
    setActionParams(initialParams);
  }, [action, isEditing, rule]);
  
  // Create rule mutation
  const createMutation = useMutation({
    mutationFn: createWordDetectionRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['word-detection-rules', guildId] });
      setStatusMessage({
        message: 'Rule created successfully!',
        error: false,
      });
      
      // Close the modal after a delay
      setTimeout(() => {
        onClose();
      }, 1500);
    },
    onError: (error: Error) => {
      setStatusMessage({
        message: `Failed to create rule: ${error.message}`,
        error: true,
      });
    },
  });
  
  // Update rule mutation
  const updateMutation = useMutation({
    mutationFn: updateWordDetectionRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['word-detection-rules', guildId] });
      setStatusMessage({
        message: 'Rule updated successfully!',
        error: false,
      });
      
      // Close the modal after a delay
      setTimeout(() => {
        onClose();
      }, 1500);
    },
    onError: (error: Error) => {
      setStatusMessage({
        message: `Failed to update rule: ${error.message}`,
        error: true,
      });
    },
  });
  
  // Handle form submission
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Validate pattern
    if (!pattern.trim()) {
      setStatusMessage({
        message: 'Pattern cannot be empty',
        error: true,
      });
      return;
    }
    
    if (isEditing && rule) {
      // Update existing rule
      updateMutation.mutate({
        guildId,
        ruleId: rule.id,
        pattern,
        action,
        action_params: actionParams,
      });
    } else {
      // Create new rule
      createMutation.mutate({
        guildId,
        pattern,
        action,
        action_params: actionParams,
      });
    }
  };
  
  // Handle action param change
  const handleActionParamChange = (name: string, value: any) => {
    setActionParams(prev => ({
      ...prev,
      [name]: value,
    }));
  };
  
  const isPending = createMutation.isPending || updateMutation.isPending;
  
  return (
    <Modal onClick={() => !isPending && onClose()}>
      <ModalContent onClick={(e) => e.stopPropagation()}>
        <Header>
          <Title>{isEditing ? 'Edit Rule' : 'Create New Rule'}</Title>
          <CloseButton onClick={() => !isPending && onClose()}>×</CloseButton>
        </Header>
        
        <form onSubmit={handleSubmit}>
          <FormGroup>
            <Label htmlFor="pattern">Pattern</Label>
            <Textarea
              id="pattern"
              value={pattern}
              onChange={(e) => setPattern(e.target.value)}
              placeholder="Enter a regex pattern or literal text"
              disabled={isPending}
            />
            <HelpText>
              Enter a regex pattern to match messages. For example:
              <PatternExample>bad[-_\\s]*word</PatternExample>
              This will match: "badword", "bad word", "bad_word", "bad-word", etc.
            </HelpText>
          </FormGroup>
          
          <FormGroup>
            <Label htmlFor="action">Action</Label>
            <Select
              id="action"
              value={action}
              onChange={(e) => setAction(e.target.value)}
              disabled={isPending}
            >
              {actions.map(action => (
                <option key={action.value} value={action.value}>
                  {action.label}
                </option>
              ))}
            </Select>
            <HelpText>
              Select what action to take when this pattern is detected in a message.
            </HelpText>
          </FormGroup>
          
          <ActionParamsContainer>
            {actionParamFields.map(field => (
              <FormGroup key={field.name}>
                <Label htmlFor={field.name}>{field.label}</Label>
                
                {field.type === 'textarea' ? (
                  <Textarea
                    id={field.name}
                    value={actionParams[field.name] || ''}
                    onChange={(e) => handleActionParamChange(field.name, e.target.value)}
                    placeholder={field.placeholder}
                    disabled={isPending}
                  />
                ) : field.type === 'select' ? (
                  <Select
                    id={field.name}
                    value={actionParams[field.name] || ''}
                    onChange={(e) => handleActionParamChange(field.name, e.target.value)}
                    disabled={isPending}
                  >
                    {field.options?.map(option => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </Select>
                ) : (
                  <Input
                    type={field.type}
                    id={field.name}
                    value={actionParams[field.name] || ''}
                    onChange={(e) => {
                      const value = field.type === 'number' 
                        ? parseInt(e.target.value, 10) 
                        : e.target.value;
                      handleActionParamChange(field.name, value);
                    }}
                    placeholder={field.placeholder}
                    disabled={isPending}
                  />
                )}
                
                {field.helpText && <HelpText>{field.helpText}</HelpText>}
              </FormGroup>
            ))}
          </ActionParamsContainer>
          
          <ButtonContainer>
            <Button type="button" onClick={onClose} disabled={isPending}>
              Cancel
            </Button>
            <Button primary type="submit" disabled={isPending}>
              {isPending 
                ? (isEditing ? 'Updating...' : 'Creating...') 
                : (isEditing ? 'Update Rule' : 'Create Rule')
              }
            </Button>
          </ButtonContainer>
          
          {statusMessage && (
            <StatusMessage error={statusMessage.error}>
              {statusMessage.message}
            </StatusMessage>
          )}
        </form>
      </ModalContent>
    </Modal>
  );
};

export default WordDetectionRuleEditor;
