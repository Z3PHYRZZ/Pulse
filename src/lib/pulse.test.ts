import {afterEach, describe, expect, it, vi} from 'vitest';
import {emptyHistory, fallbackSnapshot, type MetricSample} from '../types';
import {connectMetrics} from './pulse';

const sample = (timestamp_ms: number): MetricSample => ({
  timestamp_ms,
  cpu: {usage_percent: timestamp_ms, per_core_percent: [], frequency_mhz: null, logical_cores: 1, name: 'CPU', temperature_c: null},
  memory: {used_bytes: 1, available_bytes: 1, total_bytes: 2, usage_percent: 50},
  gpu: {name: 'GPU', usage_percent: null, vram_used_bytes: null, vram_total_bytes: null, temperature_c: null, source: 'DXGI'},
  disk: {read_bytes_per_sec: 0, write_bytes_per_sec: 0, volumes: []},
  network: {upload_bytes_per_sec: 0, download_bytes_per_sec: 0, adapters: []},
  processes: [], uptime_seconds: 0, warnings: [],
});

describe('metric connection', () => {
  afterEach(() => vi.useRealTimers());

  it('accepts events, deduplicates polling, and cleans up both subscriptions', async () => {
    vi.useFakeTimers();
    let eventHandler: ((sample: MetricSample) => void) | undefined;
    const unlisten = vi.fn();
    const received: number[] = [];
    const bridge = {
      snapshot: vi.fn().mockResolvedValue(fallbackSnapshot),
      latestSample: vi.fn().mockResolvedValueOnce(sample(1)).mockResolvedValue(sample(2)),
      listen: vi.fn().mockImplementation((handler: (sample: MetricSample) => void) => {
        eventHandler = handler;
        return Promise.resolve(unlisten);
      }),
    };
    const state = {snapshot: fallbackSnapshot, history: emptyHistory(), lastTimestamp: null as number | null};
    const connection = await connectMetrics(bridge, state, next => {
      Object.assign(state, next);
      if (next.lastTimestamp != null) received.push(next.lastTimestamp);
    });
    expect(received).toEqual([1]);
    eventHandler?.(sample(2));
    eventHandler?.(sample(2));
    expect(received).toEqual([1, 2]);
    await vi.advanceTimersByTimeAsync(1000);
    expect(received).toEqual([1, 2]);
    eventHandler?.(sample(3));
    await vi.advanceTimersByTimeAsync(1000);
    expect(received).toEqual([1, 2, 3]);
    connection();
    expect(unlisten).toHaveBeenCalledOnce();
    expect(vi.getTimerCount()).toBe(0);
  });
});
