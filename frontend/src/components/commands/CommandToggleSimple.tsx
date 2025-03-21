import React, { useState } from 'react';
import styled from 'styled-components';

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

const CommandToggleSimple: React.FC<CommandToggleProps> = ({
  commandId, 
  guildId,
  initialEnabled,
}) => {
  const [enabled, setEnabled] = useState(initialEnabled);
  const [isUpdating, setIsUpdating] = useState(false);

  const handleToggle = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const newEnabledState = e.target.checked;
    setEnabled(newEnabledState);
    setIsUpdating(true);
    
    try {
      // Call the API to update the command settings using the API service
      // which already has the /api prefix configured
      const response = await fetch(`/commands/${commandId}/settings?guild_id=${guildId}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`,
        },
        body: JSON.stringify({
          enabled: newEnabledState
        }),
      });
      
      if (!response.ok) {
        throw new Error(`API request failed with status ${response.status}`);
      }
      
      console.log(`Command ${commandId} ${newEnabledState ? 'enabled' : 'disabled'} for guild ${guildId}`);
    } catch (error) {
      console.error(`Failed to update command settings:`, error);
      // Revert UI state on failure
      setEnabled(!newEnabledState);
      alert('Failed to update command settings. Please try again.');
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <ToggleContainer>
      <ToggleInput
        type="checkbox"
        checked={enabled}
        onChange={handleToggle}
      />
      <ToggleSlider />
    </ToggleContainer>
  );
};

export default CommandToggleSimple;
