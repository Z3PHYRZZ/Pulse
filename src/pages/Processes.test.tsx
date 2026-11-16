import '@testing-library/jest-dom/vitest';
import {render, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it} from 'vitest';
import {Theme} from '@astryxdesign/core/theme';
import {neutralTheme} from '@astryxdesign/theme-neutral/built';
import {Processes} from './Processes';
import type {MetricSample, ProcessInfo} from '../types';

const processes: ProcessInfo[] = [
  {pid: 2, name: 'beta.exe', cpu_percent: 10, memory_bytes: 1024},
  {pid: 1, name: 'alpha.exe', cpu_percent: 20, memory_bytes: 2048},
];
const sample = {processes} as MetricSample;

describe('Processes', () => {
  it('keeps focus while filtering and supports sorting', async () => {
    const user = userEvent.setup();
    render(<Theme theme={neutralTheme}><Processes sample={sample} maxRows={100} initialSort="cpu" /></Theme>);
    const input = screen.getByRole('searchbox', {name: 'Filter processes'});
    await user.type(input, 'alpha');
    expect(input).toHaveFocus();
    expect(screen.getByText('alpha.exe')).toBeInTheDocument();
    expect(screen.queryByText('beta.exe')).not.toBeInTheDocument();
    await user.clear(input);
    await user.click(screen.getByRole('button', {name: 'Memory'}));
    expect(screen.getAllByRole('row')[1]).toHaveTextContent('alpha.exe');
  });
});
