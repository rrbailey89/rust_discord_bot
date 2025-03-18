import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { User, AuthResponse } from '../types';
import api from '../services/api';

interface AuthContextType {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: () => void;
  logout: () => void;
  handleAuthCallback: (token: string) => void;
}

// Create a context with a default value
const AuthContext = createContext<AuthContextType>({
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: true,
  login: () => {},
  logout: () => {},
  handleAuthCallback: () => {},
});

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  // Function to get the auth token from cookies
  const getTokenFromCookie = (): string | null => {
    const cookies = document.cookie.split(';');
    for (let cookie of cookies) {
      cookie = cookie.trim();
      // Find the auth_token cookie
      if (cookie.startsWith('auth_token=')) {
        return cookie.substring('auth_token='.length);
      }
    }
    return null;
  };

  // Load authentication state from localStorage or cookies on initial render
  useEffect(() => {
    const loadAuth = async () => {
      try {
        console.log('AuthContext: Loading authentication state');
        // Try localStorage first
        let storedToken = localStorage.getItem('auth_token');
        
        // If not in localStorage, try cookies
        if (!storedToken) {
          console.log('Token not found in localStorage, checking cookies...');
          storedToken = getTokenFromCookie();
          if (storedToken) {
            console.log('Found token in cookies');
            // If found in cookies, also save to localStorage for consistency
            localStorage.setItem('auth_token', storedToken);
          }
        }
        
        if (storedToken) {
          try {
            console.log('Processing token:', storedToken.substring(0, 10) + '...');
            
            // Validate token format
            const tokenParts = storedToken.split('.');
            if (tokenParts.length !== 3) {
              throw new Error('Invalid token format (not a JWT)');
            }
            
      // Decode JWT to get user info
      const payload = JSON.parse(atob(tokenParts[1]));
      
      // Verify token expiration
      if (!payload.exp) {
        throw new Error('Token missing expiration');
      }
      
      if (payload.exp * 1000 > Date.now()) {
        // Verify user info structure
        if (!payload.user || !payload.user.id || !payload.user.username) {
          throw new Error('Token payload missing required user information');
        }
        
        // Log full user info for debugging
        console.log('User info from token:', payload.user);
        
        // Check if guilds array exists
        if (!Array.isArray(payload.user.guilds)) {
          console.warn('Token missing guilds array, defaulting to empty array');
          payload.user.guilds = [];
        }
        
        // If guilds exist but are just strings (IDs), fetch the full guild objects
        if (payload.user.guilds.length > 0 && typeof payload.user.guilds[0] === 'string') {
          console.log('JWT contains guild IDs, fetching full guild details from API...');
          try {
            // Make an API call to get user details with full guild objects
            const userResponse = await api.get('/api/users/me');
            if (userResponse.data && userResponse.data.guilds) {
              console.log('Received full guild details:', userResponse.data.guilds);
              payload.user.guilds = userResponse.data.guilds;
            }
          } catch (apiError) {
            console.error('Failed to fetch user details with guild information:', apiError);
          }
        }
        
        setToken(storedToken);
        setUser(payload.user);
              
              // Set token in API headers
              if (api.defaults.headers) {
                api.defaults.headers.common['Authorization'] = `Bearer ${storedToken}`;
                console.log('Authorization header set with token from cookie/localStorage');
              }
            } else {
              // Token expired
              console.warn('Token expired, removing from storage', {
                expired: new Date(payload.exp * 1000),
                now: new Date()
              });
              localStorage.removeItem('auth_token');
              document.cookie = 'auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
            }
          } catch (e) {
            console.error('Error parsing or validating token', e);
            localStorage.removeItem('auth_token');
            document.cookie = 'auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
          }
        } else {
          console.log('No authentication token found');
        }
      } catch (e) {
        console.error('Unexpected error in auth initialization', e);
      } finally {
        setIsLoading(false);
      }
    };

    loadAuth();
  }, []);

  const login = () => {
    // Redirect to the Discord OAuth login endpoint
    window.location.href = '/api/auth/login';
  };

  const logout = async () => {
    console.log('Logout function called');
    try {
      // Clear token in API headers if exists
      if (api.defaults.headers) {
        console.log('Removing Authorization header');
        delete api.defaults.headers.common['Authorization'];
      }
      
      // Log current cookies
      console.log('Current cookies before logout:', document.cookie);
      
      // Call server-side logout endpoint with some retry logic
      console.log('Calling server-side logout endpoint');
      let logoutSuccess = false;
      
      try {
        const response = await api.post('/api/auth/logout');
        console.log('Logout API response:', response.status, response.data);
        logoutSuccess = true;
      } catch (apiError) {
        console.error('Logout API error:', apiError);
        
        // Try a second approach - direct fetch with credentials
        try {
          console.log('Trying direct fetch for logout');
          const fetchResponse = await fetch('/api/auth/logout', {
            method: 'POST',
            credentials: 'include'
          });
          
          if (fetchResponse.ok) {
            console.log('Direct fetch logout succeeded:', fetchResponse.status);
            logoutSuccess = true;
          } else {
            console.error('Direct fetch logout failed:', fetchResponse.status);
          }
        } catch (fetchError) {
          console.error('Direct fetch logout error:', fetchError);
        }
      }
    } catch (e) {
      console.error('Logout error:', e);
    } finally {
      console.log('Cleaning up client-side auth data');
      
      // Clear client-side storage
      localStorage.removeItem('auth_token');
      console.log('Removed auth_token from localStorage');
      
      // Clear cookies - both the http-only and js-accessible versions
      document.cookie = 'auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
      document.cookie = 'token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
      console.log('Cleared auth cookies');
      
      // Double check cookies are cleared
      console.log('Cookies after clearing:', document.cookie);
      
      // Update state
      setToken(null);
      setUser(null);
      console.log('Reset auth state');
      
      // Redirect to home page after logout
      console.log('Redirecting to home page');
      window.location.href = '/';
    }
  };

  const handleAuthCallback = async (newToken: string) => {
    try {
      console.log('Received token:', newToken.substring(0, 10) + '...');
      
      // Validate token structure
      const tokenParts = newToken.split('.');
      if (tokenParts.length !== 3) {
        throw new Error('Invalid token format');
      }
      
      // Parse and validate payload
      const payload = JSON.parse(atob(tokenParts[1]));
      if (!payload.user) {
        throw new Error('Token missing user information');
      }
      
      if (!payload.user.guilds) {
        console.warn('Token payload missing guilds, defaulting to empty array');
        payload.user.guilds = [];
      }
      
      // Add token to all future API requests
      if (api.defaults.headers) {
        api.defaults.headers.common['Authorization'] = `Bearer ${newToken}`;
      }
      
      // If guilds exist but are just strings (IDs), fetch the full guild objects
      if (payload.user.guilds.length > 0 && typeof payload.user.guilds[0] === 'string') {
        console.log('JWT contains guild IDs, fetching full guild details from API...');
        try {
          // Make an API call to get user details with full guild objects
          const userResponse = await api.get('/api/users/me');
          if (userResponse.data && userResponse.data.guilds) {
            console.log('Received full guild details:', userResponse.data.guilds);
            payload.user.guilds = userResponse.data.guilds;
          }
        } catch (apiError) {
          console.error('Failed to fetch user details with guild information:', apiError);
        }
      }
      
      // Store the token
      localStorage.setItem('auth_token', newToken);
      
      // Update state
      setToken(newToken);
      setUser(payload.user);
      
      console.log('User authenticated successfully:', {
        username: payload.user?.username,
        id: payload.user?.id,
        guildsCount: payload.user?.guilds?.length || 0
      });
    } catch (e) {
      console.error('Error handling auth callback:', e);
      // Clear any invalid tokens
      localStorage.removeItem('auth_token');
      document.cookie = 'auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
      
      // Reset state
      setToken(null);
      setUser(null);
      
      // Re-throw for caller to handle UI feedback
      throw e;
    }
  };

  const value = {
    user,
    token,
    isAuthenticated: !!token,
    isLoading,
    login,
    logout,
    handleAuthCallback,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

// Custom hook to use the auth context
export const useAuth = () => useContext(AuthContext);

export default AuthContext;
