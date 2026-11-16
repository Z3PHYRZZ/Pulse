import {invoke} from '@tauri-apps/api/core';
import {listen} from '@tauri-apps/api/event';
import type {Config, History, MetricSample, Snapshot} from '../types';
import {addSample} from './metrics';

export const snapshot = () => invoke<Snapshot>('snapshot');
export const latestSample = () => invoke<MetricSample | null>('latest_sample');
export const setTheme = (theme: string) => invoke<void>('set_theme', {theme});
export const saveConfig = (config: Config) => invoke<void>('save_config', {config});
export const setStartup = (enabled: boolean) => invoke<void>('set_startup', {enabled});
export const hideWindow = () => invoke<void>('hide_window');
export const exitApp = () => invoke<void>('exit_app');
export const openConfigFolder = () => invoke<void>('open_config_folder');

export interface MetricBridge {
  snapshot(): Promise<Snapshot>;
  latestSample(): Promise<MetricSample | null>;
  listen(handler: (sample: MetricSample) => void): Promise<() => void>;
}

export interface ConnectionState {
  snapshot: Snapshot;
  history: History;
  lastTimestamp: number | null;
  latest?: MetricSample | null;
}

export const tauriBridge: MetricBridge = {
  snapshot,
  latestSample,
  listen: handler => listen<MetricSample>('pulse://metrics', event => handler(event.payload)),
};

export async function connectMetrics(
  bridge: MetricBridge,
  state: ConnectionState,
  update: (next: Partial<ConnectionState>) => void,
): Promise<() => void> {
  let disposed = false;
  let lastTimestamp = state.lastTimestamp;
  const ingest = (sample: MetricSample | null) => {
    if (!sample || disposed) return;
    const next = addSample(state.history, sample, state.snapshot.config.history_seconds, lastTimestamp);
    if (next.lastTimestamp === lastTimestamp) return;
    lastTimestamp = next.lastTimestamp;
    update({...next, latest: sample});
  };
  let unlisten: (() => void) | undefined;
  void bridge.listen(ingest).then(stop => {
    if (disposed) stop();
    else unlisten = stop;
  }).catch(() => { /* polling remains available */ });
  try {
    update({snapshot: await bridge.snapshot()});
  } catch { /* fallback snapshot remains visible */ }
  ingest(await bridge.latestSample().catch(() => null));
  const interval = window.setInterval(async () => ingest(await bridge.latestSample().catch(() => null)), 1000);
  return () => {
    disposed = true;
    window.clearInterval(interval);
    unlisten?.();
  };
}
