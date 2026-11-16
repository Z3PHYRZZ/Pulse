export type Page = 'overview' | 'cpu' | 'memory' | 'gpu' | 'storage' | 'network' | 'processes' | 'settings';
export type ThemeMode = 'dark' | 'light';
export type ProcessSort = 'cpu' | 'memory' | 'name' | 'pid';

export interface Config {
  refresh_ms: number;
  history_seconds: number;
  max_process_rows: number;
  process_sort: string;
  start_minimized: boolean;
  minimize_to_tray: boolean;
  enable_startup: boolean;
  theme: string;
}

export interface SystemInfo {
  hostname: string;
  os_version: string;
  build_number: string;
  username: string;
  logical_cores: number;
  total_memory_bytes: number;
  boot_time_unix: number;
}

export interface Snapshot {
  version: string;
  config: Config;
  theme: ThemeMode;
  system: SystemInfo;
  startup_enabled: boolean;
  minimized: boolean;
}

export interface CpuMetrics {
  usage_percent: number;
  per_core_percent: number[];
  frequency_mhz: number | null;
  logical_cores: number;
  name: string;
  temperature_c: number | null;
}

export interface MemoryMetrics {
  used_bytes: number;
  available_bytes: number;
  total_bytes: number;
  usage_percent: number;
}

export interface GpuMetrics {
  name: string;
  usage_percent: number | null;
  vram_used_bytes: number | null;
  vram_total_bytes: number | null;
  temperature_c: number | null;
  source: string;
}

export interface VolumeUsage {
  name: string;
  used_bytes: number;
  total_bytes: number;
  usage_percent: number;
}

export interface DiskMetrics {
  read_bytes_per_sec: number;
  write_bytes_per_sec: number;
  volumes: VolumeUsage[];
}

export interface AdapterTraffic {
  name: string;
  upload_bytes_per_sec: number;
  download_bytes_per_sec: number;
}

export interface NetworkMetrics {
  upload_bytes_per_sec: number;
  download_bytes_per_sec: number;
  adapters: AdapterTraffic[];
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_percent: number;
  memory_bytes: number;
}

export interface MetricSample {
  timestamp_ms: number;
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  gpu: GpuMetrics;
  disk: DiskMetrics;
  network: NetworkMetrics;
  processes: ProcessInfo[];
  uptime_seconds: number;
  warnings: string[];
}

export type History = Record<'cpu' | 'memory' | 'gpu' | 'disk' | 'network', number[]>;
export type HistoryKey = keyof History;

export const fallbackSnapshot: Snapshot = {
  version: '',
  config: {
    refresh_ms: 1000, history_seconds: 180, max_process_rows: 300,
    process_sort: 'cpu', start_minimized: false, minimize_to_tray: false,
    enable_startup: false, theme: 'dark',
  },
  theme: 'dark',
  system: {hostname: 'pulse', os_version: 'Windows', build_number: 'n/a', username: 'n/a', logical_cores: 0, total_memory_bytes: 0, boot_time_unix: 0},
  startup_enabled: false,
  minimized: false,
};

export const emptyHistory = (): History => ({cpu: [], memory: [], gpu: [], disk: [], network: []});
