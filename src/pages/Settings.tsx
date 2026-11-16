import {useEffect, useState} from 'react';
import {NumberInput} from '@astryxdesign/core/NumberInput';
import {Switch} from '@astryxdesign/core/Switch';
import {Button} from '@astryxdesign/core/Button';
import {Card} from '@astryxdesign/core/Card';
import {Text, Heading} from '@astryxdesign/core/Text';
import {FolderOpenIcon} from '@heroicons/react/24/outline';
import type {Config, Snapshot} from '../types';
import {validateSettings} from '../lib/metrics';
import {Details} from '../components/Details';
import {PageHeading} from './Overview';

interface Props {
  snapshot: Snapshot;
  onSave: (config: Config, startup: boolean) => Promise<void>;
  onOpenFolder: () => Promise<void>;
}

export function Settings({snapshot, onSave, onOpenFolder}: Props) {
  const [refresh, setRefresh] = useState(snapshot.config.refresh_ms);
  const [history, setHistory] = useState(snapshot.config.history_seconds);
  const [rows, setRows] = useState(snapshot.config.max_process_rows);
  const [startup, setStartup] = useState(snapshot.startup_enabled);
  const [message, setMessage] = useState<{type: 'success' | 'error'; text: string} | null>(null);
  useEffect(() => {
    setRefresh(snapshot.config.refresh_ms);
    setHistory(snapshot.config.history_seconds);
    setRows(snapshot.config.max_process_rows);
    setStartup(snapshot.startup_enabled);
  }, [snapshot]);

  const save = async () => {
    const error = validateSettings({refresh_ms: refresh, history_seconds: history, max_process_rows: rows});
    if (error) { setMessage({type: 'error', text: error}); return; }
    const config = {...snapshot.config, refresh_ms: refresh, history_seconds: history, max_process_rows: rows, enable_startup: startup, theme: snapshot.theme};
    setMessage(null);
    try {
      await onSave(config, startup);
      setMessage({type: 'success', text: 'Settings saved.'});
    } catch (error) {
      setMessage({type: 'error', text: `Settings not saved: ${error instanceof Error ? error.message : String(error)}`});
    }
  };

  return <><PageHeading title="Settings" description="Monitoring, startup, and configuration" />
    <div className="settings-grid">
      <Card padding={5}><Heading level={3}>Monitoring</Heading><div className="settings-fields">
        <NumberInput label="Refresh interval" description="How often collectors sample the system." value={refresh} onChange={setRefresh} min={250} max={10000} step={250} units="ms" isIntegerOnly hasNumberSteppers />
        <NumberInput label="Graph history" description="Number of recent samples retained in each graph." value={history} onChange={setHistory} min={10} max={600} step={10} units="samples" isIntegerOnly hasNumberSteppers />
        <NumberInput label="Process rows" description="Maximum rows shown after filtering and sorting." value={rows} onChange={setRows} min={10} max={1000} step={10} units="rows" isIntegerOnly hasNumberSteppers />
      </div></Card>
      <div className="settings-side"><Card padding={5}><Heading level={3}>Startup</Heading><Switch label="Start Pulse when you sign in" description="Adds or removes Pulse from your Windows startup apps." value={startup} onChange={setStartup} /></Card>
        <Details title="Appearance & window" rows={[
          {label: 'Theme', value: snapshot.theme === 'dark' ? 'Dark' : 'Light'},
          {label: 'Close button', value: 'Exits Pulse completely'},
          {label: 'Hide to tray', value: 'Keeps monitoring in the background'},
        ]} /></div>
    </div>
    <div className="settings-actions">
      <Button label="Save settings" variant="primary" clickAction={save} />
      <Button label="Open config folder" icon={<FolderOpenIcon width={17} />} clickAction={onOpenFolder} />
    </div>
    <div aria-live="polite">{message && <Text className={message.type === 'error' ? 'error-text' : 'success-text'}>{message.text}</Text>}</div>
  </>;
}
