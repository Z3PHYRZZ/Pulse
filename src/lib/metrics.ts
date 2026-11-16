import type {History, MetricSample, ProcessInfo, ProcessSort} from '../types';

export interface MetricState {
  history: History;
  lastTimestamp: number | null;
}

export function addSample(history: History, sample: MetricSample, maxPoints: number, lastTimestamp: number | null = null): MetricState {
  if (lastTimestamp !== null && lastTimestamp >= sample.timestamp_ms) return {history, lastTimestamp};
  const limit = Math.max(1, maxPoints);
  const append = (values: number[], value: number) => [...values, value].slice(-limit);
  const disk = (sample.disk.read_bytes_per_sec + sample.disk.write_bytes_per_sec) / (1024 * 1024);
  const network = (sample.network.upload_bytes_per_sec + sample.network.download_bytes_per_sec) / (1024 * 1024);
  return {
    history: {
      cpu: append(history.cpu, sample.cpu.usage_percent),
      memory: append(history.memory, sample.memory.usage_percent),
      gpu: append(history.gpu, sample.gpu.usage_percent ?? 0),
      disk: append(history.disk, Math.min(100, disk)),
      network: append(history.network, Math.min(100, network)),
    },
    lastTimestamp: sample.timestamp_ms,
  };
}

export function filterProcesses(processes: ProcessInfo[], filter: string, sort: ProcessSort, maxRows: number): ProcessInfo[] {
  return processes
    .filter(process => process.name.toLowerCase().includes(filter.toLowerCase()))
    .sort((a, b) => sort === 'memory' ? b.memory_bytes - a.memory_bytes
      : sort === 'name' ? a.name.localeCompare(b.name)
      : sort === 'pid' ? a.pid - b.pid
      : b.cpu_percent - a.cpu_percent)
    .slice(0, maxRows);
}

export interface SettingsValues {
  refresh_ms: number;
  history_seconds: number;
  max_process_rows: number;
}

export function validateSettings(values: SettingsValues): string | null {
  if (!Number.isInteger(values.refresh_ms) || values.refresh_ms < 250 || values.refresh_ms > 10000) return 'Refresh interval must be between 250 and 10000 ms.';
  if (!Number.isInteger(values.history_seconds) || values.history_seconds < 10 || values.history_seconds > 600) return 'Graph history must be between 10 and 600 seconds.';
  if (!Number.isInteger(values.max_process_rows) || values.max_process_rows < 10 || values.max_process_rows > 1000) return 'Process rows must be between 10 and 1000.';
  return null;
}
