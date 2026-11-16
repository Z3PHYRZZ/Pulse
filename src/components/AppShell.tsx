import {useEffect, useState, type ReactNode} from 'react';
import {AppShell} from '@astryxdesign/core/AppShell';
import {SideNav, SideNavHeading, SideNavItem, SideNavSection} from '@astryxdesign/core/SideNav';
import {Button} from '@astryxdesign/core/Button';
import {Heading, Text} from '@astryxdesign/core/Text';
import {HStack} from '@astryxdesign/core/Layout';
import {
  ArrowDownTrayIcon, ArrowTopRightOnSquareIcon, ChartBarSquareIcon, CircleStackIcon,
  CpuChipIcon, Cog6ToothIcon, HomeIcon, MoonIcon, QueueListIcon, SunIcon,
  VideoCameraIcon, WifiIcon,
} from '@heroicons/react/24/outline';
import type {History, MetricSample, Page, Snapshot, ThemeMode} from '../types';
import {fmtRate, pct} from '../lib/format';

export const pages: {id: Page; label: string; icon: typeof HomeIcon}[] = [
  {id: 'overview', label: 'Overview', icon: HomeIcon},
  {id: 'cpu', label: 'CPU', icon: CpuChipIcon},
  {id: 'memory', label: 'Memory', icon: CircleStackIcon},
  {id: 'gpu', label: 'GPU', icon: VideoCameraIcon},
  {id: 'storage', label: 'Storage', icon: CircleStackIcon},
  {id: 'network', label: 'Network', icon: WifiIcon},
  {id: 'processes', label: 'Processes', icon: QueueListIcon},
  {id: 'settings', label: 'Settings', icon: Cog6ToothIcon},
];

interface Props {
  page: Page;
  onNavigate: (page: Page) => void;
  snapshot: Snapshot;
  latest: MetricSample | null;
  history: History;
  status: string;
  onTheme: (mode: ThemeMode) => Promise<void>;
  onHide: () => Promise<void>;
  onExit: () => Promise<void>;
  children: ReactNode;
}

export function PulseShell({page, onNavigate, snapshot, latest, status, onTheme, onHide, onExit, children}: Props) {
  const [clock, setClock] = useState(() => new Date());
  useEffect(() => {
    const interval = window.setInterval(() => setClock(new Date()), 1000);
    return () => window.clearInterval(interval);
  }, []);
  const active = pages.find(item => item.id === page)!;
  const subtitle = page === 'overview' ? 'Your system at a glance'
    : page === 'processes' ? 'Running applications and background services'
    : page === 'settings' ? 'Monitoring and application preferences'
    : `Live ${active.label.toLowerCase()} metrics`;
  const theme = snapshot.theme;
  const navigation = <SideNav
    className="pulse-nav"
    header={<SideNavHeading heading="Pulse" subheading={`Windows monitor · v${snapshot.version || '2.0.3'}`} icon={<ChartBarSquareIcon width={22} height={22} />} />}
    footer={<div className="nav-footer">
      <div className="mini-metrics"><span>CPU <strong>{latest ? pct(latest.cpu.usage_percent) : '—'}</strong></span><span>RAM <strong>{latest ? pct(latest.memory.usage_percent) : '—'}</strong></span></div>
      <div className="connection"><span className={`status-dot ${latest ? 'is-live' : ''}`} /><Text type="supporting">{status}</Text></div>
      <Button label={theme === 'dark' ? 'Light mode' : 'Dark mode'} icon={theme === 'dark' ? <SunIcon width={17} /> : <MoonIcon width={17} />} size="sm" width="100%" clickAction={() => onTheme(theme === 'dark' ? 'light' : 'dark')} />
      <Button label="Hide to tray" icon={<ArrowDownTrayIcon width={17} />} size="sm" variant="ghost" width="100%" clickAction={onHide} />
      <Button label="Exit Pulse" icon={<ArrowTopRightOnSquareIcon width={17} />} size="sm" variant="ghost" width="100%" clickAction={onExit} />
    </div>}>
    <SideNavSection title="Monitor">
      {pages.map(item => <SideNavItem key={item.id} label={item.label} icon={item.icon} isSelected={page === item.id} onClick={() => onNavigate(item.id)} />)}
    </SideNavSection>
  </SideNav>;

  return <AppShell className="pulse-shell" sideNav={navigation} variant="section" contentPadding={0} mobileNav={{breakpoint: 'sm'}}>
    <div className="workspace">
      <header className="topbar">
        <div><Heading level={2}>{active.label}</Heading><Text type="supporting">{subtitle}</Text></div>
        <HStack gap={3} align="center" className="topbar-meta">
          <div className="throughput">{latest ? <><span>↓ {fmtRate(latest.network.download_bytes_per_sec)}</span><span>↑ {fmtRate(latest.network.upload_bytes_per_sec)}</span></> : 'Waiting for first sample'}</div>
          <div className="clock"><strong>{clock.toLocaleTimeString([], {hour: '2-digit', minute: '2-digit'})}</strong><span>{snapshot.system.hostname}</span></div>
        </HStack>
      </header>
      <main className="view" id="main-content">{children}</main>
    </div>
  </AppShell>;
}
