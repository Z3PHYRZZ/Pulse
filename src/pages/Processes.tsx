import {useMemo, useState} from 'react';
import {TextInput} from '@astryxdesign/core/TextInput';
import {Button} from '@astryxdesign/core/Button';
import {Card} from '@astryxdesign/core/Card';
import {Table, pixel, proportional, type TableColumn} from '@astryxdesign/core/Table';
import {Text} from '@astryxdesign/core/Text';
import {MagnifyingGlassIcon} from '@heroicons/react/24/outline';
import type {MetricSample, ProcessInfo, ProcessSort} from '../types';
import {filterProcesses} from '../lib/metrics';
import {fmtBytes, pct} from '../lib/format';
import {Loading, PageHeading} from './Overview';

const columns: TableColumn<ProcessInfo & Record<string, unknown>>[] = [
  {key: 'pid', header: 'PID', width: pixel(80), resizable: false},
  {key: 'name', header: 'Process', width: proportional(2)},
  {key: 'cpu_percent', header: 'CPU', align: 'end', width: proportional(1), renderCell: process => <Text hasTabularNumbers>{pct(process.cpu_percent)}</Text>},
  {key: 'memory_bytes', header: 'Memory', align: 'end', width: proportional(1), renderCell: process => <Text hasTabularNumbers>{fmtBytes(process.memory_bytes)}</Text>},
];

export function Processes({sample, maxRows, initialSort}: {sample: MetricSample | null; maxRows: number; initialSort: ProcessSort}) {
  const [filter, setFilter] = useState('');
  const [sort, setSort] = useState<ProcessSort>(initialSort);
  const rows = useMemo(() => filterProcesses(sample?.processes ?? [], filter, sort, maxRows), [sample, filter, sort, maxRows]);
  if (!sample) return <Loading label="processes" />;
  return <><PageHeading title="Processes" description={`${rows.length} shown · sorted by ${sort}`} />
    <div className="process-tools">
      <TextInput type="search" label="Filter processes" value={filter} onChange={setFilter} placeholder="Search process name" startIcon={<MagnifyingGlassIcon width={18} />} hasClear width="100%" />
      <div className="sort-buttons" role="group" aria-label="Sort processes">
        {(['cpu', 'memory', 'name', 'pid'] as const).map(key => <Button key={key} label={key === 'pid' ? 'PID' : key[0].toUpperCase() + key.slice(1)} size="sm" variant={sort === key ? 'primary' : 'secondary'} onClick={() => setSort(key)} />)}
      </div>
    </div>
    <Card padding={0} className="process-table">
      {rows.length ? <Table data={rows as (ProcessInfo & Record<string, unknown>)[]} columns={columns} idKey="pid" density="compact" textOverflow="truncate" hasHover aria-label="Running processes" /> : <div className="empty-state"><Text type="large">No matching processes</Text><Text type="supporting">Clear the filter or try a different process name.</Text></div>}
    </Card></>;
}
