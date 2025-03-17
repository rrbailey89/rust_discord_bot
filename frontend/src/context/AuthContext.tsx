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

  // Load authentication state from localStorage on initial render
  useEffect(() => {
    const loadAuth = async () => {
      try {
        const storedToken = localStorage.getItem('auth_token');
        if (storedToken) {
          try {
            // Decode JWT to get user info
            const payload = JSON.parse(atob(storedToken.split('.')[1]));
            if (payload.exp * 1000 > Date.now()) {
              setToken(storedToken);
              setUser(payload.user);
            } else {
              // Token expired
              localStorage.removeItem('auth_token');
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
      localStorage.setItem('auth_token', newToken);
      const payload = JSON.parse(atob(newToken.split('.')[1]));
      setToken(newToken);
      setUser(payload.user);
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
