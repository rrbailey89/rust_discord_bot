import React from 'react';
import { useParams, Navigate } from 'react-router-dom';
import styled from 'styled-components';
import SettingsForm from '../components/settings/SettingsForm';

const PageContainer = styled.div`
  padding: 20px;
`;

const PageTitle = styled.h1`
  color: #ffffff;
  font-size: 24px;
  margin-bottom: 20px;
`;

const InfoText = styled.p`
  color: #b9bbbe;
  margin-bottom: 24px;
`;

const Settings: React.FC = () => {
  const { guildId } = useParams<{ guildId: string }>();
  
  // If no guild ID is provided, redirect to home
  if (!guildId) {
    return <Navigate to="/" replace />;
  }
  
  return (
    <PageContainer>
      <PageTitle>Guild Settings</PageTitle>
      <InfoText>
        Configure your bot settings for this server. Changes will take effect immediately.
      </InfoText>
      
      <SettingsForm />
    </PageContainer>
  );
};

export default Settings;
