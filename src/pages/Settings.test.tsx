import '@testing-library/jest-dom/vitest';
import {render, screen} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {describe, expect, it, vi} from 'vitest';
import {Theme} from '@astryxdesign/core/theme';
import {neutralTheme} from '@astryxdesign/theme-neutral/built';
import {Settings} from './Settings';
import {fallbackSnapshot} from '../types';

describe('Settings', () => {
  it('preserves the full config on save and reports rejected commands', async () => {
    const user = userEvent.setup();
    const onSave = vi.fn().mockRejectedValueOnce(new Error('disk full')).mockResolvedValueOnce(undefined);
    const snapshot = {...fallbackSnapshot, config: {...fallbackSnapshot.config, process_sort: 'memory', start_minimized: true}};
    render(<Theme theme={neutralTheme}><Settings snapshot={snapshot} onSave={onSave} onOpenFolder={vi.fn()} /></Theme>);
    await user.click(screen.getByRole('button', {name: 'Save settings'}));
    expect(await screen.findByText('Settings not saved: disk full')).toBeInTheDocument();
    expect(onSave).toHaveBeenCalledWith(expect.objectContaining({process_sort: 'memory', start_minimized: true}), false);
    await user.click(screen.getByRole('button', {name: 'Save settings'}));
    expect(await screen.findByText('Settings saved.')).toBeInTheDocument();
  });
});
