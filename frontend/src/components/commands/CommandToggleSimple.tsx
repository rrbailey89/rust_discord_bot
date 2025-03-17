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
  initialEnabled,
}) => {
  const [enabled, setEnabled] = useState(initialEnabled);

  const handleToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newEnabledState = e.target.checked;
    setEnabled(newEnabledState);
    // In a real component, we would call an API here to update the state
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
