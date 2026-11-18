import type {GpuMetrics, History, MetricSample, Page} from '../types';
import {fmtBytes, fmtRate, highTemp, pct, temperatureText} from '../lib/format';
import {MetricCard} from '../components/MetricCard';
import {Details} from '../components/Details';
import {Loading, PageHeading} from './Overview';

export function Hardware({page, sample, history}: {page: Exclude<Page, 'overview' | 'processes' | 'settings'>; sample: MetricSample | null; history: History}) {
  if (!sample) return <Loading label={page} />;
  if (page === 'cpu') return <Cpu sample={sample} history={history} />;
  if (page === 'memory') return <Memory sample={sample} history={history} />;
  if (page === 'gpu') return <Gpu sample={sample} history={history} />;
  if (page === 'storage') return <Storage sample={sample} history={history} />;
  return <Network sample={sample} history={history} />;
}

function Cpu({sample, history}: {sample: MetricSample; history: History}) {
  const cpu = sample.cpu;
  return <><PageHeading title="Processor" description="Utilization and logical-core activity" />
    <div className="detail-grid"><MetricCard title="CPU utilization" value={pct(cpu.usage_percent)} detail={cpu.name} history={history.cpu} accent="blue" warning={highTemp(cpu.temperature_c) ? 'Temperature high' : undefined} />
      <Details title="Processor details" rows={[
        {label: 'Logical cores', value: String(cpu.logical_cores)},
        {label: 'Estimated frequency', value: cpu.frequency_mhz ? `${cpu.frequency_mhz.toFixed(0)} MHz` : 'Unavailable'},
        {label: 'Temperature', value: temperatureText(cpu.temperature_c), warning: highTemp(cpu.temperature_c)},
      ]} /></div>
    <Details title="Logical cores" rows={cpu.per_core_percent.map((value, index) => ({label: `Core ${String(index + 1).padStart(2, '0')}`, value: pct(value)}))} />
    <p className="page-note">Temperature requires a supported driver sensor path. Standard Windows performance APIs do not guarantee one.</p></>;
}

function Memory({sample, history}: {sample: MetricSample; history: History}) {
  const memory = sample.memory;
  return <><PageHeading title="Memory" description="Physical memory allocation" />
    <div className="detail-grid"><MetricCard title="Physical memory" value={pct(memory.usage_percent)} detail={`${fmtBytes(memory.total_bytes)} installed`} history={history.memory} accent="teal" />
      <Details title="Memory numbers" rows={[
        {label: 'In use', value: fmtBytes(memory.used_bytes)},
        {label: 'Available', value: fmtBytes(memory.available_bytes)},
        {label: 'Total', value: fmtBytes(memory.total_bytes)},
      ]} /></div></>;
}

function gpuVram(gpu: GpuMetrics): string {
  if (gpu.vram_used_bytes != null && gpu.vram_total_bytes != null) {
    return `${fmtBytes(gpu.vram_used_bytes)} of ${fmtBytes(gpu.vram_total_bytes)}`;
  }
  if (gpu.vram_total_bytes != null) return fmtBytes(gpu.vram_total_bytes);
  return 'Unavailable';
}

function Gpu({sample, history}: {sample: MetricSample; history: History}) {
  const adapters = sample.gpus?.length ? sample.gpus : [sample.gpu];
  const primary = sample.gpu;
  return <><PageHeading title="Graphics" description={`${adapters.length} adapter${adapters.length === 1 ? '' : 's'} detected · utilization, memory, and temperature`} />
    {adapters.length > 1 && <MetricCard title="Primary GPU" value={pct(primary.usage_percent)} detail={`${primary.name} · ${primary.vendor}`} history={history.gpu} accent="purple" warning={highTemp(primary.temperature_c) ? 'Temperature high' : undefined} />}
    {adapters.map((gpu, index) => <div className="detail-grid" key={gpu.id || index}>
      <MetricCard
        title={adapters.length > 1 ? `Adapter ${index + 1} — ${gpu.vendor}` : `GPU utilization — ${gpu.vendor}`}
        value={pct(gpu.usage_percent)}
        detail={`${gpu.name}${gpu.is_integrated ? ' · integrated' : ''}`}
        history={adapters.length === 1 ? history.gpu : []}
        accent="purple"
        warning={highTemp(gpu.temperature_c) ? 'Temperature high' : undefined}
      />
      <Details title={`${gpu.name} details`} rows={[
        {label: 'Adapter', value: gpu.name},
        {label: 'Vendor', value: gpu.vendor},
        {label: 'Architecture', value: gpu.architecture || 'Unavailable'},
        {label: 'Type', value: gpu.is_integrated ? 'Integrated' : 'Discrete'},
        {label: 'Dedicated VRAM', value: gpuVram(gpu)},
        {label: 'Shared system memory', value: gpu.shared_system_bytes != null ? fmtBytes(gpu.shared_system_bytes) : 'Unavailable'},
        {label: 'Temperature', value: temperatureText(gpu.temperature_c), warning: highTemp(gpu.temperature_c)},
        {label: 'Telemetry source', value: gpu.source},
      ]} />
    </div>)}
    <p className="page-note">DXGI enumerates every adapter, including integrated and ARM64 Adreno GPUs. NVML supplies NVIDIA utilization, VRAM, and temperature; the PDH GPU Engine counter supplies utilization for Intel, AMD, and integrated adapters where present. Values remain unavailable when no API exposes them.</p></>;
}

function Storage({sample, history}: {sample: MetricSample; history: History}) {
  const disk = sample.disk;
  return <><PageHeading title="Storage" description="Combined disk throughput and volume capacity" />
    <div className="detail-grid"><MetricCard title="Disk activity" value={fmtRate(disk.read_bytes_per_sec + disk.write_bytes_per_sec)} detail={`Read ${fmtRate(disk.read_bytes_per_sec)} · Write ${fmtRate(disk.write_bytes_per_sec)}`} history={history.disk} accent="orange" />
      <Details title="Volumes" rows={disk.volumes.map(volume => ({label: volume.name, value: `${fmtBytes(volume.used_bytes)} / ${fmtBytes(volume.total_bytes)} (${pct(volume.usage_percent)})`}))} /></div></>;
}

function Network({sample, history}: {sample: MetricSample; history: History}) {
  const network = sample.network;
  return <><PageHeading title="Network" description="Upload and download throughput by adapter" />
    <div className="detail-grid"><MetricCard title="Network activity" value={fmtRate(network.upload_bytes_per_sec + network.download_bytes_per_sec)} detail={`Download ${fmtRate(network.download_bytes_per_sec)} · Upload ${fmtRate(network.upload_bytes_per_sec)}`} history={history.network} accent="green" />
      <Details title="Adapters" rows={network.adapters.map(adapter => ({label: adapter.name, value: `↓ ${fmtRate(adapter.download_bytes_per_sec)} · ↑ ${fmtRate(adapter.upload_bytes_per_sec)}`}))} /></div></>;
}
