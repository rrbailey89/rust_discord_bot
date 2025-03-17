import React, { useState } from 'react';
import styled from 'styled-components';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import api from '../../services/api';

const ToggleContainer = styled.label`
  display: inline-block;
  position: relative;
  width: 48px;
  height: 24px;
  cursor: pointer;
`;

const ToggleInput = styled.input`
  opacity: 0;
  width: 0;
  height: 0;
  
  &:checked + span {
    background-color: #5865F2;
  }
  
  &:checked + span:before {
    transform: translateX(24px);
  }
  
  &:disabled + span {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

const ToggleSlider = styled.span`
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #4f545c;
  border-radius: 24px;
  transition: background-color 0.2s;
  
  &:before {
    content: '';
    position: absolute;
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    border-radius: 50%;
    transition: transform 0.2s;
  }
`;

interface CommandToggleProps {
  commandId: string;
  guildId: string;
  initialEnabled: boolean;
}

// Function to toggle command state
const toggleCommandState = async (params: {
  guildId: string;
  commandId: string;
  enabled: boolean;
}): Promise<void> => {
  const { guildId, commandId, enabled } = params;
  await api.put(`/api/guilds/${guildId}/commands/${commandId}`, {
    enabled,
  });
};

const CommandToggle: React.FC<CommandToggleProps> = ({
  commandId,
  guildId,
  initialEnabled,
}) => {
  const [enabled, setEnabled] = useState(initialEnabled);
  const queryClient = useQueryClient();

  // Toggle command mutation
  const mutation = useMutation({
    mutationFn: toggleCommandState,
    onMutate: async (variables) => {
      // Optimistically update the UI
      setEnabled(variables.enabled);
      
      // Cancel any outgoing refetches
      await queryClient.cancelQueries({ queryKey: ['commands', guildId] });
      
      // Get snapshot of current data
      const previousCommands = queryClient.getQueryData(['commands', guildId]);
      
      // Return context with previous data
      return { previousCommands };
    },
    onError: (error, variables, context) => {
      // Revert optimistic update on error
      setEnabled(!variables.enabled);
      console.error('Failed to toggle command:', error);
      
      // Show a notification or toast here (implementation depends on your UI library)
    },
    onSettled: () => {
      // Refetch commands after mutation to ensure data is up to date
      queryClient.invalidateQueries({ queryKey: ['commands', guildId] });
    },
  });

  const handleToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newEnabledState = e.target.checked;
    
    mutation.mutate({
      guildId,
      commandId,
      enabled: newEnabledState,
    });
  };

  return (
    <ToggleContainer>
      <ToggleInput
        type="checkbox"
        checked={enabled}
        onChange={handleToggle}
        disabled={mutation.isPending}
      />
      <ToggleSlider />
    </ToggleContainer>
  );
};

export default CommandToggle;
