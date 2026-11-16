import {Card} from '@astryxdesign/core/Card';
import {Text} from '@astryxdesign/core/Text';
import {Badge} from '@astryxdesign/core/Badge';
import {points} from '../lib/format';

export interface MetricCardProps {
  title: string;
  value: string;
  detail: string;
  history: number[];
  accent: 'blue' | 'teal' | 'purple' | 'orange' | 'green';
  warning?: string;
}

export function MetricCard({title, value, detail, history, accent, warning}: MetricCardProps) {
  const path = points(history);
  return <Card className={`metric-card accent-${accent}`} padding={4}>
    <div className="metric-heading"><Text type="label">{title}</Text>{warning && <Badge variant="warning" label="Attention" />}</div>
    <Text type="display-3" className="metric-value" hasTabularNumbers>{value}</Text>
    <Text type="supporting" className="metric-detail">{detail}</Text>
    <svg className="spark" viewBox="0 0 300 48" preserveAspectRatio="none" role="img" aria-label={`${title} history: ${history.length ? `${history[0]}% to ${history.at(-1)}%` : 'waiting for samples'}`}>
      <line className="spark-base" x1="0" y1="47" x2="300" y2="47" />
      {path.length > 0 && <polyline points={path.map(point => `${point.x},${point.y}`).join(' ')} />}
    </svg>
    {warning && <Text type="supporting" className="warning-text">{warning}</Text>}
  </Card>;
}
