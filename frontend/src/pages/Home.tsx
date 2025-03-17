import React from 'react';
import styled from 'styled-components';
import { useAuth } from '../context/AuthContext';
import GuildList from '../components/guilds/GuildList';

const Container = styled.div`
  padding: 24px;
`;

const Header = styled.div`
  margin-bottom: 24px;
`;

const Title = styled.h1`
  color: #ffffff;
  margin: 0 0 8px 0;
  font-size: 28px;
`;

const Subtitle = styled.p`
  color: #b9bbbe;
  margin: 0;
  font-size: 16px;
  line-height: 1.5;
`;

const WelcomeSection = styled.div`
  background-color: #5865F2;
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 24px;
  color: white;
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

const WelcomeText = styled.div`
  flex: 1;
  
  h2 {
    margin: 0 0 8px 0;
    font-size: 24px;
  }
  
  p {
    margin: 0;
    opacity: 0.9;
  }
`;

const WelcomeButton = styled.button`
  background-color: white;
  color: #5865F2;
  border: none;
  border-radius: 4px;
  padding: 10px 16px;
  font-weight: 600;
  cursor: pointer;
  margin-left: 24px;
  transition: background-color 0.2s;
  
  &:hover {
    background-color: #f6f6f7;
  }
`;

const Card = styled.div`
  background-color: #2f3136;
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 24px;
`;

const CardTitle = styled.h3`
  color: #ffffff;
  margin: 0 0 16px 0;
  font-size: 18px;
`;

const StepsList = styled.ol`
  margin: 0;
  padding-left: 24px;
  color: #dcddde;
  
  li {
    margin-bottom: 12px;
  }
`;

const Home: React.FC = () => {
  const { user } = useAuth();
  
  return (
    <Container>
      <Header>
        <Title>Dashboard</Title>
        <Subtitle>Manage your Discord bot across all your servers.</Subtitle>
      </Header>
      
      <WelcomeSection>
        <WelcomeText>
          <h2>Welcome, {user?.username || 'Discord User'}!</h2>
          <p>Manage your bot, configure commands, and set up word detection rules for your servers.</p>
        </WelcomeText>
        <WelcomeButton>View Documentation</WelcomeButton>
      </WelcomeSection>
      
      {(!user?.guilds || user.guilds.length === 0) && (
        <Card>
          <CardTitle>Getting Started</CardTitle>
          <StepsList>
            <li>Add the bot to your Discord server using the invite link.</li>
            <li>Make sure you have administrator permissions on the server.</li>
            <li>Refresh this page to see your servers appear in the list below.</li>
            <li>Click on a server to start configuring the bot for that server.</li>
          </StepsList>
        </Card>
      )}
      
      <GuildList />
    </Container>
  );
};

export default Home;
