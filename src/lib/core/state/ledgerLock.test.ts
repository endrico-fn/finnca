import { describe, it, expect } from 'vitest';
import { ledgerLockState } from './ledgerLock.svelte';

describe('ledgerLockState', () => {
  it('unlocks all dates when closingDate is null', () => {
    ledgerLockState.closingDate = null;
    expect(ledgerLockState.isDateLocked('2026-01-01')).toBe(false);
    expect(ledgerLockState.isDateLocked('2026-12-31')).toBe(false);
    expect(ledgerLockState.isDateLocked(null)).toBe(false);
  });

  it('locks dates on or before closingDate', () => {
    ledgerLockState.closingDate = '2026-06-30';
    expect(ledgerLockState.isDateLocked('2026-01-01')).toBe(true);
    expect(ledgerLockState.isDateLocked('2026-06-30')).toBe(true);
    expect(ledgerLockState.isDateLocked('2026-07-01')).toBe(false);
    expect(ledgerLockState.isDateLocked('2026-12-31')).toBe(false);
  });
});
