import axios, { AxiosError, AxiosInstance, AxiosRequestConfig } from 'axios';
import { ApiError } from '../types';

// Create the API instance
const api: AxiosInstance = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 10000, // 10 seconds
});

// Request interceptor - add auth token
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('auth_token');
    if (token) {
      console.log('Adding token to request:', token.substring(0, 10) + '...');
      config.headers = config.headers || {};
      config.headers.Authorization = `Bot ${token}`;
    } else {
      console.log('No token found in localStorage');
    }
    console.log('Request headers:', JSON.stringify(config.headers));
    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor - handle common errors
api.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    // Handle 401 Unauthorized
    if (error.response?.status === 401) {
      localStorage.removeItem('auth_token');
      // Redirect to login page unless we're already there
      if (!window.location.pathname.includes('/login')) {
        window.location.href = '/login';
      }
    }

    // Format error for consistent handling
    const apiError: ApiError = {
      message: error.message || 'An unknown error occurred',
      status: error.response?.status,
    };

    if (error.response?.data && typeof error.response.data === 'object') {
      // If the server returned an error message, use it
      if ('error' in error.response.data) {
        apiError.message = error.response.data.error as string;
      } else if ('message' in error.response.data) {
        apiError.message = error.response.data.message as string;
      }
    }

    return Promise.reject(apiError);
  }
);

// Add retry logic for network issues or server errors
const MAX_RETRIES = 3;
api.interceptors.response.use(undefined, async (error: AxiosError) => {
  const { config } = error;
  
  if (!config || !error.response || config.headers['x-retry-skip']) {
    return Promise.reject(error);
  }
  
  // Only retry on network errors or 5xx errors
  if (!error.response || (error.response.status >= 500 && error.response.status < 600)) {
    config.__retryCount = (config.__retryCount || 0) + 1;
    
    if (config.__retryCount <= MAX_RETRIES) {
      // Exponential backoff
      const delay = 1000 * Math.pow(2, config.__retryCount);
      await new Promise(resolve => setTimeout(resolve, delay));
      
      // Mark this request as retried
      config.headers['x-retry-count'] = config.__retryCount;
      
      return api(config);
    }
  }
  
  return Promise.reject(error);
});

// Type augmentation for Axios
declare module 'axios' {
  export interface AxiosRequestConfig {
    __retryCount?: number;
  }
}

// API Functions

// User Information
export const fetchCurrentUser = async () => {
  const response = await api.get('/users/me');
  return response.data;
};

// Guild Settings
export const fetchGuildSettings = async (guildId: string) => {
  const response = await api.get(`/guilds/${guildId}/settings`);
  return response.data;
};

export const updateGuildSettings = async (guildId: string, settings: any) => {
  const response = await api.put(`/guilds/${guildId}/settings`, settings);
  return response.data;
};

// Guilds
export const fetchGuilds = async () => {
  const response = await api.get('/guilds');
  return response.data;
};

// Analytics
export const fetchAnalyticsData = async (params: {
  guildId?: string;
  startDate?: string;
  endDate?: string;
  eventType?: string;
}) => {
  const response = await api.get('/analytics/data', { params });
  return response.data;
};

// Renamed and added period parameter
export const fetchGuildAnalyticsSummary = async (guildId: string, period: string) => {
  // Use 'all' if guildId is 'all', otherwise pass the numeric ID
  const guildPathParam = guildId === 'all' ? 'all' : guildId;
  const response = await api.get(`/analytics/guild/${guildPathParam}`, {
    params: { period } // Pass period as query param
  });
  return response.data;
};

// Removed duplicate fetchAnalyticsData

export default api;
