import React, { useState, useEffect } from 'react'; // Added useEffect
import { useParams } from 'react-router-dom';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import styled from 'styled-components';
// Import fetchGuildDetails and necessary types
import { fetchGuildSettings, updateGuildSettings, fetchGuildDetails } from '../../services/api';
import { GuildSettings as GuildSettingsType, Guild, ChannelInfo } from '../../types';

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

  // Fetch current settings using the correct guildId type (number) if needed by API function
  const numericGuildId = guildId ? parseInt(guildId, 10) : undefined;

  const { data: fetchedSettings, isLoading, error } = useQuery({
    queryKey: ['guildSettings', numericGuildId],
    // Ensure fetchGuildSettings expects string or number as needed
    queryFn: () => fetchGuildSettings(guildId!),
    enabled: !!guildId,
    // Keep data fresh but avoid rapid refetching on focus/mount
    staleTime: 5 * 60 * 1000, // 5 minutes
    refetchOnWindowFocus: false,
  });

  // Fetch guild details (including channels)
  const { data: guildDetails } = useQuery<Guild, Error>({ // Use Guild type here
    queryKey: ['guildDetails', guildId],
    queryFn: () => fetchGuildDetails(guildId!),
    enabled: !!guildId,
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
  });

  // Extract and filter text channels once guildDetails are loaded
  const textChannels = React.useMemo(() => {
    return guildDetails?.channels?.filter(ch => ch.channel_type === 0) || []; // 0 = Text channel
  }, [guildDetails]);


  // Initialize form state with defaults matching the NEW GuildSettingsType (using string IDs)
  // Use null for optional fields where appropriate
  const [isInitialLoad, setIsInitialLoad] = useState(true); // Flag for initial load
  const [formState, setFormState] = useState<Partial<GuildSettingsType>>({
    guild_id: guildId || '', // Use string guildId from params
    prefix: null,
    mod_role_id: null, // Initialize as null (will become string | null)
    admin_role_id: null, // Initialize as null (will become string | null)
    settings: { // Initialize nested settings object
      autoModeration: {
        enabled: false,
        filterLinks: false,
        filterInvites: false,
        filterProfanity: false,
      },
      welcomeMessage: {
        enabled: false,
        channelId: null,
        message: 'Welcome {user} to {server}!',
      },
    },
    emoji_reactions_enabled: true, // Default to true as per backend logic
    level_up_channel_id: null, // Initialize as null (will become string | null)
    warn_channel_id: null, // Initialize as null (will become string | null)
    url_rule: null,
    delete_log_channel_id: null, // Initialize as null (will become string | null)
    reaction_log_channel_id: null, // Initialize as null (will become string | null)
    // Remove old/potentially conflicting fields if they are now handled within 'settings' or renamed
    // logChannelId: '', // Example: If this is now delete_log_channel_id, remove this line
    // moderationEnabled: true, // Example: If this concept is handled differently, remove/update
  });

  // Update form state when settings are loaded/refetched
  useEffect(() => {
    console.log("useEffect [fetchedSettings] running...", { isInitialLoad, fetchedSettings }); // Add log
    // Only apply fetched settings on initial load or if fetchedSettings becomes available
    if (fetchedSettings && isInitialLoad) {
      console.log("Applying fetched settings to form state (initial load)..."); // Add log
      // Merge fetched settings into the state, preserving defaults for missing fields
      setFormState(prev => ({
        ...prev, // Keep existing state (like guild_id)
        ...fetchedSettings, // Overwrite with fetched data
        // Ensure nested settings are properly merged or replaced
        settings: {
          ...(prev.settings ?? {}), // Keep previous nested defaults if needed
          ...(fetchedSettings.settings ?? {}), // Overwrite with fetched nested settings
          // Explicitly merge deeper structures if necessary
          autoModeration: {
            ...(prev.settings?.autoModeration ?? {}),
            ...(fetchedSettings.settings?.autoModeration ?? {}),
          },
          welcomeMessage: {
            ...(prev.settings?.welcomeMessage ?? {}),
            ...(fetchedSettings.settings?.welcomeMessage ?? {}),
          },
        },
        // Ensure channel IDs are treated as strings or null
        level_up_channel_id: fetchedSettings.level_up_channel_id ?? null,
        warn_channel_id: fetchedSettings.warn_channel_id ?? null,
        delete_log_channel_id: fetchedSettings.delete_log_channel_id ?? null,
        reaction_log_channel_id: fetchedSettings.reaction_log_channel_id ?? null,
      }));
      console.log("Fetched settings applied (initial load):", fetchedSettings);
      setIsInitialLoad(false); // Mark initial load as complete
    } else if (!fetchedSettings) {
        console.log("useEffect [fetchedSettings]: No fetchedSettings data yet."); // Log for else case
    } else if (!isInitialLoad) {
        console.log("useEffect [fetchedSettings]: Skipping state update, not initial load."); // Log skip
    }
  }, [fetchedSettings, isInitialLoad]); // Add isInitialLoad to dependency array

  const [saveSuccess, setSaveSuccess] = useState(false);

  // Mutation for updating settings
  const mutation = useMutation({
    // Ensure the payload matches UpdateGuildSettingsRequest (mostly top-level fields)
    mutationFn: (settingsToUpdate: Partial<GuildSettingsType>) => {
       // Construct the payload based on UpdateGuildSettingsRequest structure
       const payload = {
         prefix: settingsToUpdate.prefix,
         mod_role_id: settingsToUpdate.mod_role_id,
         admin_role_id: settingsToUpdate.admin_role_id,
         // Note: Backend doesn't seem to accept the whole 'settings' object for update
         // It handles specific fields like emoji_reactions_enabled and url_rule separately
         emoji_reactions_enabled: settingsToUpdate.emoji_reactions_enabled,
         url_rule: settingsToUpdate.url_rule,
         // Channel IDs are expected as strings by the backend request model! Convert back.
         level_up_channel_id: settingsToUpdate.level_up_channel_id?.toString() || null,
         warn_channel_id: settingsToUpdate.warn_channel_id?.toString() || null,
         delete_log_channel_id: settingsToUpdate.delete_log_channel_id?.toString() || null,
         reaction_log_channel_id: settingsToUpdate.reaction_log_channel_id?.toString() || null,
         // Include the nested settings object in the payload
         settings: settingsToUpdate.settings,
       };
       return updateGuildSettings(guildId!, payload);
    },
    onSuccess: (data) => { // API response might contain success/message
      console.log("Settings updated successfully:", data);
      // Invalidate queries to ensure fresh data on potential future fetches
      queryClient.invalidateQueries({ queryKey: ['guildSettings', numericGuildId] });
      queryClient.invalidateQueries({ queryKey: ['guildDetails', guildId] }); // Also invalidate details if channels changed

      setSaveSuccess(true);
      // Reload the page after a short delay to show success message
      setTimeout(() => {
        window.location.reload();
      }, 1500); // Reload after 1.5 seconds
    },
    onError: (error) => {
      console.error("Error updating settings:", error);
      // Potentially display a more user-friendly error message
    }
  });

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value, type } = e.target;
    const checked = type === 'checkbox' ? (e.target as HTMLInputElement).checked : undefined;
    console.log(`handleChange - name: ${name}, type: ${type}, checked: ${checked}, value: ${value}`);
    // Identify ID fields that should remain strings
    const isIdField = ['guild_id', 'mod_role_id', 'admin_role_id', 'level_up_channel_id', 'warn_channel_id', 'delete_log_channel_id', 'reaction_log_channel_id'].includes(name) || name.endsWith('channelId');

    // Function to update nested state within the 'settings' object
    const updateNestedSetting = (keys: string[], val: any) => {
      setFormState(prev => {
        const newState = { ...prev };
        let currentLevel: any = newState.settings ?? {};
        if (!newState.settings) newState.settings = {}; // Ensure settings object exists

        for (let i = 0; i < keys.length - 1; i++) {
          if (!currentLevel[keys[i]]) {
            currentLevel[keys[i]] = {};
          }
          currentLevel = currentLevel[keys[i]];
        }
        currentLevel[keys[keys.length - 1]] = val;
        console.log(`Nested state updated for ${name}:`, newState); // Log state right after update
        return newState;
      });
    };

    if (name.startsWith('settings.')) {
      // Handle fields explicitly nested under 'settings' (like autoModeration, welcomeMessage)
      const keys = name.substring('settings.'.length).split('.');
      const finalValue = type === 'checkbox' ? checked : value;
      updateNestedSetting(keys, finalValue);

    } else {
      // Handle top-level properties
      let finalValue: any;
      if (type === 'checkbox') {
        finalValue = checked;
      } else if (isIdField) {
        // Keep ID fields as strings, use null if empty string
        finalValue = value === '' ? null : value;
      } else {
        // Handle other types (e.g., prefix, url_rule) - keep as string or handle specific types if needed
        finalValue = value;
      }

      // Update state using a functional update
      setFormState(prevState => {
        console.log(`handleChange: State *before* update for ${name}:`, prevState);
        const newState = {
          ...prevState,
          [name]: finalValue,
        };
        console.log(`handleChange: State *after* update for ${name}:`, newState);
        return newState;
      });
    }
  };


  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    console.log("Submitting form state:", formState); // Log state before mutation
    // Pass the current formState to the mutation function,
    // it will construct the correct payload inside mutationFn
    mutation.mutate(formState);
  };

  // Reset form to originally fetched settings
  const handleReset = () => {
    if (fetchedSettings) {
       setFormState(prev => ({
        ...prev, // Keep existing state (like guild_id)
        ...fetchedSettings, // Overwrite with fetched data
        settings: { // Ensure nested settings are reset too
          ...(prev.settings ?? {}),
          ...(fetchedSettings.settings ?? {}),
           autoModeration: {
            ...(prev.settings?.autoModeration ?? {}),
            ...(fetchedSettings.settings?.autoModeration ?? {}),
          },
          welcomeMessage: {
            ...(prev.settings?.welcomeMessage ?? {}),
            ...(fetchedSettings.settings?.welcomeMessage ?? {}),
          },
        },
         level_up_channel_id: fetchedSettings.level_up_channel_id ?? null,
         warn_channel_id: fetchedSettings.warn_channel_id ?? null,
         delete_log_channel_id: fetchedSettings.delete_log_channel_id ?? null,
         reaction_log_channel_id: fetchedSettings.reaction_log_channel_id ?? null,
      }));
    }
  };

  if (isLoading) return <div>Loading settings...</div>;
  // Display error from fetching
  if (error) return <div>Error loading settings: {String(error)}</div>;
  // Display error from mutation
  const mutationError = mutation.error ? String(mutation.error) : null;


  // Render the form using the updated formState structure
  return (
    <SettingsContainer>
      <SettingsHeader>Guild Settings</SettingsHeader>

      <form onSubmit={handleSubmit}>
        {/* General Settings */}
        <SettingsSection>
          <SectionTitle>General Settings</SectionTitle>
          <FormGroup>
            <Label htmlFor="prefix">Command Prefix</Label>
            <Input
              id="prefix"
              name="prefix"
              value={formState.prefix ?? ''} // Use ?? '' for optional string
              onChange={handleChange}
              placeholder="!"
            />
          </FormGroup>
           {/* Example: Emoji Reactions Toggle */}
           <FormGroup>
             <CheckboxLabel>
               <Checkbox
                 name="emoji_reactions_enabled"
                 // Use ?? true because backend defaults to true if missing
                 checked={formState.emoji_reactions_enabled ?? true}
                 onChange={handleChange}
               />
               Enable Emoji Reactions
             </CheckboxLabel>
           </FormGroup>
           {/* Channel Select Dropdowns */}
           <FormGroup>
             <Label htmlFor="level_up_channel_id">Level Up Channel</Label>
             <Select
               id="level_up_channel_id"
               name="level_up_channel_id"
               value={formState.level_up_channel_id ?? ''} // Use string value directly
               onChange={handleChange}
             >
               <option value="">None</option>
               {textChannels.map(channel => (
                 <option key={channel.id} value={channel.id}>
                   #{channel.name}
                 </option>
               ))}
             </Select>
           </FormGroup>
            <FormGroup>
             <Label htmlFor="warn_channel_id">Warn Channel</Label>
             <Select
               id="warn_channel_id"
               name="warn_channel_id"
               value={formState.warn_channel_id ?? ''} // Use string value directly
               onChange={handleChange}
             >
               <option value="">None</option>
               {textChannels.map(channel => (
                 <option key={channel.id} value={channel.id}>
                   #{channel.name}
                 </option>
               ))}
             </Select>
           </FormGroup>
            <FormGroup>
             <Label htmlFor="delete_log_channel_id">Delete Log Channel</Label>
             <Select
               id="delete_log_channel_id"
               name="delete_log_channel_id"
               value={formState.delete_log_channel_id ?? ''} // Use string value directly
               onChange={handleChange}
             >
               <option value="">None</option>
               {textChannels.map(channel => (
                 <option key={channel.id} value={channel.id}>
                   #{channel.name}
                 </option>
               ))}
             </Select>
           </FormGroup>
            <FormGroup>
             <Label htmlFor="reaction_log_channel_id">Reaction Log Channel</Label>
             <Select
               id="reaction_log_channel_id"
               name="reaction_log_channel_id"
               value={formState.reaction_log_channel_id ?? ''} // Use string value directly
               onChange={handleChange}
             >
               <option value="">None</option>
               {textChannels.map(channel => (
                 <option key={channel.id} value={channel.id}>
                   #{channel.name}
                 </option>
               ))}
             </Select>
           </FormGroup>
           <FormGroup>
             <Label htmlFor="url_rule">URL Rule Regex</Label>
             <Input
               id="url_rule"
               name="url_rule"
               value={formState.url_rule ?? ''}
               onChange={handleChange}
               placeholder="Enter regex for URL rule"
             />
           </FormGroup>
           {/* Remove old fields like logChannelId, moderationEnabled if replaced */}
        </SettingsSection>

        {/* Auto-Moderation - Access via settings object */}
        <SettingsSection>
          <SectionTitle>Auto-Moderation</SectionTitle>
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="settings.autoModeration.enabled" // Updated name
                // Use optional chaining and nullish coalescing
                checked={formState.settings?.autoModeration?.enabled ?? false}
                onChange={handleChange}
              />
              Enable Auto-Moderation
            </CheckboxLabel>
          </FormGroup>
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="settings.autoModeration.filterLinks" // Updated name
                checked={formState.settings?.autoModeration?.filterLinks ?? false}
                onChange={handleChange}
                disabled={!(formState.settings?.autoModeration?.enabled ?? false)}
              />
              Filter Links
            </CheckboxLabel>
          </FormGroup>
           <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="settings.autoModeration.filterInvites" // Updated name
                checked={formState.settings?.autoModeration?.filterInvites ?? false}
                onChange={handleChange}
                disabled={!(formState.settings?.autoModeration?.enabled ?? false)}
              />
              Filter Discord Invites
            </CheckboxLabel>
          </FormGroup>
           <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="settings.autoModeration.filterProfanity" // Updated name
                checked={formState.settings?.autoModeration?.filterProfanity ?? false}
                onChange={handleChange}
                disabled={!(formState.settings?.autoModeration?.enabled ?? false)}
              />
              Filter Profanity
            </CheckboxLabel>
          </FormGroup>
        </SettingsSection>

        {/* Welcome Message - Access via settings object */}
        <SettingsSection>
          <SectionTitle>Welcome Message</SectionTitle>
          <FormGroup>
            <CheckboxLabel>
              <Checkbox
                name="settings.welcomeMessage.enabled" // Updated name
                checked={formState.settings?.welcomeMessage?.enabled ?? false}
                onChange={handleChange}
              />
              Enable Welcome Message
            </CheckboxLabel>
          </FormGroup>
          <FormGroup>
            <Label htmlFor="settings.welcomeMessage.channelId">Welcome Channel</Label>
            <Select
              id="settings.welcomeMessage.channelId"
              name="settings.welcomeMessage.channelId" // Updated name
              value={formState.settings?.welcomeMessage?.channelId ?? ''} // Use string value directly
              onChange={handleChange}
              disabled={!(formState.settings?.welcomeMessage?.enabled ?? false)}
            >
              <option value="">None</option>
              {textChannels.map(channel => (
                <option key={channel.id} value={channel.id}>
                  #{channel.name}
                </option>
              ))}
            </Select>
          </FormGroup>
          <FormGroup>
            <Label htmlFor="settings.welcomeMessage.message">Welcome Message</Label>
            <Input
              id="settings.welcomeMessage.message"
              name="settings.welcomeMessage.message" // Updated name
              value={formState.settings?.welcomeMessage?.message ?? 'Welcome {user} to {server}!'}
              onChange={handleChange}
              placeholder="Welcome {user} to {server}!"
              disabled={!(formState.settings?.welcomeMessage?.enabled ?? false)}
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

        {mutationError && (
          <ErrorMessage>Error saving settings: {mutationError}</ErrorMessage>
        )}

        {saveSuccess && (
          <SuccessMessage>Settings saved successfully!</SuccessMessage>
        )}
      </form>
    </SettingsContainer>
  );
};

export default SettingsForm;
