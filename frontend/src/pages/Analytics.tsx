import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useLocation } from 'react-router-dom'; // Import useLocation
import styled from 'styled-components';
import { fetchAnalyticsData, fetchAnalyticsSummary, fetchGuilds } from '../services/api'; // Added fetchGuilds
import AnalyticsChart from '../components/analytics/AnalyticsChart';
import AnalyticsSummary from '../components/analytics/AnalyticsSummary';
import { Guild, GuildAnalyticsSummary, CommandUsage } from '../types'; // Import Guild, GuildAnalyticsSummary, CommandUsage types

// Define types for the data we expect from the API
// SummaryStat is now defined in AnalyticsSummary component, but we'll keep it here for clarity if needed elsewhere
interface SummaryStat {
  title: string;
  value: number | string;
  change?: number; // Optional change percentage
}

// Assuming fetchAnalyticsData returns an object with these structures
// Note: CommandUsageData might need adjustment based on actual API response for charts
interface CommandUsageData {
  name: string; // e.g., month or day or command name
  count: number; // count for that command/period
  // Potentially other fields depending on how the backend aggregates
}

interface UserActivityData {
  name: string; // e.g., day of the week or date
  messages: number;
  commands: number;
}

interface AnalyticsData {
  commandUsage: CommandUsageData[];
  userActivity: UserActivityData[];
  // Add other potential data structures returned by fetchAnalyticsData
}


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

// Analytics page component
const Analytics: React.FC = () => {
  const location = useLocation(); // Get location object
  const queryParams = new URLSearchParams(location.search);
  const initialGuildId = queryParams.get('guildId') || 'all'; // Get guildId from URL or default to 'all'

  const [timeRange, setTimeRange] = useState('30d');
  const [guildFilter, setGuildFilter] = useState(initialGuildId); // Initialize with guildId from URL
  const [startDate, setStartDate] = useState('');
  const [endDate, setEndDate] = useState('');

  // Update guildFilter if URL query param changes
  useEffect(() => {
    const guildIdFromUrl = queryParams.get('guildId') || 'all';
    if (guildIdFromUrl !== guildFilter) {
      setGuildFilter(guildIdFromUrl);
    }
  }, [location.search]); // Rerun effect when URL search params change

  // Fetch guilds for the dropdown
  const { data: guildsData, isLoading: guildsLoading } = useQuery<Guild[]>({
    queryKey: ['guilds'],
    queryFn: fetchGuilds, // Assuming fetchGuilds is imported from api.ts
  });

  const guilds = [
    { id: 'all', name: 'All Guilds' },
    ...(guildsData || []).map(g => ({ id: g.id, name: g.name }))
  ];

  const {
    data: summaryData, // This will be GuildAnalyticsSummary | undefined
    isLoading: summaryLoading,
    error: summaryError
  } = useQuery<GuildAnalyticsSummary>({ // Expect GuildAnalyticsSummary
    // Summary likely doesn't depend on time range, only guild filter
    queryKey: ['analytics-summary', guildFilter], 
    queryFn: () => fetchAnalyticsSummary(guildFilter !== 'all' ? guildFilter : undefined), // Pass only guildId or undefined
    // enabled: true, // Fetch always
  });

  const {
    data: analyticsData,
    isLoading: analyticsLoading,
    error: analyticsError
  } = useQuery<AnalyticsData>({
    queryKey: ['analytics-data', guildFilter, timeRange, startDate, endDate],
    queryFn: () => {
      // Prepare parameters for the API call
      const params: { guildId?: string; startDate?: string; endDate?: string; timeRange?: string } = {};
      if (guildFilter !== 'all') {
        params.guildId = guildFilter;
      }
      if (timeRange === 'custom' && startDate && endDate) {
        params.startDate = startDate;
        params.endDate = endDate;
      } else if (timeRange !== 'custom') {
        // Pass the preset time range (e.g., '7d', '30d') if not custom
        // The backend needs to handle these presets
        params.timeRange = timeRange;
      }
      
      return fetchAnalyticsData(params);
    },
    // enabled: true, // Fetch always, whether 'all' or specific guild
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

  // Transform summaryData into the format expected by AnalyticsSummary component
  const transformedSummaryStats: SummaryStat[] = summaryData ? [
    { title: 'Active Users', value: summaryData.active_users },
    { title: 'Commands Used', value: summaryData.commands_used },
    { title: 'Messages Sent', value: summaryData.message_count },
    // Add more stats derived from events_by_type if needed
    ...Object.entries(summaryData.events_by_type).map(([key, value]) => ({
      title: `Events: ${key}`, // Example transformation
      value: value
    }))
  ] : [];

  // TODO: Add logic to derive chart data keys dynamically from fetched data
  // This needs adjustment based on the actual structure of analyticsData.commandUsage
  const commandUsageKeys = analyticsData?.commandUsage?.[0]
    ? Object.keys(analyticsData.commandUsage[0]).filter(key => key !== 'name' && key !== 'command_id') // Exclude non-numeric/category keys
    : [];
  const userActivityKeys = ['messages', 'commands'];


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
            disabled={guildsLoading} // Disable while loading guilds
          >
            {guildsLoading ? (
              <option>Loading servers...</option>
            ) : (
              guilds.map(guild => (
                <option key={guild.id} value={guild.id}>{guild.name}</option>
              ))
            )}
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
        stats={transformedSummaryStats} // Use the transformed data
        isLoading={summaryLoading}
      />

      {/* Charts */}
      {analyticsData && (
        <>
          <AnalyticsChart
            title="Command Usage"
            data={analyticsData?.commandUsage || []} // Use fetched data or empty array
            dataKeys={commandUsageKeys} // Use dynamic keys
            xAxisDataKey="name"
            // isLoading={analyticsLoading} // Remove isLoading prop
          />

          <AnalyticsChart
            title="User Activity"
            data={analyticsData?.userActivity || []} // Use fetched data or empty array
            dataKeys={userActivityKeys}
            xAxisDataKey="name"
            // isLoading={analyticsLoading} // Remove isLoading prop
          />
        </>
      )}
    </PageContainer>
  );
};

export default Analytics;
