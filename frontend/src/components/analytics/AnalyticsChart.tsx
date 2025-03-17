import React from 'react';
import styled from 'styled-components';
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Legend, CartesianGrid } from 'recharts';
import { TooltipProps } from 'recharts/types/component/Tooltip';

const ChartContainer = styled.div`
  background-color: #2f3136;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  height: 400px;
`;

const ChartTitle = styled.h3`
  color: #ffffff;
  margin-top: 0;
  margin-bottom: 16px;
  font-size: 1.2rem;
`;

const NoDataMessage = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  height: 300px;
  color: #72767d;
  font-size: 1rem;
`;

// Custom tooltip styling
const CustomTooltip = styled.div`
  background-color: #18191c;
  border: 1px solid #202225;
  border-radius: 4px;
  padding: 10px;
  color: #dcddde;
  font-size: 14px;
  
  .label {
    margin-bottom: 5px;
    color: #ffffff;
    font-weight: 600;
  }
  
  .data-item {
    display: flex;
    justify-content: space-between;
    margin-bottom: 3px;
  }
  
  .value {
    margin-left: 20px;
    font-weight: 500;
  }
`;

interface AnalyticsChartProps {
  title: string;
  data: any[];
  dataKeys: string[];
  colors?: string[];
  xAxisDataKey?: string;
  stacked?: boolean;
}

import { Payload } from 'recharts/types/component/DefaultTooltipContent';

// Custom recharts tooltip component
const CustomTooltipComponent: React.FC<TooltipProps<number, string>> = ({ active, payload, label }) => {
  if (!active || !payload || !payload.length) {
    return null;
  }

  return (
    <CustomTooltip>
      <div className="label">{label}</div>
      {payload.map((entry, index: number) => (
        <div key={index} className="data-item">
          <span style={{ color: entry.color }}>{entry.name}:</span>
          <span className="value">{entry.value}</span>
        </div>
      ))}
    </CustomTooltip>
  );
};

const AnalyticsChart: React.FC<AnalyticsChartProps> = ({
  title,
  data,
  dataKeys,
  colors = ['#5865F2', '#57F287', '#FEE75C', '#EB459E', '#ED4245'],
  xAxisDataKey = 'name',
  stacked = false,
}) => {
  if (!data || data.length === 0) {
    return (
      <ChartContainer>
        <ChartTitle>{title}</ChartTitle>
        <NoDataMessage>No data available</NoDataMessage>
      </ChartContainer>
    );
  }

  return (
    <ChartContainer>
      <ChartTitle>{title}</ChartTitle>
      <ResponsiveContainer width="100%" height={300}>
        <BarChart data={data} margin={{ top: 10, right: 30, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#40444b" vertical={false} />
          <XAxis 
            dataKey={xAxisDataKey} 
            tick={{ fill: '#b9bbbe' }}
            axisLine={{ stroke: '#40444b' }}
          />
          <YAxis 
            tick={{ fill: '#b9bbbe' }}
            axisLine={{ stroke: '#40444b' }}
          />
          <Tooltip content={<CustomTooltipComponent />} />
          <Legend wrapperStyle={{ color: '#b9bbbe' }} />
          {dataKeys.map((key, index) => (
            <Bar
              key={key}
              dataKey={key}
              stackId={stacked ? 'stack' : undefined}
              fill={colors[index % colors.length]}
              radius={[4, 4, 0, 0]}
            />
          ))}
        </BarChart>
      </ResponsiveContainer>
    </ChartContainer>
  );
};

export default AnalyticsChart;
