import React, { useEffect, useState } from 'react';
import { ApiError } from '../types';
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
  const [detailedError, setDetailedError] = useState<ApiError | null>(null);

  useEffect(() => {
    const processCallback = async () => {
      try {
        console.log('Starting auth callback processing');
        // First, check if we have a token in URL parameters
        const params = new URLSearchParams(location.search);
        const token = params.get('token');
        const errorParam = params.get('error');
        
        // If there's an error, display it
        if (errorParam) {
          const decodedError = decodeURIComponent(errorParam);
          console.error('Authentication error from URL param:', decodedError);
          setError(decodedError);
          return;
        }
        
        // If we have a token directly in the URL, use it
        if (token) {
          console.log('Found token in URL parameters, validating structure');
          try {
            // Verify token has correct structure (header.payload.signature)
            const tokenParts = token.split('.');
            if (tokenParts.length !== 3) {
              throw new Error('Invalid token format (not a valid JWT)');
            }
            
            // Try to decode the payload to verify it's valid JSON
            const payload = JSON.parse(atob(tokenParts[1]));
            
            // Verify user info exists in payload
            if (!payload.user || !payload.user.id) {
              throw new Error('Token missing user information');
            }
            
            console.log('Token validation successful, storing token');
            // Store in cookie for server-side use
            document.cookie = `auth_token=${token}; path=/; max-age=86400; SameSite=Strict`;
            handleAuthCallback(token);
            console.log('Redirecting to home page');
            navigate('/', { replace: true });
            return;
          } catch (err) {
            console.error('Token validation error:', err);
            setError('Invalid authentication token structure');
            setDetailedError({
              message: 'Token validation failed',
              code: 'INVALID_TOKEN',
              status: 400
            });
            return;
          }
        }
        
        // For direct JSON response processing (fallback)
        const responseText = document.body.textContent;
        if (responseText) {
          try {
            console.log('Attempting to parse body content as JSON');
            const responseData = JSON.parse(responseText);
            if (responseData.token) {
              console.log('Found token in response body, processing');
              handleAuthCallback(responseData.token);
              navigate('/', { replace: true });
              return;
            } else {
              console.warn('Response body contained JSON but no token', responseData);
            }
          } catch (err) {
            console.error('Error parsing response body as JSON:', err);
          }
        }

        // For OAuth2 code parameter processing
        const code = params.get('code');
        if (code) {
          console.log('Found OAuth code parameter, redirecting to backend');
          // Redirect to our backend for processing
          window.location.href = `/api/auth/callback?code=${code}`;
          return;
        }

        console.error('No authentication information found in the URL');
        setError('No authentication information found in the URL');
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Authentication failed';
        console.error('Authentication callback error:', errorMessage, err);
        setError(errorMessage);
        setDetailedError({
          message: errorMessage,
          code: 'AUTH_CALLBACK_ERROR',
          status: 500
        });
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
          {detailedError && (
            <ErrorMessage>
              {detailedError.code && `Error code: ${detailedError.code}`}
              {detailedError.status && ` (${detailedError.status})`}
            </ErrorMessage>
          )}
          <button 
            style={{ 
              marginTop: '20px', 
              padding: '10px 20px', 
              backgroundColor: '#5865F2', 
              color: 'white', 
              border: 'none', 
              borderRadius: '4px',
              cursor: 'pointer'
            }}
            onClick={() => navigate('/login')}
          >
            Return to Login
          </button>
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
