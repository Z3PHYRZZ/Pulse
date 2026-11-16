import '@testing-library/jest-dom/vitest';
import {render, screen} from '@testing-library/react';
import {describe, expect, it} from 'vitest';
import {Theme} from '@astryxdesign/core/theme';
import {neutralTheme} from '@astryxdesign/theme-neutral/built';
import {MetricCard} from './MetricCard';

describe('MetricCard', () => {
  it('renders a readable value, detail, warning, and history description', () => {
    render(<Theme theme={neutralTheme} mode="dark"><MetricCard title="CPU" value="12.3%" detail="8 logical cores" history={[0, 100]} accent="blue" warning="Temperature high" /></Theme>);
    expect(screen.getByText('CPU')).toBeInTheDocument();
    expect(screen.getByText('12.3%')).toBeInTheDocument();
    expect(screen.getByText('8 logical cores')).toBeInTheDocument();
    expect(screen.getByRole('img', {name: 'CPU history: 0% to 100%'})).toBeInTheDocument();
    expect(screen.getByText('Temperature high')).toBeInTheDocument();
  });
});
