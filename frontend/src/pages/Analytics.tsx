import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useLocation } from 'react-router-dom';
import styled from 'styled-components';
// Removed fetchAnalyticsData, fetchAnalyticsSummary might need adjustment or be replaced
import { fetchGuilds, fetchGuildAnalyticsSummary } from '../services/api'; // Use specific summary fetch
import AnalyticsChart from '../components/analytics/AnalyticsChart';
import AnalyticsSummary from '../components/analytics/AnalyticsSummary';
// Ensure GuildAnalyticsSummary type includes the new fields from backend models
import { Guild, GuildAnalyticsSummary, TimeSeriesDataPoint, UserActivityDataPoint } from '../types';

// Type for summary stats passed to the component
interface SummaryStat {
  title: string;
  value: number | string;
  change?: number;
}

// No longer need separate AnalyticsData interface, GuildAnalyticsSummary holds everything

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

  // Default time range to 'week' to match backend default if needed
  const [timeRange, setTimeRange] = useState('week');
  const [guildFilter, setGuildFilter] = useState(initialGuildId);
  const [startDate, setStartDate] = useState(''); // Keep for custom range UI
  const [endDate, setEndDate] = useState(''); // Keep for custom range UI

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
    queryFn: fetchGuilds,
  });

  // Prepare guilds list for dropdown, ensuring 'all' is present
  const guilds = [
    { id: 'all', name: 'All Guilds' },
    ...(guildsData || []).map(g => ({ id: g.id, name: g.name }))
  ];

  // Single query to fetch all analytics data (summary + charts)
  const {
    data: summaryData, // This is now GuildAnalyticsSummary including chart data
    isLoading: summaryLoading,
    error: summaryError,
    // refetch // Optional: if you want manual refetching
  } = useQuery<GuildAnalyticsSummary>({
    // Query key includes all dependencies
    queryKey: ['guildAnalytics', guildFilter, timeRange, startDate, endDate],
    queryFn: () => {
      // Determine the period parameter based on state
      let period = timeRange;
      if (timeRange === 'custom') {
        // Backend doesn't support custom date ranges via query params directly in this plan
        // We'll stick to preset periods for now.
        // If custom range is needed, backend API needs adjustment.
        // For now, maybe default custom to 'week' or 'month'? Let's use 'week'.
        console.warn("Custom date range selected, but backend currently uses preset periods. Using 'week'.");
        period = 'week';
        // Or pass startDate/endDate if backend is updated later
      }
      // Pass guildId ('all' or numeric ID) and the determined period
      return fetchGuildAnalyticsSummary(guildFilter, period);
    },
    // Keep data fresh or stale as needed
    // staleTime: 5 * 60 * 1000, // e.g., 5 minutes
    // refetchOnWindowFocus: false,
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

  // Transform summaryData for the summary boxes
  const transformedSummaryStats: SummaryStat[] = summaryData ? [
    { title: 'Active Users', value: summaryData.active_users ?? 'N/A' },
    { title: 'Commands Used', value: summaryData.commands_used ?? 'N/A' },
    { title: 'Messages Sent', value: summaryData.message_count ?? 'N/A' },
    // Example: Add top event type if available
    // ...(summaryData.events_by_type && Object.keys(summaryData.events_by_type).length > 0 ?
    //   [{ title: `Top Event: ${Object.keys(summaryData.events_by_type)[0]}`, value: Object.values(summaryData.events_by_type)[0] }]
    //   : [])
  ] : [];

  // Prepare data for charts - ensure the fields exist and default to empty arrays
  const commandUsageChartData = summaryData?.command_usage_over_time || [];
  const userActivityChartData = summaryData?.user_activity_over_time || [];

  // Define keys for the charts based on the backend model structure
  const commandUsageKeys = ['value']; // From TimeSeriesDataPoint
  const userActivityKeys = ['messages', 'commands']; // From UserActivityDataPoint
  const xAxisKey = 'date'; // From both TimeSeriesDataPoint and UserActivityDataPoint

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
            {/* Match options to backend supported periods */}
            <option value="day">Last 24 hours</option>
            <option value="week">Last 7 days</option>
            <option value="month">Last 30 days</option>
            <option value="90d">Last 90 days</option>
            {/* <option value="custom">Custom Range</option> */} {/* Disable custom for now */}
          </FilterSelect>
        </FilterGroup>

        {/* Keep custom date inputs but note they aren't used by API yet */}
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
      
      {/* Display error if summary query fails */}
      {summaryError && (
        <ErrorMessage>
          Error loading analytics data: {summaryError.message || 'Please try again later.'}
        </ErrorMessage>
      )}

      {/* Summary stats */}
      <AnalyticsSummary
        stats={transformedSummaryStats} // Use the transformed data
        isLoading={summaryLoading}
      />

      {/* Charts - Render only if not loading and data exists */}
      {!summaryLoading && summaryData && (
        <>
          <AnalyticsChart
            title="Command Usage Over Time"
            data={commandUsageChartData}
            dataKeys={commandUsageKeys}
            xAxisDataKey={xAxisKey}
            // isLoading={summaryLoading} // Removed prop
          />

          <AnalyticsChart
            title="User Activity Over Time"
            data={userActivityChartData}
            dataKeys={userActivityKeys}
            xAxisDataKey={xAxisKey}
            // isLoading={summaryLoading} // Removed prop
          />
        </>
      )}
    </PageContainer>
  );
};

export default Analytics;
