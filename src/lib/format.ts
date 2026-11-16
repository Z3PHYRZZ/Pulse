export function fmtBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = Number(bytes || 0);
  let index = 0;
  while (value >= 1024 && index < units.length - 1) {
    value /= 1024;
    index += 1;
  }
  return `${value.toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
}

export const fmtRate = (bytes: number): string => `${fmtBytes(bytes)}/s`;
export const pct = (value: number | null): string => value != null && Number.isFinite(value) ? `${value.toFixed(1)}%` : 'n/a';
export const highTemp = (temp: number | null): boolean => temp != null && Number.isFinite(temp) && temp >= 80;
export const temperatureText = (temp: number | null): string => temp != null && Number.isFinite(temp) ? `${temp.toFixed(0)} °C` : 'Unavailable';

export function points(values: number[]): {x: number; y: number}[] {
  if (values.length < 2) return [];
  return values.map((value, index) => {
    const previous = values[Math.max(0, index - 1)];
    const next = values[Math.min(values.length - 1, index + 1)];
    const smooth = previous * 0.22 + value * 0.56 + next * 0.22;
    return {
      x: (index / (values.length - 1)) * 300,
      y: 48 - (Math.max(0, Math.min(100, smooth)) / 100) * 44 - 2,
    };
  });
}
