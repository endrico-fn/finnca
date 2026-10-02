import { describe, it, expect } from 'vitest';
import {
  calculateWaterfallLayout,
  linearScale,
  generateNiceTicks,
  type WaterfallStepInput,
} from './chartMath';

describe('calculateWaterfallLayout', () => {
  it('handles empty steps', () => {
    const res = calculateWaterfallLayout([], 800, 300);
    expect(res.bars).toHaveLength(0);
    expect(res.connectors).toHaveLength(0);
    expect(res.zeroY).toBe(150);
  });

  it('correctly calculates step positions for standard cash flow reconciliation', () => {
    const steps: WaterfallStepInput[] = [
      { id: 'start', label: 'Starting', amount: 10_000_000, isTotal: true },
      { id: 'inflow', label: 'Inflow', amount: 5_000_000 },
      { id: 'outflow', label: 'Outflow', amount: -3_000_000 },
      { id: 'investing', label: 'Investing', amount: -1_000_000 },
      { id: 'ending', label: 'Ending', amount: 11_000_000, isTotal: true },
    ];

    const res = calculateWaterfallLayout(steps, 800, 300);

    expect(res.bars).toHaveLength(5);
    expect(res.connectors).toHaveLength(4);

    // Starting cash: from 0 to 10M
    expect(res.bars[0].startVal).toBe(0);
    expect(res.bars[0].endVal).toBe(10_000_000);
    expect(res.bars[0].isTotal).toBe(true);

    // Inflow: from 10M to 15M
    expect(res.bars[1].startVal).toBe(10_000_000);
    expect(res.bars[1].endVal).toBe(15_000_000);
    expect(res.bars[1].tone).toBe('income');

    // Outflow: from 15M down to 12M
    expect(res.bars[2].startVal).toBe(15_000_000);
    expect(res.bars[2].endVal).toBe(12_000_000);
    expect(res.bars[2].tone).toBe('expense');

    // Investing: from 12M down to 11M
    expect(res.bars[3].startVal).toBe(12_000_000);
    expect(res.bars[3].endVal).toBe(11_000_000);
    expect(res.bars[3].tone).toBe('expense');

    // Ending cash: from 0 to 11M
    expect(res.bars[4].startVal).toBe(0);
    expect(res.bars[4].endVal).toBe(11_000_000);
    expect(res.bars[4].isTotal).toBe(true);

    // Connectors align horizontally:
    // Connector from bar 0 (endVal 10M) to bar 1 (startVal 10M)
    expect(res.connectors[0].y1).toBeCloseTo(res.bars[0].connectorY, 2);
    expect(res.connectors[0].y2).toBeCloseTo(res.bars[0].connectorY, 2);
    expect(res.connectors[0].x1).toBeLessThan(res.connectors[0].x2);
  });

  it('handles negative balances gracefully', () => {
    const steps: WaterfallStepInput[] = [
      { id: 'start', label: 'Starting', amount: 1_000, isTotal: true },
      { id: 'loss', label: 'Loss', amount: -3_000 },
      { id: 'ending', label: 'Ending', amount: -2_000, isTotal: true },
    ];

    const res = calculateWaterfallLayout(steps, 600, 200);
    expect(res.bars).toHaveLength(3);
    expect(res.minVal).toBeLessThan(0);
    expect(res.bars[1].endVal).toBe(-2_000);
  });

  it('handles zero starting balance with subsequent flows', () => {
    const steps: WaterfallStepInput[] = [
      { id: 'start', label: 'Starting', amount: 0, isTotal: true },
      { id: 'inflow', label: 'Inflow', amount: 5_000_000 },
      { id: 'outflow', label: 'Outflow', amount: -2_000_000 },
      { id: 'ending', label: 'Ending', amount: 3_000_000, isTotal: true },
    ];

    const res = calculateWaterfallLayout(steps, 800, 300);
    expect(res.bars).toHaveLength(4);
    expect(res.bars[0].startVal).toBe(0);
    expect(res.bars[0].endVal).toBe(0);
    expect(res.bars[1].startVal).toBe(0);
    expect(res.bars[1].endVal).toBe(5_000_000);
    expect(res.bars[2].startVal).toBe(5_000_000);
    expect(res.bars[2].endVal).toBe(3_000_000);
    expect(res.bars[3].endVal).toBe(3_000_000);
  });

  it('prevents bar collisions on narrow containers with many steps', () => {
    const manySteps: WaterfallStepInput[] = Array.from({ length: 15 }, (_, i) => ({
      id: `step-${i}`,
      label: `S${i}`,
      amount: i % 2 === 0 ? 100 : -50,
    }));

    const res = calculateWaterfallLayout(manySteps, 320, 200, 10, 10);
    expect(res.bars).toHaveLength(15);
    for (let i = 0; i < res.bars.length - 1; i++) {
      const currentRight = res.bars[i].x + res.bars[i].width;
      const nextLeft = res.bars[i + 1].x;
      expect(currentRight).toBeLessThanOrEqual(nextLeft);
    }
  });
});

describe('linearScale and generateNiceTicks', () => {
  it('scales values proportionally', () => {
    expect(linearScale(50, 0, 100, 0, 200)).toBe(100);
    expect(linearScale(0, 0, 100, 10, 20)).toBe(10);
    expect(linearScale(100, 0, 100, 10, 20)).toBe(20);
  });

  it('generates ticks spanning the range', () => {
    const ticks = generateNiceTicks(0, 100, 5);
    expect(ticks.length).toBeGreaterThan(0);
    expect(ticks[0]).toBeLessThanOrEqual(0);
    expect(ticks[ticks.length - 1]).toBeGreaterThanOrEqual(100);
  });
});
