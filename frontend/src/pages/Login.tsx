import React, { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import styled from 'styled-components';
import LoginButton from '../components/auth/LoginButton';
import { useAuth } from '../context/AuthContext';

const LoginContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: calc(100vh - 64px);
  padding: 20px;
  background-color: #36393f;
  color: white;
`;

const LoginCard = styled.div`
  background-color: #2f3136;
  border-radius: 5px;
  padding: 40px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  width: 100%;
  max-width: 400px;
  text-align: center;
`;

const Logo = styled.div`
  margin-bottom: 24px;
  
  svg {
    width: 64px;
    height: 64px;
  }
`;

const Title = styled.h1`
  font-size: 24px;
  margin-bottom: 16px;
  color: white;
`;

const Subtitle = styled.p`
  font-size: 16px;
  margin-bottom: 32px;
  color: #dcddde;
`;

const LoginPage: React.FC = () => {
  const { isAuthenticated, isLoading } = useAuth();
  const location = useLocation();
  const navigate = useNavigate();

  // Extract return URL from location state if available
  const from = location.state?.from || '/';

  // Redirect to the return URL if already authenticated
  useEffect(() => {
    if (isAuthenticated && !isLoading) {
      navigate(from, { replace: true });
    }
  }, [isAuthenticated, isLoading, navigate, from]);

  // If still loading, show nothing (handled by ProtectedRoute)
  if (isLoading) {
    return null;
  }

  return (
    <LoginContainer>
      <LoginCard>
        <Logo>
          <svg viewBox="0 0 24 24" fill="#5865F2" xmlns="http://www.w3.org/2000/svg">
            <path d="M19.952 5.672C18.048 4.141 16.023 3.671 15.895 3.66C15.694 3.643 15.503 3.757 15.421 3.941C15.415 3.953 15.349 4.104 15.276 4.339C17.362 4.732 18.908 5.529 20.31 6.765C20.533 6.903 20.603 7.197 20.464 7.421C20.37 7.572 20.205 7.655 20.034 7.655C19.947 7.655 19.859 7.633 19.779 7.587C17.059 6.018 14.075 5.724 11.999 5.724C9.923 5.724 6.939 6.018 4.219 7.587C3.997 7.718 3.703 7.648 3.572 7.421C3.441 7.199 3.511 6.904 3.733 6.765C5.135 5.529 6.681 4.732 8.767 4.339C8.694 4.104 8.628 3.953 8.622 3.941C8.54 3.757 8.349 3.643 8.148 3.66C8.02 3.671 5.995 4.141 4.091 5.672C3.188 6.449 1.3 10.58 1.3 14.281C1.3 14.345 1.315 14.407 1.344 14.462C2.521 16.77 5.893 17.529 6.825 17.561C6.826 17.561 6.828 17.561 6.829 17.561C7.034 17.561 7.227 17.447 7.315 17.263L7.994 16.007C5.83 15.393 4.722 14.351 4.66 14.293C4.46 14.101 4.453 13.78 4.644 13.579C4.836 13.377 5.157 13.37 5.358 13.563C5.386 13.588 7.222 15.215 11.999 15.215C16.789 15.215 18.625 13.577 18.642 13.563C18.843 13.371 19.164 13.378 19.355 13.58C19.547 13.781 19.54 14.102 19.34 14.294C19.278 14.352 18.17 15.394 16.006 16.008L16.685 17.264C16.773 17.448 16.966 17.562 17.171 17.562C17.172 17.562 17.174 17.562 17.175 17.562C18.107 17.53 21.479 16.77 22.656 14.462C22.685 14.407 22.7 14.345 22.7 14.281C22.7 10.58 20.812 6.449 19.952 5.672Z" />
          </svg>
        </Logo>
        <Title>Discord Bot Management</Title>
        <Subtitle>Sign in with your Discord account to manage your bot settings</Subtitle>
        <LoginButton />
      </LoginCard>
    </LoginContainer>
  );
};

export default LoginPage;
