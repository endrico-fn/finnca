export type TransferAmountBlock =
  'BAD_AMOUNT' | 'FEE_BAD_AMOUNT' | 'FEE_ACCOUNT_MISSING' | 'FEE_IS_SIDE';

export type TransferLegsBlock = 'MISSING_SIDE' | 'SAME_SIDE' | TransferAmountBlock;

export function checkTransferAmounts(
  amountMinor: number,
  feeAccountId: string,
  feeMinor: number,
  fromId = '',
  toId = ''
): TransferAmountBlock | null {
  if (amountMinor <= 0) return 'BAD_AMOUNT';
  if (feeAccountId && feeMinor <= 0) return 'FEE_BAD_AMOUNT';
  if (!feeAccountId && feeMinor > 0) return 'FEE_ACCOUNT_MISSING';
  if (feeMinor > 0 && fromId && (feeAccountId === fromId || feeAccountId === toId))
    return 'FEE_IS_SIDE';
  return null;
}

export function checkTransferLegs(
  fromId: string,
  toId: string,
  amountMinor: number,
  feeAccountId: string,
  feeMinor: number
): TransferLegsBlock | null {
  if (!fromId || !toId) return 'MISSING_SIDE';
  if (fromId === toId) return 'SAME_SIDE';
  return checkTransferAmounts(amountMinor, feeAccountId, feeMinor, fromId, toId);
}
