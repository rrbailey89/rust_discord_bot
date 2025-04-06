import React, { useState } from 'react';
import styled from 'styled-components';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Command } from '../../types';
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

const TextArea = styled.textarea`
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

const Checkbox = styled.input`
  margin-right: 8px;
`;

const CheckboxLabel = styled.label`
  display: flex;
  align-items: center;
  color: #dcddde;
  font-size: 14px;
  cursor: pointer;
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

interface CommandSettingsProps {
  command: Command;
  guildId: string;
  onClose: () => void;
}

// Function to update command settings
const updateCommandSettings = async (params: {
  guildId: string;
  commandId: string;
  settings: Record<string, any>;
}): Promise<Command> => {
  try {
    // Note: The 'api' instance already has '/api' as its baseURL,
    // so we don't need to include it in the path
    const { guildId, commandId, settings } = params;
    const response = await api.put(`/commands/${commandId}/settings?guild_id=${guildId}`, {
      settings,
    });
    return response.data;
  } catch (error) {
    console.error("Error updating command settings:", error);
    throw error;
  }
};

// Helper to get nested value
const getNestedValue = (obj: any, path: string) => {
  if (!obj) return undefined;
  const keys = path.split('.');
  return keys.reduce((o, key) => (o || {})[key], obj);
};

// Helper to set nested value
const setNestedValue = (obj: any, path: string, value: any) => {
  const result = { ...obj };
  const keys = path.split('.');
  const lastKey = keys.pop()!;
  const target = keys.reduce((o, key) => {
    o[key] = o[key] || {};
    return o[key];
  }, result);
  target[lastKey] = value;
  return result;
};

// Define field types
interface BaseField {
  type: string;
  name: string;
  label: string;
  defaultValue: any;
}

interface TextField extends BaseField {
  type: 'text';
  placeholder?: string;
  defaultValue: string;
}

interface TextAreaField extends BaseField {
  type: 'textarea';
  placeholder?: string;
  defaultValue: string;
}

interface CheckboxField extends BaseField {
  type: 'checkbox';
  defaultValue: boolean;
}

interface SelectField extends BaseField {
  type: 'select';
  options: Array<{value: string; label: string}>;
  defaultValue: string;
}

type SettingsField = TextField | TextAreaField | CheckboxField | SelectField;

// Generate schema based on command type - this would be dynamic in a real app
const getSettingsSchema = (command: Command): SettingsField[] => {
  // This is a simplified example. In a real application, this would be 
  // determined dynamically based on the command type or fetched from the server.
  
  // Default schema with common settings
  const defaultSchema: SettingsField[] = [
    {
      type: 'checkbox',
      name: 'restricted',
      label: 'Command requires moderator permissions',
      defaultValue: false,
    },
    {
      type: 'text',
      name: 'cooldown',
      label: 'Cooldown (in seconds)',
      placeholder: '0',
      defaultValue: '0',
    },
  ];
  
  // Add command-specific settings based on command name or ID
  // This is just a simple example - in a real application this would be more dynamic
  switch (command.name.toLowerCase()) {
    case 'ban':
    case 'kick':
    case 'mute':
      return [
        ...defaultSchema,
        {
          type: 'select',
          name: 'restriction_level',
          label: 'Restriction Level',
          options: [
            { value: 'low', label: 'Low (Moderator)' },
            { value: 'medium', label: 'Medium (Admin)' },
            { value: 'high', label: 'High (Server Owner)' },
          ],
          defaultValue: 'medium',
        },
        {
          type: 'checkbox',
          name: 'require_reason',
          label: 'Require reason for action',
          defaultValue: true,
        },
        {
          type: 'text',
          name: 'log_channel',
          label: 'Log Channel ID',
          placeholder: 'Channel ID for action logs',
          defaultValue: '',
        },
      ];
      
    case 'welcome':
    case 'greet':
      return [
        ...defaultSchema,
        {
          type: 'text',
          name: 'welcome_channel',
          label: 'Welcome Channel ID',
          placeholder: 'Channel ID for welcome messages',
          defaultValue: '',
        },
        {
          type: 'textarea',
          name: 'welcome_message',
          label: 'Welcome Message',
          placeholder: 'Welcome {user} to {server}!',
          defaultValue: 'Welcome {user} to {server}!',
        },
      ];
      
    default:
      return defaultSchema;
  }
};

