import React from 'react';
import styled from 'styled-components';

const SummaryContainer = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
`;

const StatsCard = styled.div`
  background-color: #2f3136;
  border-radius: 8px;
  padding: 20px;
  display: flex;
  flex-direction: column;
`;

const StatTitle = styled.div`
  color: #b9bbbe;
  font-size: 14px;
  margin-bottom: 8px;
`;

const StatValue = styled.div`
  color: #ffffff;
  font-size: 28px;
  font-weight: 600;
  margin-bottom: 4px;
`;

const StatChange = styled.div<{ positive?: boolean }>`
  color: ${props => (props.positive ? '#57F287' : '#ED4245')};
  font-size: 12px;
  display: flex;
  align-items: center;
`;

const LoadingState = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  height: 120px;
  background-color: #2f3136;
  border-radius: 8px;
  color: #72767d;
`;

// Types
interface Stat {
  title: string;
  value: number | string;
  change?: number;
  loading?: boolean;
}

interface AnalyticsSummaryProps {
  stats: Stat[];
  isLoading?: boolean;
}

const AnalyticsSummary: React.FC<AnalyticsSummaryProps> = ({ stats, isLoading = false }) => {
  if (isLoading) {
    return (
      <SummaryContainer>
        {[1, 2, 3, 4].map(i => (
          <LoadingState key={i}>Loading...</LoadingState>
        ))}
      </SummaryContainer>
    );
  }

  return (
    <SummaryContainer>
      {stats.map((stat, index) => (
        <StatsCard key={index}>
          <StatTitle>{stat.title}</StatTitle>
          <StatValue>{stat.value}</StatValue>
          {stat.change !== undefined && (
            <StatChange positive={stat.change >= 0}>
              {stat.change >= 0 ? '↑' : '↓'} {Math.abs(stat.change)}% from last period
            </StatChange>
          )}
        </StatsCard>
      ))}
    </SummaryContainer>
  );
};

export default AnalyticsSummary;
