import React, { useEffect, useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import styled from 'styled-components';

const LoadingContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background-color: #36393f;
  color: white;
`;

const Spinner = styled.div`
  border: 4px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top: 4px solid #5865F2;
  width: 40px;
  height: 40px;
  margin-bottom: 20px;
  animation: spin 1s linear infinite;
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
`;

const Message = styled.div`
  font-size: 18px;
  margin-bottom: 10px;
`;

const ErrorMessage = styled.div`
  color: #ED4245;
  margin-top: 10px;
`;

const AuthCallback: React.FC = () => {
  const { handleAuthCallback } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const processCallback = async () => {
      try {
        // First, check if we have a token in URL parameters
        const params = new URLSearchParams(location.search);
        const token = params.get('token');
        const error = params.get('error');
        
        // If there's an error, display it
        if (error) {
          setError(decodeURIComponent(error));
          return;
        }
        
        // If we have a token directly in the URL, use it
        if (token) {
          handleAuthCallback(token);
          navigate('/', { replace: true });
          return;
        }
        
        // For direct JSON response processing (fallback)
        const responseText = document.body.textContent;
        if (responseText) {
          try {
            const responseData = JSON.parse(responseText);
            if (responseData.token) {
              handleAuthCallback(responseData.token);
              navigate('/', { replace: true });
              return;
            }
          } catch (err) {
            console.error('Error parsing response:', err);
          }
        }

        // For OAuth2 code parameter processing
        const code = params.get('code');
        if (code) {
          // Redirect to our backend for processing
          window.location.href = `/api/auth/callback?code=${code}`;
          return;
        }

        setError('No authentication information found in the URL');
      } catch (err) {
        console.error('Authentication error:', err);
        setError(err instanceof Error ? err.message : 'Authentication failed');
      }
    };

    processCallback();
  }, [handleAuthCallback, navigate, location]);

  return (
    <LoadingContainer>
      {error ? (
        <>
          <Message>Authentication Failed</Message>
          <ErrorMessage>{error}</ErrorMessage>
        </>
      ) : (
        <>
          <Spinner />
          <Message>Completing authentication...</Message>
        </>
      )}
    </LoadingContainer>
  );
};

export default AuthCallback;
