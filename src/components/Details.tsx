import type {ReactNode} from 'react';
import {Card} from '@astryxdesign/core/Card';
import {Heading, Text} from '@astryxdesign/core/Text';

export interface DetailRow {
  label: string;
  value: ReactNode;
  warning?: boolean;
}

export function Details({title, rows}: {title: string; rows: DetailRow[]}) {
  return <Card padding={4} className="details-card">
    <Heading level={3}>{title}</Heading>
    <dl className="detail-list">
      {rows.length ? rows.map(row => <div className="detail-row" key={row.label}>
        <dt><Text type="supporting">{row.label}</Text></dt>
        <dd className={row.warning ? 'detail-warning' : undefined}>{row.value}</dd>
      </div>) : <Text type="supporting">No data available.</Text>}
    </dl>
  </Card>;
}
