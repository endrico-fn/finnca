import type { VaultData } from '../types';
import { convertMinor, roundHalfToEven } from '../core/math';

export interface FxRevaluationResult {
  accountId: string;
  nativeBalance: number;
  costBasisIdr: number;
  currentValueIdr: number;
  unrealizedGainIdr: number;
}

export function calculateFxRevaluation(vault: VaultData): {
  revaluations: FxRevaluationResult[];
  totalUnrealizedGain: number;
} {
  const foreignAccounts = vault.accounts.filter((a) => a.currency !== 'IDR' && !a.placeholder);
  const revaluations: FxRevaluationResult[] = [];
  let totalUnrealizedGain = 0;

  const currentFxRate = vault.fxRate || 15000;

  for (const acc of foreignAccounts) {
    let nativeBalance = 0;
    let remainingQty = 0;
    let costBasisIdr = 0;

    const orderedTxs = [...vault.transactions].sort((a, b) => a.date.localeCompare(b.date));
    for (const tx of orderedTxs) {
      for (const s of tx.splits) {
        if (s.accountId === acc.id) {
          nativeBalance += s.amount;

          const txRate = tx.fxRateAtTransaction || currentFxRate;
          if (s.amount > 0) {
            costBasisIdr += convertMinor(s.amount, acc.currency, 'IDR', txRate);
            remainingQty += s.amount;
          } else if (s.amount < 0 && remainingQty > 0) {
            costBasisIdr -= roundHalfToEven((costBasisIdr * -s.amount) / remainingQty);
            remainingQty += s.amount;
          } else {
            costBasisIdr += convertMinor(s.amount, acc.currency, 'IDR', txRate);
            remainingQty += s.amount;
          }
        }
      }
    }

    if (nativeBalance !== 0) {
      const currentValueIdr = convertMinor(nativeBalance, acc.currency, 'IDR', currentFxRate);
      const unrealizedGainIdr = currentValueIdr - costBasisIdr;

      revaluations.push({
        accountId: acc.id,
        nativeBalance,
        costBasisIdr,
        currentValueIdr,
        unrealizedGainIdr,
      });
      totalUnrealizedGain += unrealizedGainIdr;
    }
  }

  return { revaluations, totalUnrealizedGain };
}
