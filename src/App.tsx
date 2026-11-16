import {useState} from 'react';
import {Theme} from '@astryxdesign/core/theme';
import {LayerProvider} from '@astryxdesign/core/Layer';
import {neutralTheme} from '@astryxdesign/theme-neutral/built';
import type {Config, Page, ProcessSort, Snapshot, ThemeMode} from './types';
import {usePulse} from './hooks/usePulse';
import {exitApp, hideWindow, openConfigFolder, saveConfig, setStartup, setTheme, snapshot as loadSnapshot} from './lib/pulse';
import {PulseShell} from './components/AppShell';
import {Overview} from './pages/Overview';
import {Hardware} from './pages/Hardware';
import {Processes} from './pages/Processes';
import {Settings} from './pages/Settings';

export default function App() {
  const [page, setPage] = useState<Page>('overview');
  const {snapshot, latest, history, status, setSnapshot, setStatus} = usePulse();
  const changeTheme = async (mode: ThemeMode) => {
    try {
      await setTheme(mode);
      setSnapshot(await loadSnapshot());
      setStatus('Appearance saved.');
    } catch (error) { setStatus(`Theme not saved: ${errorText(error)}`); }
  };
  const save = async (config: Config, startup: boolean) => {
    await saveConfig(config);
    await setStartup(startup);
    setSnapshot(await loadSnapshot());
    setStatus('Settings saved to config.toml.');
  };
  const run = async (action: () => Promise<void>, name: string) => {
    try { await action(); } catch (error) { setStatus(`${name} failed: ${errorText(error)}`); }
  };
  const sort: ProcessSort = ['cpu', 'memory', 'name', 'pid'].includes(snapshot.config.process_sort) ? snapshot.config.process_sort as ProcessSort : 'cpu';
  return <Theme theme={neutralTheme} mode={snapshot.theme}>
    <LayerProvider>
      <PulseShell page={page} onNavigate={setPage} snapshot={snapshot} latest={latest} history={history} status={status} onTheme={changeTheme} onHide={() => run(hideWindow, 'Hide to tray')} onExit={() => run(exitApp, 'Exit')}>
        {page === 'overview' && <Overview sample={latest} snapshot={snapshot} history={history} onNavigate={setPage} />}
        {['cpu', 'memory', 'gpu', 'storage', 'network'].includes(page) && <Hardware page={page as 'cpu' | 'memory' | 'gpu' | 'storage' | 'network'} sample={latest} history={history} />}
        {page === 'processes' && <Processes key={sort} sample={latest} maxRows={snapshot.config.max_process_rows} initialSort={sort} />}
        {page === 'settings' && <Settings snapshot={snapshot} onSave={save} onOpenFolder={() => run(openConfigFolder, 'Open config folder')} />}
      </PulseShell>
    </LayerProvider>
  </Theme>;
}

const errorText = (error: unknown) => error instanceof Error ? error.message : String(error);
