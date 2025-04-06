import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { GuildSettings as GuildSettingsType } from '../../types';
import api from '../../services/api';

const Container = styled.div`
  background-color: #2f3136;
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 24px;
`;

const Title = styled.h2`
  color: #ffffff;
  margin: 0 0 24px 0;
`;

const Tabs = styled.div`
  display: flex;
  border-bottom: 1px solid #40444b;
  margin-bottom: 24px;
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
  transition: all 0.2s;
  
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
  
  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
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

// Helper to get nested properties
const getNestedValue = (obj: any, path: string) => {
  const keys = path.split('.');
  return keys.reduce((o, key) => (o || {})[key], obj);
};

// Helper to set nested properties
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

// Function to fetch guild settings
const fetchGuildSettings = async (guildId: string): Promise<GuildSettingsType> => {
  const response = await api.get(`/api/guilds/${guildId}/settings`);
  return response.data;
};

// Function to update guild settings
const updateGuildSettings = async (params: { 
  guildId: string; 
  settings: Record<string, any> 
}): Promise<GuildSettingsType> => {
  const { guildId, settings } = params;
  const response = await api.put(`/api/guilds/${guildId}/settings`, { settings });
  return response.data;
};

// Settings schema to define form fields
const settingsSchema = [
  {
    tab: 'General',
    fields: [
      {
        type: 'text',
        name: 'general.prefix',
        label: 'Command Prefix',
        placeholder: '!',
        defaultValue: '!',
      },
      {
        type: 'checkbox',
        name: 'general.enable_dm_commands',
        label: 'Enable DM Commands',
        defaultValue: true,
      },
      {
        type: 'select',
        name: 'general.log_level',
        label: 'Logging Level',
        options: [
          { value: 'none', label: 'None' },
          { value: 'error', label: 'Errors Only' },
          { value: 'warning', label: 'Warnings & Errors' },
          { value: 'info', label: 'All Information' },
          { value: 'debug', label: 'Debug (Verbose)' },
        ],
        defaultValue: 'info',
      },
    ],
  },
  {
    tab: 'Moderation',
    fields: [
      {
        type: 'select',
        name: 'moderation.default_action',
        label: 'Default Moderation Action',
        options: [
          { value: 'warn', label: 'Warn User' },
          { value: 'mute', label: 'Mute User' },
          { value: 'kick', label: 'Kick User' },
          { value: 'ban', label: 'Ban User' },
        ],
        defaultValue: 'warn',
      },
      {
        type: 'checkbox',
        name: 'moderation.enable_auto_mod',
        label: 'Enable Auto Moderation',
        defaultValue: false,
      },
      {
        type: 'text',
        name: 'moderation.log_channel',
        label: 'Moderation Log Channel ID',
        placeholder: 'Channel ID',
        defaultValue: '',
      },
    ],
  },
  {
    tab: 'Notifications',
    fields: [
      {
        type: 'checkbox',
        name: 'notifications.welcome_message',
        label: 'Send Welcome Message',
        defaultValue: true,
      },
      {
        type: 'text',
        name: 'notifications.welcome_channel',
        label: 'Welcome Channel ID',
        placeholder: 'Channel ID',
        defaultValue: '',
      },
      {
        type: 'text',
        name: 'notifications.welcome_text',
        label: 'Welcome Message',
        placeholder: 'Welcome {user} to {server}!',
        defaultValue: 'Welcome {user} to {server}!',
      },
    ],
  },
];

const GuildSettings: React.FC = () => {
  const { guildId } = useParams<{ guildId: string }>();
  const [activeTab, setActiveTab] = useState('General');
  const [formValues, setFormValues] = useState<Record<string, any>>({});
  const [statusMessage, setStatusMessage] = useState<{ message: string; error: boolean } | null>(null);
  const queryClient = useQueryClient();
  
  // Fetch guild settings
  const { 
    data: settings, 
    isLoading, 
    isError, 
    error,
  } = useQuery<GuildSettingsType, Error>({
    queryKey: ['guild-settings', guildId],
    queryFn: () => fetchGuildSettings(guildId!),
    enabled: !!guildId,
  });

  // Set form values when settings data is loaded
  React.useEffect(() => {
    if (settings) {
      setFormValues(settings.settings || {});
    }
  }, [settings]);
  
  // Update settings mutation
  const mutation = useMutation({
    mutationFn: updateGuildSettings,
    onSuccess: () => {
      // Invalidate and refetch the settings
      queryClient.invalidateQueries({ queryKey: ['guild-settings', guildId] });
      setStatusMessage({
        message: 'Settings saved successfully!',
        error: false,
      });
      
      // Clear success message after 3 seconds
      setTimeout(() => {
        setStatusMessage(null);
      }, 3000);
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
    setFormValues(prev => setNestedValue(prev, name, value));
  };
  
  // Handle form submission
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!guildId) return;
    
    mutation.mutate({
      guildId,
      settings: formValues,
    });
  };
  
  // Reset form to saved settings
  const handleReset = () => {
    if (settings) {
      setFormValues(settings.settings || {});
      setStatusMessage({
        message: 'Form reset to saved settings',
        error: false,
      });
      
      // Clear message after 3 seconds
      setTimeout(() => {
        setStatusMessage(null);
      }, 3000);
    }
  };
  
  // Render loading state
  if (isLoading) {
    return (
      <Container>
        <div>Loading settings...</div>
      </Container>
    );
  }
  
  // Render error state
  if (isError) {
    return (
      <Container>
        <StatusMessage error>
          Failed to load settings: {error?.message || 'Unknown error'}
        </StatusMessage>
        <Button onClick={() => queryClient.invalidateQueries({ queryKey: ['guild-settings', guildId] })}>
          Retry
        </Button>
      </Container>
    );
  }
  
  // Get active tab configuration
  const activeTabConfig = settingsSchema.find(tab => tab.tab === activeTab);
  
  return (
    <Container>
      <Title>Guild Settings</Title>
      
      <Tabs>
        {settingsSchema.map(tab => (
          <Tab
            key={tab.tab}
            active={activeTab === tab.tab}
            onClick={() => setActiveTab(tab.tab)}
          >
            {tab.tab}
          </Tab>
        ))}
      </Tabs>
      
      <form onSubmit={handleSubmit}>
        {activeTabConfig?.fields.map(field => {
          const value = getNestedValue(formValues, field.name) ?? field.defaultValue;
          
          return (
            <FormGroup key={field.name}>
              {field.type === 'checkbox' ? (
                <CheckboxLabel>
                  <Checkbox
                    type="checkbox"
                    checked={value}
                    onChange={(e) => handleInputChange(field.name, e.target.checked)}
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
                  >
                    {field.options?.map(option => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </Select>
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
                  />
                </>
              )}
            </FormGroup>
          );
        })}
        
        <ButtonContainer>
          <Button type="button" onClick={handleReset}>
            Reset
          </Button>
          <Button primary type="submit" disabled={mutation.isPending}>
            {mutation.isPending ? 'Saving...' : 'Save Changes'}
          </Button>
        </ButtonContainer>
      </form>
      
      {statusMessage && (
        <StatusMessage error={statusMessage.error}>
          {statusMessage.message}
        </StatusMessage>
      )}
    </Container>
  );
};

export default GuildSettings;
