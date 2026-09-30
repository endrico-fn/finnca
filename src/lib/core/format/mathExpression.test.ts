import { describe, it, expect } from 'vitest';
import { evaluateFinancialExpression, hasMathExpression } from './mathExpression';

describe('evaluateFinancialExpression', () => {
  it('evaluates basic numbers', () => {
    expect(evaluateFinancialExpression('50000', 'IDR')).toBe(50000);
    expect(evaluateFinancialExpression('150.50', 'USD')).toBe(150.5);
  });

  it('evaluates unit abbreviations k, rb, m, jt', () => {
    expect(evaluateFinancialExpression('50k', 'IDR')).toBe(50000);
    expect(evaluateFinancialExpression('25rb', 'IDR')).toBe(25000);
    expect(evaluateFinancialExpression('1.5m', 'IDR')).toBe(1500000);
    expect(evaluateFinancialExpression('2,5jt', 'IDR')).toBe(2500000);
  });

  it('evaluates arithmetic operations', () => {
    expect(evaluateFinancialExpression('50k + 12k', 'IDR')).toBe(62000);
    expect(evaluateFinancialExpression('100000 - 25000', 'IDR')).toBe(75000);
    expect(evaluateFinancialExpression('15000 * 3', 'IDR')).toBe(45000);
    expect(evaluateFinancialExpression('500000 / 2', 'IDR')).toBe(250000);
  });

  it('respects operator precedence and parentheses', () => {
    expect(evaluateFinancialExpression('10k + 20k * 3', 'IDR')).toBe(70000);
    expect(evaluateFinancialExpression('(10k + 20k) * 3', 'IDR')).toBe(90000);
  });

  it('handles decimal commas in units', () => {
    expect(evaluateFinancialExpression('1,5jt + 500k', 'IDR')).toBe(2000000);
  });

  it('handles division by zero and invalid inputs gracefully', () => {
    expect(evaluateFinancialExpression('100 / 0', 'IDR')).toBeNull();
    expect(evaluateFinancialExpression('abc + def', 'IDR')).toBeNull();
    expect(evaluateFinancialExpression('', 'IDR')).toBeNull();
  });
});

describe('hasMathExpression', () => {
  it('detects operators and units', () => {
    expect(hasMathExpression('50k + 12k')).toBe(true);
    expect(hasMathExpression('1.5jt')).toBe(true);
    expect(hasMathExpression('50000')).toBe(false);
  });
});

describe('parseStringAmountToMinor integration', () => {
  it('parses math expressions to integer minor units', async () => {
    const { parseStringAmountToMinor, getCurrencyPrefix, getCurrencyFactor, formatMinorGrouping } =
      await import('./currency');
    expect(parseStringAmountToMinor('50k + 12k', 'IDR')).toBe(62000);
    expect(parseStringAmountToMinor('1.5m', 'IDR')).toBe(1500000);
    expect(parseStringAmountToMinor('10.50 + 2.50', 'USD')).toBe(1300);
    expect(parseStringAmountToMinor('100000', 'IDR')).toBe(100000);
    expect(parseStringAmountToMinor('500,000', 'IDR')).toBe(500000);
    expect(parseStringAmountToMinor('1.000.000', 'IDR')).toBe(1000000);
    expect(parseStringAmountToMinor('1.000.000 + 500.000', 'IDR')).toBe(1500000);
    expect(parseStringAmountToMinor('1,000,000 + 500,000', 'IDR')).toBe(1500000);
    expect(parseStringAmountToMinor('500,000', 'USD')).toBe(50000000);
    expect(parseStringAmountToMinor('1,000,000', 'USD')).toBe(100000000);
    expect(parseStringAmountToMinor('1.000.000', 'USD')).toBe(100000000);
    expect(parseStringAmountToMinor('1,500.50', 'USD')).toBe(150050);
    expect(parseStringAmountToMinor('1.500,50', 'USD')).toBe(150050);
    expect(parseStringAmountToMinor('1,000 + 500', 'USD')).toBe(150000);
    expect(parseStringAmountToMinor('12.50', 'USD')).toBe(1250);

    expect(getCurrencyPrefix('IDR')).toBe('Rp');
    expect(getCurrencyPrefix('USD')).toBe('$');
    expect(getCurrencyPrefix('EUR')).toBe('€');
    expect(getCurrencyPrefix('GBP')).toBe('£');

    expect(getCurrencyFactor('IDR')).toBe(1);
    expect(getCurrencyFactor('USD')).toBe(100);
    expect(getCurrencyFactor('EUR')).toBe(100);

    expect(formatMinorGrouping(500000, 'IDR', 'en-US')).toBe('500,000');
    expect(formatMinorGrouping(150050, 'USD', 'en-US')).toBe('1,500.50');
    expect(formatMinorGrouping(10000, 'USD', 'en-US')).toBe('100.00');
  });
});
