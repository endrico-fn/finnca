import { describe, it, expect } from 'vitest';
import { checkTransferLegs, checkTransferAmounts } from './transferValidation';

describe('transferValidation', () => {
  it('rejects transfer when source or destination is missing', () => {
    expect(checkTransferLegs('', 'acc-2', 100_000, '', 0)).toBe('MISSING_SIDE');
    expect(checkTransferLegs('acc-1', '', 100_000, '', 0)).toBe('MISSING_SIDE');
  });

  it('rejects transfer when source and destination are identical', () => {
    expect(checkTransferLegs('acc-1', 'acc-1', 100_000, '', 0)).toBe('SAME_SIDE');
  });

  it('rejects transfer with non-positive amount', () => {
    expect(checkTransferLegs('acc-1', 'acc-2', 0, '', 0)).toBe('BAD_AMOUNT');
    expect(checkTransferLegs('acc-1', 'acc-2', -50_000, '', 0)).toBe('BAD_AMOUNT');
  });

  it('rejects transfer when fee amount is positive but fee account is missing', () => {
    expect(checkTransferLegs('acc-1', 'acc-2', 100_000, '', 5_000)).toBe('FEE_ACCOUNT_MISSING');
  });

  it('rejects transfer when fee account is set but fee amount is non-positive', () => {
    expect(checkTransferLegs('acc-1', 'acc-2', 100_000, 'acc-fee', 0)).toBe('FEE_BAD_AMOUNT');
    expect(checkTransferLegs('acc-1', 'acc-2', 100_000, 'acc-fee', -2_500)).toBe('FEE_BAD_AMOUNT');
  });

  it('rejects transfer when fee account is identical to source or target', () => {
    expect(checkTransferLegs('acc-1', 'acc-2', 100_000, 'acc-1', 5_000)).toBe('FEE_IS_SIDE');
    expect(checkTransferLegs('acc-1', 'acc-2', 100_000, 'acc-2', 5_000)).toBe('FEE_IS_SIDE');
  });

  it('passes valid transfer without fee', () => {
    expect(checkTransferLegs('acc-1', 'acc-2', 500_000, '', 0)).toBeNull();
  });

  it('passes valid transfer with fee to dedicated expense account', () => {
    expect(checkTransferLegs('acc-1', 'acc-2', 500_000, 'acc-admin-fee', 6_500)).toBeNull();
  });

  describe('checkTransferAmounts', () => {
    it('returns BAD_AMOUNT on non-positive transfer amounts', () => {
      expect(checkTransferAmounts(0, '', 0)).toBe('BAD_AMOUNT');
      expect(checkTransferAmounts(-100, '', 0)).toBe('BAD_AMOUNT');
    });

    it('validates fee account consistency', () => {
      expect(checkTransferAmounts(100_000, '', 5_000)).toBe('FEE_ACCOUNT_MISSING');
      expect(checkTransferAmounts(100_000, 'fee-acc', 0)).toBe('FEE_BAD_AMOUNT');
      expect(checkTransferAmounts(100_000, 'from-acc', 5_000, 'from-acc', 'to-acc')).toBe(
        'FEE_IS_SIDE'
      );
      expect(checkTransferAmounts(100_000, 'to-acc', 5_000, 'from-acc', 'to-acc')).toBe(
        'FEE_IS_SIDE'
      );
      expect(checkTransferAmounts(100_000, 'fee-acc', 5_000, 'from-acc', 'to-acc')).toBeNull();
    });
  });
});