const CommandSettings: React.FC<CommandSettingsProps> = ({
  command,
  guildId,
  onClose,
}) => {
  const [settings, setSettings] = useState<Record<string, any>>(command.settings || {});
  const [statusMessage, setStatusMessage] = useState<{ message: string; error: boolean } | null>(null);
  const queryClient = useQueryClient();
  
  // Generate settings schema based on command type
  const settingsSchema = getSettingsSchema(command);
  
  // Update settings mutation
  const mutation = useMutation({
    mutationFn: updateCommandSettings,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['commands', guildId] });
      setStatusMessage({
        message: 'Settings saved successfully!',
        error: false,
      });
      
      // Close the modal after successful save (with a slight delay to show success message)
      setTimeout(() => {
        onClose();
      }, 1500);
    },
    onError: (error: Error) => {
      setStatusMessage({
        message: `Failed to save settings: ${error.message}`,
        error: true,
      });
    },
  });
  
  // Handle input changes
  const handleInputChange = (name: string, value: any) => {
    setSettings(prev => setNestedValue(prev, name, value));
  };
  
  // Handle form submission
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    mutation.mutate({
      guildId,
      commandId: command.id,
      settings,
    });
  };
  
  return (
    <Modal onClick={() => !mutation.isPending && onClose()}>
      <ModalContent onClick={(e) => e.stopPropagation()}>
        <Header>
          <Title>Settings: {command.name}</Title>
          <CloseButton onClick={() => !mutation.isPending && onClose()}>×</CloseButton>
        </Header>
        
        <form onSubmit={handleSubmit}>
          {settingsSchema.map(field => {
            const value = getNestedValue(settings, field.name) ?? field.defaultValue;
            
            return (
              <FormGroup key={field.name}>
                {field.type === 'checkbox' ? (
                  <CheckboxLabel>
                    <Checkbox
                      type="checkbox"
                      checked={!!value}
                      onChange={(e) => handleInputChange(field.name, e.target.checked)}
                      disabled={mutation.isPending}
                    />
                    {field.label}
                  </CheckboxLabel>
                ) : field.type === 'select' ? (
                  <>
                    <Label htmlFor={field.name}>{field.label}</Label>
                    <Select
                      id={field.name}
                      value={value}
                      onChange={(e) => handleInputChange(field.name, e.target.value)}
                      disabled={mutation.isPending}
                    >
                      {field.options?.map(option => (
                        <option key={option.value} value={option.value}>
                          {option.label}
                        </option>
                      ))}
                    </Select>
                  </>
                ) : field.type === 'textarea' ? (
                  <>
                    <Label htmlFor={field.name}>{field.label}</Label>
                    <TextArea
                      id={field.name}
                      value={value}
                      placeholder={field.placeholder}
                      onChange={(e) => handleInputChange(field.name, e.target.value)}
                      disabled={mutation.isPending}
                    />
                  </>
                ) : (
                  <>
                    <Label htmlFor={field.name}>{field.label}</Label>
                    <Input
                      type={field.type}
                      id={field.name}
                      value={value}
                      placeholder={field.placeholder}
                      onChange={(e) => handleInputChange(field.name, e.target.value)}
                      disabled={mutation.isPending}
                    />
                  </>
                )}
              </FormGroup>
            );
          })}
          
          <ButtonContainer>
            <Button type="button" onClick={onClose} disabled={mutation.isPending}>
              Cancel
            </Button>
            <Button primary type="submit" disabled={mutation.isPending}>
              {mutation.isPending ? 'Saving...' : 'Save Changes'}
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

export default CommandSettings;
