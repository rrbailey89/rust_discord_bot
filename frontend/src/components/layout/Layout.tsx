import React, { useState } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router-dom';
import styled from 'styled-components';
import { useAuth } from '../../context/AuthContext';
import Header from './Header';
import Sidebar from './Sidebar';

const LayoutContainer = styled.div`
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background-color: #36393f;
`;

const ContentContainer = styled.div`
  display: flex;
  flex: 1;
`;

const MainContent = styled.main`
  flex: 1;
  padding: 20px;
  margin-left: 0;
  
  @media (min-width: 768px) {
    margin-left: 240px;
  }
`;

const PageContainer = styled.div`
  max-width: 1200px;
  margin: 0 auto;
`;

const Layout: React.FC = () => {
  const { isAuthenticated } = useAuth();
  const location = useLocation();
  const navigate = useNavigate();
  
  // Check if we're on the login page
  const isLoginPage = location.pathname === '/login';
  
  // Handler for guild selection
  const handleGuildSelect = (guildId: string) => {
    if (location.pathname.includes('/guilds/')) {
      // If already in a guild route, just update the guild ID
      const pathParts = location.pathname.split('/');
      pathParts[2] = guildId;
      navigate(pathParts.join('/'));
    } else {
      // Otherwise navigate to the guild page
      navigate(`/guilds/${guildId}`);
    }
  };

  return (
    <LayoutContainer>
      <Header />
      <ContentContainer>
        {/* Only show sidebar if authenticated and not on login page */}
        {isAuthenticated && !isLoginPage && (
          <Sidebar onGuildSelect={handleGuildSelect} />
        )}
        <MainContent>
          <PageContainer>
            <Outlet />
          </PageContainer>
        </MainContent>
      </ContentContainer>
    </LayoutContainer>
  );
};

export default Layout;
