import {useEffect, useRef, useState} from 'react';
import {emptyHistory, fallbackSnapshot, type History, type MetricSample, type Snapshot} from '../types';
import {connectMetrics, tauriBridge, type ConnectionState} from '../lib/pulse';

export function usePulse() {
  const [snapshot, setSnapshot] = useState<Snapshot>(fallbackSnapshot);
  const [latest, setLatest] = useState<MetricSample | null>(null);
  const [history, setHistory] = useState<History>(emptyHistory);
  const [status, setStatus] = useState('Connecting to collectors…');
  const state = useRef<ConnectionState>({snapshot, history, lastTimestamp: null});
  state.current.snapshot = snapshot;
  state.current.history = history;

  useEffect(() => {
    let cleanup: (() => void) | undefined;
    let disposed = false;
    void connectMetrics(tauriBridge, state.current, next => {
      if (disposed) return;
      if (next.snapshot) {
        state.current.snapshot = next.snapshot;
        setSnapshot(next.snapshot);
        setStatus(next.snapshot.minimized ? 'Started hidden. Close exits Pulse.' : 'Live metrics connected');
      }
      if (next.history) {
        state.current.history = next.history;
        state.current.lastTimestamp = next.lastTimestamp ?? null;
        setHistory(next.history);
      }
      if (next.latest) setLatest(next.latest);
    }).then(stop => {
      if (disposed) stop();
      else cleanup = stop;
    }).catch(error => setStatus(`Metric connection unavailable: ${error instanceof Error ? error.message : String(error)}`));
    return () => { disposed = true; cleanup?.(); };
  }, []);

  return {snapshot, latest, history, status, setSnapshot, setStatus};
}
