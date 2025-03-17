import React, { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import styled from 'styled-components';
import { useAuth } from '../../context/AuthContext';
import { Guild } from '../../types';

const SidebarContainer = styled.aside<{ isOpen: boolean }>`
  background-color: #202225;
  width: 240px;
  height: 100%;
  position: fixed;
  left: ${({ isOpen }) => (isOpen ? '0' : '-240px')};
  top: 64px;
  bottom: 0;
  transition: left 0.3s ease;
  overflow-y: auto;
  z-index: 100;

  @media (min-width: 768px) {
    position: relative;
    left: 0;
    top: 0;
  }
`;

const GuildList = styled.ul`
  list-style: none;
  padding: 0;
  margin: 0;
`;

const GuildItem = styled.li<{ isSelected: boolean }>`
  cursor: pointer;
  padding: 12px 16px;
  border-left: 3px solid ${({ isSelected }) => (isSelected ? '#5865F2' : 'transparent')};
  background-color: ${({ isSelected }) => (isSelected ? '#36393f' : 'transparent')};
  transition: background-color 0.2s;

  &:hover {
    background-color: #36393f;
  }
`;

const GuildItemContent = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
`;

const GuildIcon = styled.div<{ iconUrl?: string }>`
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background-color: #5865F2;
  background-image: ${({ iconUrl }) => (iconUrl ? `url(${iconUrl})` : 'none')};
  background-size: cover;
  background-position: center;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: bold;
`;

const GuildName = styled.span`
  color: #dcddde;
  font-weight: 500;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
`;

const SearchBox = styled.div`
  padding: 16px;
  border-bottom: 1px solid #36393f;
`;

const SearchInput = styled.input`
  width: 100%;
  padding: 8px 12px;
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

const EmptyState = styled.div`
  padding: 16px;
  color: #72767d;
  text-align: center;
  font-size: 14px;
`;

const ToggleButton = styled.button`
  position: fixed;
  left: 10px;
  bottom: 10px;
  z-index: 110;
  background-color: #5865F2;
  border: none;
  border-radius: 50%;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
  color: white;

  @media (min-width: 768px) {
    display: none;
  }
`;

interface SidebarProps {
  onGuildSelect: (guildId: string) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ onGuildSelect }) => {
  const { user } = useAuth();
  const { guildId } = useParams<{ guildId: string }>();
  const [isOpen, setIsOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const navigate = useNavigate();
  
  // Filter guilds based on search term
  const filteredGuilds = user?.guilds?.filter((guild) =>
    guild.name && guild.name.toLowerCase().includes(searchTerm.toLowerCase())
  ) || [];

  const handleGuildClick = (guild: Guild) => {
    onGuildSelect(guild.id);
    navigate(`/guilds/${guild.id}`);
    setIsOpen(false); // Close sidebar on mobile after selection
  };

  // Get initial letter of guild name for avatar fallback
  const getInitial = (name: string) => name && name.length > 0 ? name.charAt(0).toUpperCase() : '?';

  // Convert Discord icon hash to URL
  const getIconUrl = (guildId: string, iconHash?: string) => {
    if (!iconHash) return undefined;
    return `https://cdn.discordapp.com/icons/${guildId}/${iconHash}.png`;
  };

  return (
    <>
      <SidebarContainer isOpen={isOpen}>
        <SearchBox>
          <SearchInput
            type="text"
            placeholder="Search servers..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </SearchBox>
        <GuildList>
          {filteredGuilds.length > 0 ? (
            filteredGuilds.map((guild) => (
              <GuildItem
                key={guild.id}
                isSelected={guild.id === guildId}
                onClick={() => handleGuildClick(guild)}
              >
                <GuildItemContent>
                  <GuildIcon iconUrl={getIconUrl(guild.id, guild.icon)}>
                    {!guild.icon && getInitial(guild.name)}
                  </GuildIcon>
                  <GuildName>{guild.name}</GuildName>
                </GuildItemContent>
              </GuildItem>
            ))
          ) : (
            <EmptyState>
              {searchTerm
                ? 'No servers found matching your search'
                : 'No servers available. You need administrator permissions on a server to manage it.'}
            </EmptyState>
          )}
        </GuildList>
      </SidebarContainer>

      <ToggleButton onClick={() => setIsOpen(!isOpen)}>
        {isOpen ? '×' : '☰'}
      </ToggleButton>
    </>
  );
};

export default Sidebar;
