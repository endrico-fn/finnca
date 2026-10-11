import type { DailyDataPoint } from './report.svelte';

export type TrendsMetric = 'netWorth' | 'assets' | 'liabilities' | 'liquidCash';
export type TrendsPeriod = '1W' | '1M' | '3M' | 'YTD' | '1Y' | 'ALL';

export interface ChartPoint {
  x: number;
  y: number;
  val: number;
  point: DailyDataPoint;
}

export interface ChartCoordsResult {
  pathD: string;
  areaD: string;
  points: ChartPoint[];
  minVal: number;
  maxVal: number;
  peakPoint: ChartPoint | null;
  troughPoint: ChartPoint | null;
}

export function getPointValue(pt: DailyDataPoint | null, metric: TrendsMetric): number {
  if (!pt) return 0;
  if (metric === 'assets') return pt.assets;
  if (metric === 'liabilities') return pt.liabilities;
  if (metric === 'liquidCash') return pt.liquidCash;
  return pt.netWorth;
}

export function calculateChartCoords(
  pts: DailyDataPoint[],
  metric: TrendsMetric,
  chartWidth = 900,
  chartHeight = 240,
  padX = 16,
  padY = 24
): ChartCoordsResult {
  if (pts.length === 0) {
    return {
      pathD: '',
      areaD: '',
      points: [],
      minVal: 0,
      maxVal: 0,
      peakPoint: null,
      troughPoint: null,
    };
  }

  const vals = pts.map((p) => getPointValue(p, metric));
  let minVal = Math.min(...vals);
  let maxVal = Math.max(...vals);

  if (minVal === maxVal) {
    minVal -= 100000;
    maxVal += 100000;
  } else {
    const margin = (maxVal - minVal) * 0.08;
    minVal -= margin;
    maxVal += margin;
  }

  const rangeY = maxVal - minVal;
  const n = pts.length;

  let peakIndex = 0;
  let troughIndex = 0;
  vals.forEach((v, i) => {
    if (v > vals[peakIndex]) peakIndex = i;
    if (v < vals[troughIndex]) troughIndex = i;
  });

  const points: ChartPoint[] = pts.map((p, i) => {
    const val = getPointValue(p, metric);
    const x = n > 1 ? padX + (i / (n - 1)) * (chartWidth - 2 * padX) : chartWidth / 2;
    const y = chartHeight - padY - ((val - minVal) / rangeY) * (chartHeight - 2 * padY);
    return { x, y, val, point: p };
  });

  const pathD = points
    .map((pt, i) => `${i === 0 ? 'M' : 'L'} ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`)
    .join(' ');
  const firstX = points[0].x;
  const lastX = points[points.length - 1].x;
  const areaD = `${pathD} L ${lastX.toFixed(1)} ${chartHeight} L ${firstX.toFixed(1)} ${chartHeight} Z`;

  const peakPoint = points[peakIndex] ?? null;
  const troughPoint = points[troughIndex] ?? null;

  return { pathD, areaD, points, minVal, maxVal, peakPoint, troughPoint };
}

export function calculateZeroLineY(
  minVal: number,
  maxVal: number,
  chartHeight = 240,
  padY = 24
): number | null {
  if (minVal < 0 && maxVal > 0) {
    const rangeY = maxVal - minVal;
    return chartHeight - padY - ((0 - minVal) / rangeY) * (chartHeight - 2 * padY);
  }
  return null;
}

export function computeTrendsDateRange(
  period: TrendsPeriod,
  earliestDate?: string
): { startStr: string; endStr: string } {
  const today = new Date();
  const format = (d: Date) =>
    `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  const todayStr = format(today);
  let startStr = todayStr;

  if (period === '1W') {
    const d = new Date(today);
    d.setDate(d.getDate() - 6);
    startStr = format(d);
  } else if (period === '1M') {
    const d = new Date(today);
    d.setDate(d.getDate() - 29);
    startStr = format(d);
  } else if (period === '3M') {
    const d = new Date(today);
    d.setDate(d.getDate() - 89);
    startStr = format(d);
  } else if (period === 'YTD') {
    startStr = `${today.getFullYear()}-01-01`;
  } else if (period === '1Y') {
    const d = new Date(today);
    d.setDate(d.getDate() - 364);
    startStr = format(d);
  } else if (period === 'ALL') {
    if (earliestDate) {
      startStr = earliestDate;
    } else {
      const d = new Date(today);
      d.setDate(d.getDate() - 364);
      startStr = format(d);
    }
  }

  return { startStr, endStr: todayStr };
}
