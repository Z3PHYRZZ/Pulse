import {describe, expect, it} from 'vitest';
import type {History, MetricSample, ProcessInfo} from '../types';
import {emptyHistory} from '../types';
import {addSample, filterProcesses, validateSettings} from './metrics';

const sample = (timestamp_ms: number): MetricSample => ({
  timestamp_ms,
  cpu: {usage_percent: 10, per_core_percent: [10], frequency_mhz: 2400, logical_cores: 1, name: 'CPU', temperature_c: null},
  memory: {used_bytes: 5, available_bytes: 5, total_bytes: 10, usage_percent: 50},
  gpu: {name: 'GPU', usage_percent: null, vram_used_bytes: null, vram_total_bytes: null, temperature_c: null, source: 'DXGI'},
  disk: {read_bytes_per_sec: 1048576, write_bytes_per_sec: 1048576, volumes: []},
  network: {upload_bytes_per_sec: 1048576, download_bytes_per_sec: 3145728, adapters: []},
  processes: [], uptime_seconds: 1, warnings: [],
});

describe('metric state', () => {
  it('adds a sample once and trims every history channel', () => {
    const first = addSample(emptyHistory(), sample(1), 2);
    const duplicate = addSample(first.history, sample(1), 2, first.lastTimestamp);
    const second = addSample(first.history, sample(2), 2, first.lastTimestamp);
    const third = addSample(second.history, sample(3), 2, second.lastTimestamp);
    expect(first.history.cpu).toEqual([10]);
    expect(duplicate).toEqual({history: first.history, lastTimestamp: 1});
    const stale = addSample(first.history, sample(0), 2, first.lastTimestamp);
    expect(stale).toEqual({history: first.history, lastTimestamp: 1});
    expect(third.history.cpu).toEqual([10, 10]);
    expect(third.history.disk).toEqual([2, 2]);
    expect(third.history.network).toEqual([4, 4]);
    expect(third.lastTimestamp).toBe(3);
  });

  it('filters and sorts processes before applying the configured row cap', () => {
    const processes: ProcessInfo[] = [
      {pid: 3, name: 'Beta', cpu_percent: 5, memory_bytes: 50},
      {pid: 1, name: 'alpha', cpu_percent: 20, memory_bytes: 10},
      {pid: 2, name: 'alphabet', cpu_percent: 1, memory_bytes: 100},
    ];
    expect(filterProcesses(processes, 'ALPHA', 'memory', 1).map(p => p.pid)).toEqual([2]);
    expect(filterProcesses(processes, '', 'name', 10).map(p => p.pid)).toEqual([1, 2, 3]);
    expect(filterProcesses(processes, '', 'pid', 2).map(p => p.pid)).toEqual([1, 2]);
    expect(filterProcesses(processes, 'missing', 'cpu', 10)).toEqual([]);
  });

  it('validates settings instead of relying on input attributes', () => {
    expect(validateSettings({refresh_ms: 1000, history_seconds: 180, max_process_rows: 300})).toBeNull();
    expect(validateSettings({refresh_ms: 249, history_seconds: 180, max_process_rows: 300})).toMatch('250');
    expect(validateSettings({refresh_ms: 1000, history_seconds: 601, max_process_rows: 300})).toMatch('600');
    expect(validateSettings({refresh_ms: 1000, history_seconds: 180, max_process_rows: 0})).toMatch('10');
  });
});
