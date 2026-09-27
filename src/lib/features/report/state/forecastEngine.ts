import { todayString } from '$lib/core/format/date';
import type { PlanProgressView } from '$lib/core/ipc/bindings';

export type HorizonDays = 30 | 60 | 90 | 180;

export interface ProjectedCommitment {
  date: string;
  title: string;
  planType: string;
  amount: number;
  isInflow: boolean;
}

export interface TrajectoryPoint {
  date: string;
  balance: number;
  dayOffset: number;
}

export interface CashForecastSimulationResult {
  points: TrajectoryPoint[];
  commitments: ProjectedCommitment[];
  totalInflow: number;
  totalOutflow: number;
  depletionDay: number | null;
  depletionDate: string | null;
  endingBalance: number;
}

export interface ChartGeometryPoint extends TrajectoryPoint {
  x: number;
  y: number;
}

export interface ChartGeometry {
  width: number;
  height: number;
  padding: { top: number; right: number; bottom: number; left: number };
  innerWidth: number;
  innerHeight: number;
  coords: ChartGeometryPoint[];
  pathData: string;
  areaPath: string;
  zeroY: number;
  minVal: number;
  maxVal: number;
  hasZeroLine: boolean;
}

export function runCashForecastSimulation(
  liquidCash: number,
  dailyBaselineBurn: number,
  planItems: PlanProgressView[],
  horizon: HorizonDays
): CashForecastSimulationResult {
  const today = new Date();
  let currentBal = liquidCash;
  const points: TrajectoryPoint[] = [
    { date: todayString(today), balance: currentBal, dayOffset: 0 },
  ];
  const commitments: ProjectedCommitment[] = [];
  let totalInflow = 0;
  let totalOutflow = 0;
  let depletionDay: number | null = null;
  let depletionDate: string | null = null;

  const activePlans = planItems
    .filter((p) => p.plan.status === 'ACTIVE' && !p.is_settled)
    .map((p) => p.plan);

  for (let day = 1; day <= horizon; day++) {
    const d = new Date(today);
    d.setDate(today.getDate() + day);
    const dateStr = todayString(d);

    let dayInflow = 0;
    let dayOutflow = dailyBaselineBurn;

    for (const p of activePlans) {
      let triggers = false;
      if (p.due_date === dateStr) {
        triggers = true;
      } else if (p.frequency === 'DAILY') {
        triggers = true;
      } else if (p.frequency === 'WEEKLY') {
        const startDay = new Date(p.start_date).getDay();
        if (d.getDay() === startDay) triggers = true;
      } else if (p.frequency === 'MONTHLY') {
        const dom = p.day_of_month ?? new Date(p.start_date).getDate();
        if (d.getDate() === dom) triggers = true;
      }

      if (triggers) {
        const amt = p.installment_amount > 0 ? p.installment_amount : p.total_amount;
        const isInflow = p.plan_type === 'RECEIVABLE';
        if (isInflow) {
          dayInflow += amt;
        } else {
          dayOutflow += amt;
        }
        commitments.push({
          date: dateStr,
          title: p.title,
          planType: p.plan_type,
          amount: amt,
          isInflow,
        });
      }
    }

    totalInflow += dayInflow;
    totalOutflow += dayOutflow;
    currentBal = currentBal + dayInflow - dayOutflow;

    if (currentBal <= 0 && depletionDay === null) {
      depletionDay = day;
      depletionDate = dateStr;
    }

    points.push({ date: dateStr, balance: currentBal, dayOffset: day });
  }

  return {
    points,
    commitments,
    totalInflow,
    totalOutflow,
    depletionDay,
    depletionDate,
    endingBalance: currentBal,
  };
}

export function computeForecastGeometry(
  points: TrajectoryPoint[],
  width = 800,
  height = 220
): ChartGeometry | null {
  if (points.length < 2) return null;

  const padding = { top: 20, right: 30, bottom: 30, left: 60 };
  const innerWidth = width - padding.left - padding.right;
  const innerHeight = height - padding.top - padding.bottom;

  const balances = points.map((p) => p.balance);
  const minVal = Math.min(0, ...balances);
  const maxVal = Math.max(1, ...balances);
  const valRange = maxVal - minVal || 1;

  const coords = points.map((p, idx) => {
    const x = padding.left + (idx / (points.length - 1)) * innerWidth;
    const y = padding.top + (1 - (p.balance - minVal) / valRange) * innerHeight;
    return { x, y, ...p };
  });

  const pathData = coords.reduce(
    (acc, c, idx) => (idx === 0 ? `M ${c.x} ${c.y}` : `${acc} L ${c.x} ${c.y}`),
    ''
  );

  const zeroY = padding.top + (1 - (0 - minVal) / valRange) * innerHeight;
  const areaBottom = Math.min(innerHeight + padding.top, zeroY);

  const areaPath =
    coords.length > 0
      ? `${pathData} L ${coords[coords.length - 1].x} ${areaBottom} L ${coords[0].x} ${areaBottom} Z`
      : '';

  return {
    width,
    height,
    padding,
    innerWidth,
    innerHeight,
    coords,
    pathData,
    areaPath,
    zeroY,
    minVal,
    maxVal,
    hasZeroLine: minVal < 0 && maxVal > 0,
  };
}
