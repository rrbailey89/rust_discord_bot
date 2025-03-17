import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import styled from 'styled-components';
import { fetchAnalyticsData, fetchAnalyticsSummary } from '../services/api';
import AnalyticsChart from '../components/analytics/AnalyticsChart';
import AnalyticsSummary from '../components/analytics/AnalyticsSummary';

const PageContainer = styled.div`
  padding: 20px;
`;

const PageTitle = styled.h1`
  color: #ffffff;
  font-size: 24px;
  margin-bottom: 20px;
`;

const FiltersContainer = styled.div`
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
  flex-wrap: wrap;
`;

const FilterGroup = styled.div`
  display: flex;
  flex-direction: column;
`;

const FilterLabel = styled.label`
  color: #b9bbbe;
  font-size: 14px;
  margin-bottom: 8px;
`;

const FilterSelect = styled.select`
  background-color: #40444b;
  border: 1px solid #202225;
  border-radius: 4px;
  color: #dcddde;
  padding: 8px 12px;
  font-size: 14px;
  min-width: 150px;
  
  &:focus {
    outline: none;
    border-color: #5865F2;
  }
`;

const DateInput = styled.input`
  background-color: #40444b;
  border: 1px solid #202225;
  border-radius: 4px;
  color: #dcddde;
  padding: 8px 12px;
  font-size: 14px;
  
  &:focus {
    outline: none;
    border-color: #5865F2;
  }
  
  &::-webkit-calendar-picker-indicator {
    filter: invert(0.8);
  }
`;

const ErrorMessage = styled.div`
  background-color: rgba(237, 66, 69, 0.1);
  border-left: 4px solid #ED4245;
  color: #ffffff;
  padding: 12px;
  margin-bottom: 20px;
  border-radius: 4px;
`;

// Mock data for development purposes
const MOCK_SUMMARY_DATA = [
  { title: 'Total Commands Used', value: 12453, change: 5.2 },
  { title: 'Active Users', value: 387, change: 2.1 },
  { title: 'Messages Processed', value: 24789, change: -1.3 },
  { title: 'Rules Triggered', value: 126, change: 7.8 },
];

const MOCK_COMMAND_USAGE = [
  { name: 'January', help: 120, ban: 45, kick: 30, mute: 80, play: 150 },
  { name: 'February', help: 132, ban: 42, kick: 25, mute: 75, play: 165 },
  { name: 'March', help: 141, ban: 48, kick: 32, mute: 92, play: 158 },
  { name: 'April', help: 154, ban: 51, kick: 27, mute: 85, play: 172 },
  { name: 'May', help: 162, ban: 49, kick: 31, mute: 79, play: 181 },
  { name: 'June', help: 159, ban: 53, kick: 35, mute: 94, play: 169 },
];

const MOCK_USER_ACTIVITY = [
  { name: 'Monday', messages: 430, commands: 86 },
  { name: 'Tuesday', messages: 520, commands: 104 },
  { name: 'Wednesday', messages: 580, commands: 116 },
  { name: 'Thursday', messages: 540, commands: 108 },
  { name: 'Friday', messages: 610, commands: 122 },
  { name: 'Saturday', messages: 720, commands: 144 },
  { name: 'Sunday', messages: 650, commands: 130 },
];

