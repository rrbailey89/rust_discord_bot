import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { createGlobalStyle } from 'styled-components';
import { AuthProvider } from './context/AuthContext';
import Layout from './components/layout/Layout';
import ProtectedRoute from './components/auth/ProtectedRoute';
import LoginPage from './pages/Login';
import Home from './pages/Home';
import GuildManagement from './pages/GuildManagement';
import CommandConfig from './pages/CommandConfig';
import WordDetection from './pages/WordDetection';
import Settings from './pages/Settings';
import Analytics from './pages/Analytics';

// Create a placeholder component for pages we haven't built yet
const PlaceholderPage: React.FC<{ title: string }> = ({ title }) => (
  <div style={{ padding: '20px', color: 'white' }}>
    <h1>{title}</h1>
    <p>This page is coming soon!</p>
  </div>
);

// Global styles
const GlobalStyle = createGlobalStyle`
  * {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
    background-color: #36393f;
    color: #dcddde;
    line-height: 1.6;
  }

  a {
    color: #00aff4;
    text-decoration: none;
  }

  button {
    cursor: pointer;
  }
`;

// Create Query Client for React Query
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
      staleTime: 1000 * 60 * 5, // 5 minutes
    },
  },
});

const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <GlobalStyle />
        <BrowserRouter>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/" element={<Layout />}>
              <Route
                index
                element={
                  <ProtectedRoute>
                    <Home />
                  </ProtectedRoute>
                }
              />
              <Route
                path="guilds/:guildId"
                element={
                  <ProtectedRoute>
                    <GuildManagement />
                  </ProtectedRoute>
                }
              />
              <Route
                path="guilds/:guildId/commands"
                element={
                  <ProtectedRoute>
                    <CommandConfig />
                  </ProtectedRoute>
                }
              />
              <Route
                path="guilds/:guildId/word-detection"
                element={
                  <ProtectedRoute>
                    <WordDetection />
                  </ProtectedRoute>
                }
              />
              <Route
                path="guilds/:guildId/settings"
                element={
                  <ProtectedRoute>
                    <Settings />
                  </ProtectedRoute>
                }
              />
              <Route
                path="analytics"
                element={
                  <ProtectedRoute>
                    <Analytics />
                  </ProtectedRoute>
                }
              />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  );
};

export default App;
