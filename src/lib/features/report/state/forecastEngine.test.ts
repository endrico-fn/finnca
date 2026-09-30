import { describe, it, expect } from 'vitest';
import { runCashForecastSimulation, computeForecastGeometry } from './forecastEngine';
import type { PlanProgressView } from '$lib/core/ipc/bindings';

describe('forecastEngine', () => {
  it('simulates sustainable cash runway when baseline burn is zero and no plans', () => {
    const res = runCashForecastSimulation(10_000_000, 0, [], 60);
    expect(res.points.length).toBe(61);
    expect(res.endingBalance).toBe(10_000_000);
    expect(res.depletionDay).toBeNull();
    expect(res.depletionDate).toBeNull();
    expect(res.totalInflow).toBe(0);
    expect(res.totalOutflow).toBe(0);
  });

  it('detects depletion day accurately with constant burn rate', () => {
    const res = runCashForecastSimulation(1_000_000, 200_000, [], 30);
    expect(res.depletionDay).toBe(5);
    expect(res.depletionDate).toBeDefined();
    expect(res.endingBalance).toBe(1_000_000 - 30 * 200_000);
    expect(res.totalOutflow).toBe(30 * 200_000);
  });

  it('correctly aggregates recurring receivables as inflows', () => {
    const mockPlan: PlanProgressView = {
      plan: {
        id: 'plan-1',
        title: 'Freelance Inflow',
        plan_type: 'RECEIVABLE',
        status: 'ACTIVE',
        total_amount: 5_000_000,
        installment_amount: 5_000_000,
        frequency: 'DAILY',
        start_date: '2026-01-01',
        due_date: null,
        day_of_month: null,
        from_account_id: 'acc-client',
        to_account_id: 'acc-bank',
        notes: null,
        last_posted_date: null,
        auto_post: false,
        created_at: 1700000000,
      },
      paid_amount: 0,
      remaining_amount: 5_000_000,
      progress_percent: 0,
      is_settled: false,
      installments_paid_count: 0,
    };

    const res = runCashForecastSimulation(1_000_000, 500_000, [mockPlan], 30);
    expect(res.totalInflow).toBe(30 * 5_000_000);
    expect(res.totalOutflow).toBe(30 * 500_000);
    expect(res.endingBalance).toBe(1_000_000 + 30 * (5_000_000 - 500_000));
    expect(res.depletionDay).toBeNull();
  });

  it('computes chart geometry with normalized svg coordinates', () => {
    const points = [
      { date: '2026-09-01', balance: 1_000_000, dayOffset: 0 },
      { date: '2026-09-02', balance: 2_000_000, dayOffset: 1 },
      { date: '2026-09-03', balance: 500_000, dayOffset: 2 },
    ];
    const geom = computeForecastGeometry(points, 800, 220);
    expect(geom).not.toBeNull();
    if (!geom) return;

    expect(geom.coords.length).toBe(3);
    expect(geom.coords[0].x).toBe(geom.padding.left);
    expect(geom.coords[2].x).toBe(800 - geom.padding.right);
    expect(geom.pathData).toMatch(/^M \d+(\.\d+)? \d+(\.\d+)? L/);
    expect(geom.areaPath).toContain('Z');
  });

  it('returns null for geometry when points length is less than 2', () => {
    expect(computeForecastGeometry([])).toBeNull();
    expect(
      computeForecastGeometry([{ date: '2026-09-01', balance: 100, dayOffset: 0 }])
    ).toBeNull();
  });
});
