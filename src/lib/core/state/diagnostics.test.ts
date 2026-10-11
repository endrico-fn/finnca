import { describe, it, expect, beforeEach } from 'vitest';
import { diagnosticsState } from './diagnostics.svelte';

describe('DiagnosticsStore', () => {
  beforeEach(() => {
    diagnosticsState.clear();
  });

  it('captures and sanitizes errors without leaking user home paths', () => {
    diagnosticsState.captureError(
      'error',
      'Crash reading /home/johndoe/vaults/finnca/data.db or C:\\Users\\Alice\\vault.finnca',
      'at /home/johndoe/project/file.ts:10:5'
    );

    expect(diagnosticsState.crashes.length).toBe(1);
    const crash = diagnosticsState.crashes[0];
    expect(crash.message).not.toContain('/home/johndoe');
    expect(crash.message).not.toContain('C:\\Users\\Alice');
    expect(crash.message).toContain('[REDACTED_PATH]');
    expect(crash.stack).toContain('[REDACTED_PATH]');
  });

  it('scrubs currency and monetary figures', () => {
    diagnosticsState.captureError(
      'unhandledrejection',
      'Failed transaction: 1500000.50 IDR balance exceeds 5000.00 USD, Rp 50.000, or $100.00'
    );

    const crash = diagnosticsState.crashes[0];
    expect(crash.message).not.toContain('1500000.50');
    expect(crash.message).not.toContain('IDR');
    expect(crash.message).not.toContain('USD');
    expect(crash.message).not.toContain('Rp');
    expect(crash.message).not.toContain('$');
    expect(crash.message).toContain('[NUMERIC_MASKED]');
    expect(crash.message).toContain('[CURRENCY_MASKED]');
  });

  it('scrubs sensitive account names and IDs', () => {
    diagnosticsState.captureError(
      'manual',
      'Ledger imbalance in Assets > Banking > BCA Personal and Aset > Kas & Bank > Dompet Tunai with acc_01HN7V9J6Q1111111111111111'
    );

    const crash = diagnosticsState.crashes[0];
    expect(crash.message).not.toContain('Assets > Banking > BCA Personal');
    expect(crash.message).not.toContain('Aset > Kas & Bank > Dompet Tunai');
    expect(crash.message).not.toContain('acc_01HN7V9J6Q1111111111111111');
    expect(crash.message).toContain('[ACCOUNT_MASKED]');
    expect(crash.message).toContain('[ACCOUNT_ID_MASKED]');
    expect(crash.message).toBe(
      'Ledger imbalance in [ACCOUNT_MASKED] and [ACCOUNT_MASKED] with [ACCOUNT_ID_MASKED]'
    );
  });

  it('does not swallow sentence conjunctions and handles colon-separated paths', () => {
    diagnosticsState.captureError(
      'error',
      'Error in Assets > Banking > Checking in transfer for account with note and Assets:Bank:BCA_Main with acc_cash'
    );

    const crash = diagnosticsState.crashes[0];
    // acc_cash is shorter than 10 characters so it remains intact as a variable name
    expect(crash.message).toBe(
      'Error in [ACCOUNT_MASKED] in transfer for account with note and [ACCOUNT_MASKED] with acc_cash'
    );
  });

  it('enforces maximum 10 stored crashes', () => {
    const letters = [
      'a',
      'b',
      'c',
      'd',
      'e',
      'f',
      'g',
      'h',
      'i',
      'j',
      'k',
      'l',
      'm',
      'n',
      'latest',
    ];
    for (const letter of letters) {
      diagnosticsState.captureError('error', `Error sequence ${letter}`);
    }

    expect(diagnosticsState.crashes.length).toBe(10);
    // Most recent is index 0
    expect(diagnosticsState.crashes[0].message).toContain('latest');
  });
});
