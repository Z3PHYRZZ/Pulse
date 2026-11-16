import {Button} from '@astryxdesign/core/Button';
import {Card} from '@astryxdesign/core/Card';
import {Text} from '@astryxdesign/core/Text';
import {Badge} from '@astryxdesign/core/Badge';
import {ArrowRightIcon} from '@heroicons/react/24/outline';
import type {History, MetricSample, Page, Snapshot} from '../types';
import {fmtBytes, fmtRate, pct} from '../lib/format';
import {MetricCard} from '../components/MetricCard';
import {Details} from '../components/Details';

export function PageHeading({title, description, action}: {title: string; description: string; action?: React.ReactNode}) {
  return <div className="page-heading"><div><h1>{title}</h1><Text type="supporting">{description}</Text></div>{action}</div>;
}

export function Loading({label}: {label: string}) {
  return <Card className="loading-card" padding={8}><Text type="large">Waiting for {label}…</Text><Text type="supporting">Collectors are warming up. This usually takes one refresh.</Text></Card>;
}

export function Overview({sample, snapshot, history, onNavigate}: {sample: MetricSample | null; snapshot: Snapshot; history: History; onNavigate: (page: Page) => void}) {
  if (!sample) return <Loading label="the first system sample" />;
  const disk = sample.disk.read_bytes_per_sec + sample.disk.write_bytes_per_sec;
  const network = sample.network.upload_bytes_per_sec + sample.network.download_bytes_per_sec;
  return <>
    <PageHeading title="System overview" description={`${snapshot.system.os_version} · ${snapshot.system.logical_cores} logical cores`} action={<Button label="Open settings" icon={<ArrowRightIcon width={16} />} size="sm" onClick={() => onNavigate('settings')} />} />
    {sample.warnings.length > 0 && <Card variant="orange" className="warning-banner"><Badge variant="warning" label="Collector notes" /><Text>{sample.warnings.join(' · ')}</Text></Card>}
    <div className="metric-grid">
      <MetricCard title="CPU" value={pct(sample.cpu.usage_percent)} detail={`${sample.cpu.logical_cores} logical cores · ${sample.cpu.name}`} history={history.cpu} accent="blue" />
      <MetricCard title="Memory" value={pct(sample.memory.usage_percent)} detail={`${fmtBytes(sample.memory.used_bytes)} of ${fmtBytes(sample.memory.total_bytes)}`} history={history.memory} accent="teal" />
      <MetricCard title="GPU" value={pct(sample.gpu.usage_percent)} detail={`${sample.gpu.name} · ${sample.gpu.source}`} history={history.gpu} accent="purple" />
      <MetricCard title="Disk activity" value={fmtRate(disk)} detail={`Read ${fmtRate(sample.disk.read_bytes_per_sec)} · Write ${fmtRate(sample.disk.write_bytes_per_sec)}`} history={history.disk} accent="orange" />
      <MetricCard title="Network activity" value={fmtRate(network)} detail={`Download ${fmtRate(sample.network.download_bytes_per_sec)} · Upload ${fmtRate(sample.network.upload_bytes_per_sec)}`} history={history.network} accent="green" />
    </div>
    <div className="overview-bottom">
      <Details title="System" rows={[
        {label: 'Computer', value: snapshot.system.hostname},
        {label: 'Windows build', value: snapshot.system.build_number},
        {label: 'Signed-in user', value: snapshot.system.username},
        {label: 'Uptime', value: `${Math.floor(sample.uptime_seconds / 3600)} hours`},
      ]} />
      <Details title="Activity" rows={[
        {label: 'Running processes', value: String(sample.processes.length)},
        {label: 'Refresh interval', value: `${snapshot.config.refresh_ms} ms`},
        {label: 'Graph window', value: `${snapshot.config.history_seconds} seconds`},
      ]} />
    </div>
  </>;
}