// Analytics page component
const Analytics: React.FC = () => {
  const [timeRange, setTimeRange] = useState('30d');
  const [guildFilter, setGuildFilter] = useState('all');
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');
  
  // We'll use mock data for now to demonstrate the UI
  // In a real implementation, these would be API calls
  
  // Define types for our mock data
  type SummaryData = typeof MOCK_SUMMARY_DATA;
  
  interface AnalyticsData {
    commandUsage: typeof MOCK_COMMAND_USAGE;
    userActivity: typeof MOCK_USER_ACTIVITY;
  }
  
  const {
    data: summaryData,
    isLoading: summaryLoading,
    error: summaryError
  } = useQuery<SummaryData>({
    queryKey: ['analytics-summary', guildFilter],
    queryFn: () => {
      // In a real implementation, this would call the API
      // return fetchAnalyticsSummary(guildFilter !== 'all' ? guildFilter : undefined);
      
      // For now, return mock data with a delay to simulate API call
      return new Promise<SummaryData>(resolve => {
        setTimeout(() => resolve(MOCK_SUMMARY_DATA), 500);
      });
    }
  });
  
  const {
    data: analyticsData,
    isLoading: analyticsLoading,
    error: analyticsError
  } = useQuery<AnalyticsData>({
    queryKey: ['analytics-data', guildFilter, timeRange, startDate, endDate],
    queryFn: () => {
      // In a real implementation, this would call the API
      /*
      return fetchAnalyticsData({
        guildId: guildFilter !== 'all' ? guildFilter : undefined,
        startDate: startDate || undefined,
        endDate: endDate || undefined,
      });
      */
      
      // For now, return mock data with a delay to simulate API call
      return new Promise<AnalyticsData>(resolve => {
        setTimeout(() => resolve({
          commandUsage: MOCK_COMMAND_USAGE,
          userActivity: MOCK_USER_ACTIVITY
        }), 700);
      });
    }
  });
  
  // Handle time range change
  const handleTimeRangeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    setTimeRange(e.target.value);
    
    // Clear custom date range if a preset is selected
    if (e.target.value !== 'custom') {
      setStartDate('');
      setEndDate('');
    }
  };
  
  // Get guilds for the filter dropdown - would be fetched from API in real implementation
  const guilds = [
    { id: 'all', name: 'All Guilds' },
    { id: '123456789', name: 'Server 1' },
    { id: '987654321', name: 'Server 2' },
    { id: '456123789', name: 'Server 3' },
  ];
  
  return (
    <PageContainer>
      <PageTitle>Analytics Dashboard</PageTitle>
      
      {/* Filters */}
      <FiltersContainer>
        <FilterGroup>
          <FilterLabel htmlFor="guild-filter">Server</FilterLabel>
          <FilterSelect
            id="guild-filter"
            value={guildFilter}
            onChange={(e) => setGuildFilter(e.target.value)}
          >
            {guilds.map(guild => (
              <option key={guild.id} value={guild.id}>{guild.name}</option>
            ))}
          </FilterSelect>
        </FilterGroup>
        
        <FilterGroup>
          <FilterLabel htmlFor="time-range">Time Range</FilterLabel>
          <FilterSelect
            id="time-range"
            value={timeRange}
            onChange={handleTimeRangeChange}
          >
            <option value="7d">Last 7 days</option>
            <option value="30d">Last 30 days</option>
            <option value="90d">Last 90 days</option>
            <option value="custom">Custom Range</option>
          </FilterSelect>
        </FilterGroup>
        
        {timeRange === 'custom' && (
          <>
            <FilterGroup>
              <FilterLabel htmlFor="start-date">Start Date</FilterLabel>
              <DateInput
                id="start-date"
                type="date"
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
              />
            </FilterGroup>
            
            <FilterGroup>
              <FilterLabel htmlFor="end-date">End Date</FilterLabel>
              <DateInput
                id="end-date"
                type="date"
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
              />
            </FilterGroup>
          </>
        )}
      </FiltersContainer>
      
      {/* Display errors if any */}
      {(summaryError || analyticsError) && (
        <ErrorMessage>
          Error loading analytics data. Please try again later.
        </ErrorMessage>
      )}
      
      {/* Summary stats */}
      <AnalyticsSummary
        stats={summaryData || []}
        isLoading={summaryLoading}
      />
      
      {/* Charts */}
      {analyticsData && (
        <>
          <AnalyticsChart
            title="Command Usage"
            data={analyticsData.commandUsage}
            dataKeys={['help', 'ban', 'kick', 'mute', 'play']}
            xAxisDataKey="name"
          />
          
          <AnalyticsChart
            title="User Activity"
            data={analyticsData.userActivity}
            dataKeys={['messages', 'commands']}
            xAxisDataKey="name"
          />
        </>
      )}
    </PageContainer>
  );
};

export default Analytics;
