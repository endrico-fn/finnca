import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { parseStringAmountToMinor, formatMinorToDisplay, getCurrencyFactor } from './currency';

describe('currency float-display fixes (RED)', () => {
  beforeEach(() => {
    (globalThis as unknown as { window: unknown }).window = {
      localStorage: {
        _m: new Map<string, string>([['finnca_num_format', 'comma']]),
        getItem(k: string) {
          return (this._m as Map<string, string>).get(k) ?? null;
        },
        setItem(k: string, v: string) {
          (this._m as Map<string, string>).set(k, v);
        },
        removeItem(k: string) {
          (this._m as Map<string, string>).delete(k);
        },
      },
    };
  });

  afterEach(() => {
    delete (globalThis as unknown as { window?: unknown }).window;
  });

  it('parses single-thousand dot for USD as 1000.00', () => {
    expect(parseStringAmountToMinor('1.000', 'USD')).toBe(100000);
  });

  it('parses 10.000 USD as 10000.00', () => {
    expect(parseStringAmountToMinor('10.000', 'USD')).toBe(1000000);
  });

  it('keeps IDR dot-grouping working', () => {
    expect(parseStringAmountToMinor('1.000', 'IDR')).toBe(1000);
    expect(parseStringAmountToMinor('1.000.000', 'IDR')).toBe(1000000);
  });

  it('fallback generic currency divides by 100', () => {
    const out = formatMinorToDisplay(150, 'INVALID', { locale: 'en-US' });
    expect(out).toContain('1.50');
    expect(out).not.toContain('150');
  });

  it('USD dot-pref keeps $ prefix (id-ID, not de-DE suffix)', () => {
    const w = globalThis.window as unknown as { localStorage: { _m: Map<string, string> } };
    w.localStorage._m.set('finnca_num_format', 'dot');
    const out = formatMinorToDisplay(150050, 'USD');
    expect(out.startsWith('US$') || out.startsWith('$')).toBe(true);
    expect(out.trim().endsWith('$')).toBe(false);
  });

  it('showSign positive adds +, negative stays -', () => {
    const pos = formatMinorToDisplay(5000, 'IDR', { showSign: true });
    const neg = formatMinorToDisplay(-5000, 'IDR', { showSign: true });
    expect(pos.startsWith('+')).toBe(true);
    expect(neg.startsWith('-')).toBe(true);
  });

  it('sanity factor', () => {
    expect(getCurrencyFactor('IDR')).toBe(1);
    expect(getCurrencyFactor('USD')).toBe(100);
  });
});
