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
            // Decode JWT to get user info
            const payload = JSON.parse(atob(storedToken.split('.')[1]));
            if (payload.exp * 1000 > Date.now()) {
              setToken(storedToken);
              setUser(payload.user);
              
              // Set token in API headers
              if (api.defaults.headers) {
                api.defaults.headers.common['Authorization'] = `Bearer ${storedToken}`;
                console.log('Authorization header set with token from cookie/localStorage');
              }
            } else {
              // Token expired
              console.log('Token expired, removing from storage');
              localStorage.removeItem('auth_token');
              document.cookie = 'auth_token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
            }
          } catch (e) {
            console.error('Error parsing token', e);
            localStorage.removeItem('auth_token');
          }
        }
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

  const logout = () => {
    localStorage.removeItem('auth_token');
    setToken(null);
    setUser(null);
    // Redirect to home page after logout
    window.location.href = '/';
  };

  const handleAuthCallback = (newToken: string) => {
    try {
      console.log('Received token:', newToken.substring(0, 10) + '...');
      localStorage.setItem('auth_token', newToken);
      const payload = JSON.parse(atob(newToken.split('.')[1]));
      setToken(newToken);
      setUser(payload.user);
      
      // Add token to all future API requests
      if (api.defaults.headers) {
        api.defaults.headers.common['Authorization'] = `Bearer ${newToken}`;
      }
      
      console.log('User authenticated:', payload.user?.username);
    } catch (e) {
      console.error('Error handling auth callback', e);
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
