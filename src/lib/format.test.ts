import {describe, expect, it} from 'vitest';
import {fmtBytes, fmtRate, highTemp, pct, points, temperatureText} from './format';

describe('format', () => {
  it('formats byte values with binary units', () => {
    expect(fmtBytes(0)).toBe('0 B');
    expect(fmtBytes(1536)).toBe('1.5 KB');
    expect(fmtBytes(1024 ** 3)).toBe('1.0 GB');
    expect(fmtRate(2048)).toBe('2.0 KB/s');
  });

  it('keeps unavailable percentages and temperatures honest', () => {
    expect(pct(null)).toBe('n/a');
    expect(pct(12.34)).toBe('12.3%');
    expect(temperatureText(null)).toBe('Unavailable');
    expect(temperatureText(79)).toBe('79 °C');
    expect(highTemp(80)).toBe(true);
    expect(highTemp(null)).toBe(false);
  });

  it('creates bounded smoothed sparkline coordinates', () => {
    expect(points([1])).toEqual([]);
    expect(points([0, 100])).toEqual([{x: 0, y: 36.32}, {x: 300, y: 11.68}]);
  });
});
