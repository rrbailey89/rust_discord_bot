import React, { useState } from 'react';
import { useParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import styled from 'styled-components';
import { fetchGuildSettings, updateGuildSettings } from '../../services/api';
import { GuildSettings as GuildSettingsType } from '../../types';

const SettingsContainer = styled.div`
  background-color: #2f3136;
  border-radius: 5px;
  padding: 20px;
  margin-bottom: 20px;
`;

const SettingsHeader = styled.h2`
  color: #ffffff;
  margin-bottom: 20px;
  font-size: 1.5rem;
`;

const SettingsSection = styled.div`
  margin-bottom: 24px;
`;

const SectionTitle = styled.h3`
  color: #ffffff;
  font-size: 1.2rem;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #40444b;
`;

const FormGroup = styled.div`
  margin-bottom: 16px;
`;

const Label = styled.label`
  display: block;
  margin-bottom: 8px;
  color: #b9bbbe;
  font-weight: 500;
`;

const Input = styled.input`
  width: 100%;
  padding: 10px;
  background-color: #40444b;
  border: 1px solid #202225;
  border-radius: 3px;
  color: #dcddde;
  font-size: 0.9rem;
  
  &:focus {
    outline: none;
    border-color: #7289da;
  }
`;

const Select = styled.select`
  width: 100%;
  padding: 10px;
  background-color: #40444b;
  border: 1px solid #202225;
  border-radius: 3px;
  color: #dcddde;
  font-size: 0.9rem;
  
  &:focus {
    outline: none;
    border-color: #7289da;
  }
`;

const Checkbox = styled.input.attrs({ type: 'checkbox' })`
  margin-right: 8px;
`;

const CheckboxLabel = styled.label`
  display: flex;
  align-items: center;
  color: #b9bbbe;
  cursor: pointer;
`;

const Button = styled.button`
  padding: 10px 16px;
  background-color: #7289da;
  color: white;
  border: none;
  border-radius: 3px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
  
  &:hover {
    background-color: #677bc4;
  }
  
  &:disabled {
    background-color: #4f5d7e;
    cursor: not-allowed;
  }
`;

const CancelButton = styled(Button)`
  background-color: #4f545c;
  margin-right: 10px;
  
  &:hover {
    background-color: #646970;
  }
`;

const ButtonGroup = styled.div`
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
`;

const ErrorMessage = styled.div`
  color: #f04747;
  margin-top: 5px;
  font-size: 0.9rem;
`;

const SuccessMessage = styled.div`
  color: #43b581;
  margin-top: 5px;
  font-size: 0.9rem;
`;

const SettingsForm: React.FC = () => {
  const { guildId } = useParams<{ guildId: string }>();
  const queryClient = useQueryClient();
  
  // Fetch current settings
  const { data: settings, isLoading, error } = useQuery({
    queryKey: ['guildSettings', guildId],
    queryFn: () => fetchGuildSettings(guildId!),
    enabled: !!guildId,
  });
  
  // Initialize form state with settings data or defaults
  const [formState, setFormState] = useState<GuildSettingsType>({
    prefix: '!',
    logChannelId: '',
    moderationEnabled: true,
    autoModeration: {
      enabled: false,
      filterLinks: false,
      filterInvites: false,
      filterProfanity: false,
    },
    welcomeMessage: {
      enabled: false,
      channelId: '',
      message: 'Welcome {user} to {server}!',
    },
  });
  
  // Update form state when settings are loaded
  React.useEffect(() => {
    if (settings) {
      setFormState(settings);
    }
  }, [settings]);
  
  const [saveSuccess, setSaveSuccess] = useState(false);
  
  // Mutation for updating settings
  const mutation = useMutation({
    mutationFn: (updatedSettings: GuildSettingsType) => 
      updateGuildSettings(guildId!, updatedSettings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['guildSettings', guildId] });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
  });
  
  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value, type } = e.target as HTMLInputElement;
    const checked = type === 'checkbox' ? (e.target as HTMLInputElement).checked : undefined;
    
    if (name.includes('.')) {
      // Handle nested properties
      const [parent, child] = name.split('.');
      setFormState(prev => ({
        ...prev,
        [parent]: {
          ...((prev[parent as keyof GuildSettingsType] as object) || {}),
          [child]: type === 'checkbox' ? checked : value,
        },
      }));
    } else {
      // Handle top-level properties
      setFormState(prev => ({
        ...prev,
        [name]: type === 'checkbox' ? checked : value,
      }));
    }
  };
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    mutation.mutate(formState);
  };
  
  const handleReset = () => {
    if (settings) {
      setFormState(settings);
    }
  };
  
  if (isLoading) return <div>Loading settings...</div>;
  if (error) return <div>Error loading settings: {String(error)}</div>;
  
  return (
    <SettingsContainer>
      <SettingsHeader>Guild Settings</SettingsHeader>
      
      <form onSubmit={handleSubmit}>
        <SettingsSection>
          <SectionTitle>General Settings</SectionTitle>
          
          <FormGroup>
            <Label htmlFor="prefix">Command Prefix</Label>
            <Input
              id="prefix"
              name="prefix"
              value={formState.prefix}
              onChange={handleChange}
              placeholder="!"
            />
          </FormGroup>
          
          <FormGroup>
            <Label htmlFor="logChannelId">Log Channel ID</Label>
            <Input
              id="logChannelId"
              name="logChannelId"
              value={formState.logChannelId}
              onChange={handleChange}
              placeholder="Enter channel ID for logs"
            />
          </FormGroup>
          
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="moderationEnabled"
                checked={formState.moderationEnabled}
                onChange={handleChange}
              />
              Enable Moderation Commands
            </CheckboxLabel>
          </FormGroup>
        </SettingsSection>
        
        <SettingsSection>
          <SectionTitle>Auto-Moderation</SectionTitle>
          
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="autoModeration.enabled"
                checked={formState.autoModeration.enabled}
                onChange={handleChange}
              />
              Enable Auto-Moderation
            </CheckboxLabel>
          </FormGroup>
          
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="autoModeration.filterLinks"
                checked={formState.autoModeration.filterLinks}
                onChange={handleChange}
                disabled={!formState.autoModeration.enabled}
              />
              Filter Links
            </CheckboxLabel>
          </FormGroup>
          
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="autoModeration.filterInvites"
                checked={formState.autoModeration.filterInvites}
                onChange={handleChange}
                disabled={!formState.autoModeration.enabled}
              />
              Filter Discord Invites
            </CheckboxLabel>
          </FormGroup>
          
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="autoModeration.filterProfanity"
                checked={formState.autoModeration.filterProfanity}
                onChange={handleChange}
                disabled={!formState.autoModeration.enabled}
              />
              Filter Profanity
            </CheckboxLabel>
          </FormGroup>
        </SettingsSection>
        
        <SettingsSection>
          <SectionTitle>Welcome Message</SectionTitle>
          
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="welcomeMessage.enabled"
                checked={formState.welcomeMessage.enabled}
                onChange={handleChange}
              />
              Enable Welcome Message
            </CheckboxLabel>
          </FormGroup>
          
          <FormGroup>
            <Label htmlFor="welcomeMessage.channelId">Welcome Channel ID</Label>
            <Input
              id="welcomeMessage.channelId"
              name="welcomeMessage.channelId"
              value={formState.welcomeMessage.channelId}
              onChange={handleChange}
              placeholder="Enter channel ID for welcome messages"
              disabled={!formState.welcomeMessage.enabled}
            />
          </FormGroup>
          
          <FormGroup>
            <Label htmlFor="welcomeMessage.message">Welcome Message</Label>
            <Input
              id="welcomeMessage.message"
              name="welcomeMessage.message"
              value={formState.welcomeMessage.message}
              onChange={handleChange}
              placeholder="Welcome {user} to {server}!"
              disabled={!formState.welcomeMessage.enabled}
            />
            <div style={{ fontSize: '0.8rem', color: '#72767d', marginTop: '4px' }}>
              Use {'{user}'} for username and {'{server}'} for server name
            </div>
          </FormGroup>
        </SettingsSection>
        
        <ButtonGroup>
          <CancelButton type="button" onClick={handleReset}>
            Reset
          </CancelButton>
          <Button type="submit" disabled={mutation.isPending}>
            {mutation.isPending ? 'Saving...' : 'Save Settings'}
          </Button>
        </ButtonGroup>
        
        {mutation.isError && (
          <ErrorMessage>Error saving settings: {String(mutation.error)}</ErrorMessage>
        )}
        
        {saveSuccess && (
          <SuccessMessage>Settings saved successfully!</SuccessMessage>
        )}
      </form>
    </SettingsContainer>
  );
};

export default SettingsForm;
