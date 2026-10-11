import { describe, it, expect } from 'vitest';
import {
  createTwoLegPostings,
  createEmptyJournalDraft,
  createTransferDraft,
  cloneJournalEntry,
  createReconciledStatementDraft,
  initializeInspectorForm,
  computeImbalance,
  computeTotals,
  convertSimpleToSplits,
  buildSimpleSplitsWithFee,
  extractSimpleFromSplits,
  autoBalanceSplits,
  updateSplitAmount,
  validateDraft,
} from './journalFormUtils';
import type { Account, JournalEntryView, PostingInput } from '$lib/core/ipc/bindings';

describe('journalFormUtils', () => {
  describe('createTwoLegPostings', () => {
    it('creates debit and credit legs with balanced amounts', () => {
      const postings = createTwoLegPostings('acc-from', 'acc-to', 50000);
      expect(postings).toHaveLength(2);
      expect(postings[0].account_id).toBe('acc-to');
      expect(postings[0].amount).toBe(50000);
      expect(postings[1].account_id).toBe('acc-from');
      expect(postings[1].amount).toBe(-50000);
      expect(computeImbalance(postings)).toBe(0);
    });
  });

  describe('createEmptyJournalDraft', () => {
    it('initializes a fresh balanced draft with specified accounts and currency', () => {
      const draft = createEmptyJournalDraft('acc-from', 'acc-to', 'USD');
      expect(draft.id).toBeDefined();
      expect(draft.currency).toBe('USD');
      expect(draft.postings).toHaveLength(2);
      expect(draft.postings[0].account_id).toBe('acc-to');
      expect(draft.postings[0].amount).toBe(0);
      expect(draft.postings[1].account_id).toBe('acc-from');
      expect(draft.postings[1].amount).toBe(0);
      expect(computeImbalance(draft.postings)).toBe(0);
    });
  });

  describe('createTransferDraft', () => {
    it('creates balanced transfer draft between accounts with debit first', () => {
      const draft = createTransferDraft({
        fromId: 'bank-1',
        toId: 'wallet-1',
        amount: 250000,
        description: 'ATM Cash Withdrawal',
        notes: 'Monthly pocket cash',
        currency: 'IDR',
        date: '2026-10-02',
      });
      expect(draft.description).toBe('ATM Cash Withdrawal');
      expect(draft.notes).toBe('Monthly pocket cash');
      expect(draft.date).toBe('2026-10-02');
      expect(draft.currency).toBe('IDR');
      expect(draft.postings).toHaveLength(2);
      expect(draft.postings[0].account_id).toBe('wallet-1');
      expect(draft.postings[0].amount).toBe(250000);
      expect(draft.postings[1].account_id).toBe('bank-1');
      expect(draft.postings[1].amount).toBe(-250000);
      expect(computeImbalance(draft.postings)).toBe(0);
    });

    it('handles defaults when optional fields are omitted', () => {
      const draft = createTransferDraft({});
      expect(draft.description).toBe('');
      expect(draft.currency).toBe('IDR');
      expect(draft.postings[0].amount).toBe(0);
      expect(draft.postings[1].amount).toBe(-0);
    });
  });

  describe('cloneJournalEntry', () => {
    const originalEntry: JournalEntryView = {
      id: 'tx-orig',
      date: '2026-09-15',
      description: 'Monthly Cloud Hosting',
      notes: 'Server renewal',
      reference_no: 'INV-2026-09',
      due_date: '2026-09-30',
      plan_id: null,
      currency: 'USD',
      fx_rate: 15500,
      posted_at: 1700000000,
      postings: [
        {
          id: 'p1',
          entry_id: 'tx-orig',
          account_id: 'acc-exp',
          account_code: '5100',
          account_name: 'Hosting Expense',
          account_type: 'EXPENSE',
          amount: 5000,
          memo: 'Linode',
          action: null,
          reconcile: 'c',
          reconciled_at: 1700000500,
          currency: 'USD',
          fx_rate: null,
          cost_amount: null,
        },
        {
          id: 'p2',
          entry_id: 'tx-orig',
          account_id: 'acc-bank',
          account_code: '1001',
          account_name: 'Corporate Card',
          account_type: 'LIABILITY',
          amount: -5000,
          memo: 'Card charge',
          action: null,
          reconcile: 'c',
          reconciled_at: 1700000500,
          currency: 'USD',
          fx_rate: null,
          cost_amount: null,
        },
      ],
    };

    it('clones entry with a new ID, fresh date, (Copy) suffix, and reset reconcile statuses', () => {
      const clone = cloneJournalEntry(originalEntry);
      expect(clone.id).not.toBe(originalEntry.id);
      expect(clone.description).toBe('Monthly Cloud Hosting (Copy)');
      expect(clone.reference_no).toBeNull();
      expect(clone.due_date).toBeNull();
      expect(clone.currency).toBe('USD');
      expect(clone.fx_rate).toBe(15500);
      expect(clone.notes).toBe('Server renewal');
      expect(clone.postings).toHaveLength(2);
      expect(clone.postings[0].id).not.toBe('p1');
      expect(clone.postings[0].reconcile).toBe('n');
      expect(clone.postings[1].reconcile).toBe('n');
      expect(computeImbalance(clone.postings)).toBe(0);
    });

    it('respects copy options when disabling suffix or retaining original date', () => {
      const clone = cloneJournalEntry(originalEntry, { copySuffix: false, resetDate: false });
      expect(clone.description).toBe('Monthly Cloud Hosting');
      expect(clone.date).toBe('2026-09-15');
    });
  });

  describe('createReconciledStatementDraft', () => {
    it('creates draft for income with bank account debited and cleared', () => {
      const draft = createReconciledStatementDraft({
        date: '2026-10-01',
        description: 'Interest Payment',
        amount: 15000,
        currency: 'IDR',
        bankAccountId: 'bank-acc',
        offsetAccountId: 'income-acc',
      });

      expect(draft.date).toBe('2026-10-01');
      expect(draft.description).toBe('Interest Payment');
      expect(draft.postings).toHaveLength(2);
      // Income: bank is debited (+15000, cleared 'c'), offset is credited (-15000, 'n')
      expect(draft.postings[0].account_id).toBe('bank-acc');
      expect(draft.postings[0].amount).toBe(15000);
      expect(draft.postings[0].reconcile).toBe('c');
      expect(draft.postings[1].account_id).toBe('income-acc');
      expect(draft.postings[1].amount).toBe(-15000);
      expect(draft.postings[1].reconcile).toBe('n');
      expect(computeImbalance(draft.postings)).toBe(0);
    });

    it('creates draft for expense with bank account credited and cleared', () => {
      const draft = createReconciledStatementDraft({
        date: '2026-10-02',
        description: 'Bank Admin Fee',
        amount: -6500,
        currency: 'IDR',
        bankAccountId: 'bank-acc',
        offsetAccountId: 'fee-acc',
      });

      expect(draft.postings).toHaveLength(2);
      // Expense: offset is debited (+6500), bank is credited (-6500, cleared 'c')
      expect(draft.postings[0].account_id).toBe('fee-acc');
      expect(draft.postings[0].amount).toBe(6500);
      expect(draft.postings[1].account_id).toBe('bank-acc');
      expect(draft.postings[1].amount).toBe(-6500);
      expect(draft.postings[1].reconcile).toBe('c');
      expect(computeImbalance(draft.postings)).toBe(0);
    });
  });

  describe('initializeInspectorForm', () => {
    const mockAccounts: Account[] = [
      {
        id: 'acc-asset-1',
        code: '1000',
        name: 'Bank BCA',
        account_type: 'ASSET',
        parent_id: null,
        currency: 'IDR',
        placeholder: false,
        hidden: false,
        color: null,
        note: null,
        description: null,
        interest_rate: null,
        created_at: 0,
      },
      {
        id: 'acc-exp-1',
        code: '5000',
        name: 'Food & Dining',
        account_type: 'EXPENSE',
        parent_id: null,
        currency: 'IDR',
        placeholder: false,
        hidden: false,
        color: null,
        note: null,
        description: null,
        interest_rate: null,
        created_at: 0,
      },
      {
        id: 'acc-inc-1',
        code: '4000',
        name: 'Salary',
        account_type: 'INCOME',
        parent_id: null,
        currency: 'IDR',
        placeholder: false,
        hidden: false,
        color: null,
        note: null,
        description: null,
        interest_rate: null,
        created_at: 0,
      },
    ];

    it('initializes default form state when no entry or draft is provided', () => {
      const state = initializeInspectorForm({
        defaultFxRate: 1.0,
        leafAccounts: mockAccounts,
      });

      expect(state.mode).toBe('transfer');
      expect(state.currency).toBe('IDR');
      expect(state.fxRate).toBe(1.0);
      expect(state.postings).toHaveLength(2);
      expect(state.standardFrom).toBe('acc-asset-1');
      expect(state.standardTo).toBe('acc-exp-1');
      expect(state.simpleCategory).toBe('EXPENSE');
    });

    it('initializes form state correctly from an existing 2-leg entry', () => {
      const entry: JournalEntryView = {
        id: 'e1',
        date: '2026-10-01',
        description: 'Dinner with client',
        notes: 'Business expense [Ref: INV-1] [Due: 2026-10-15] [Settled]',
        reference_no: null,
        due_date: null,
        plan_id: null,
        currency: 'IDR',
        fx_rate: 1.0,
        posted_at: 1700000000,
        postings: [
          {
            id: 'p1',
            entry_id: 'e1',
            account_id: 'acc-exp-1',
            account_code: '5000',
            account_name: 'Food',
            account_type: 'EXPENSE',
            amount: 75000,
            memo: null,
            action: null,
            reconcile: 'n',
            reconciled_at: null,
            currency: 'IDR',
            fx_rate: null,
            cost_amount: null,
          },
          {
            id: 'p2',
            entry_id: 'e1',
            account_id: 'acc-asset-1',
            account_code: '1000',
            account_name: 'Bank BCA',
            account_type: 'ASSET',
            amount: -75000,
            memo: null,
            action: null,
            reconcile: 'n',
            reconciled_at: null,
            currency: 'IDR',
            fx_rate: null,
            cost_amount: null,
          },
        ],
      };

      const state = initializeInspectorForm({
        entry,
        defaultFxRate: 1.0,
        leafAccounts: mockAccounts,
      });

      expect(state.mode).toBe('transfer');
      expect(state.description).toBe('Dinner with client');
      expect(state.standardFrom).toBe('acc-asset-1');
      expect(state.standardTo).toBe('acc-exp-1');
      expect(state.standardAmount).toBe('75000');
      expect(state.num).toBe('INV-1');
      expect(state.dueDate).toBe('2026-10-15');
      expect(state.settled).toBe(true);
      expect(state.notes).toBe('Business expense');
    });

    it('initializes multi-split journal mode when entry has > 2 legs', () => {
      const entry: JournalEntryView = {
        id: 'e-split',
        date: '2026-10-01',
        description: 'Multi split transaction',
        notes: '',
        reference_no: null,
        due_date: null,
        plan_id: null,
        currency: 'IDR',
        fx_rate: 1.0,
        posted_at: 1700000000,
        postings: [
          {
            id: 'p1',
            entry_id: 'e-split',
            account_id: 'acc-asset-1',
            account_code: '1000',
            account_name: 'Bank',
            account_type: 'ASSET',
            amount: -100000,
            memo: null,
            action: null,
            reconcile: 'n',
            reconciled_at: null,
            currency: 'IDR',
            fx_rate: null,
            cost_amount: null,
          },
          {
            id: 'p2',
            entry_id: 'e-split',
            account_id: 'acc-exp-1',
            account_code: '5000',
            account_name: 'Food',
            account_type: 'EXPENSE',
            amount: 60000,
            memo: null,
            action: null,
            reconcile: 'n',
            reconciled_at: null,
            currency: 'IDR',
            fx_rate: null,
            cost_amount: null,
          },
          {
            id: 'p3',
            entry_id: 'e-split',
            account_id: 'acc-exp-1',
            account_code: '5000',
            account_name: 'Food',
            account_type: 'EXPENSE',
            amount: 40000,
            memo: null,
            action: null,
            reconcile: 'n',
            reconciled_at: null,
            currency: 'IDR',
            fx_rate: null,
            cost_amount: null,
          },
        ],
      };

      const state = initializeInspectorForm({
        entry,
        defaultFxRate: 1.0,
        leafAccounts: mockAccounts,
      });

      expect(state.mode).toBe('journal');
      expect(state.postings).toHaveLength(3);
    });

    it('preserves from and to accounts even when transfer amount is 0', () => {
      const draft = createTransferDraft({
        fromId: 'acc-asset-1',
        toId: 'acc-exp-1',
        amount: 0,
      });

      const state = initializeInspectorForm({
        initialDraft: draft,
        defaultFxRate: 1.0,
        leafAccounts: mockAccounts,
      });

      expect(state.mode).toBe('transfer');
      expect(state.standardFrom).toBe('acc-asset-1');
      expect(state.standardTo).toBe('acc-exp-1');
    });

    it('preserves specified single from-account and resolves default to-account', () => {
      const draft = createEmptyJournalDraft('acc-asset-1', '', 'IDR');

      const state = initializeInspectorForm({
        initialDraft: draft,
        defaultFxRate: 1.0,
        leafAccounts: mockAccounts,
      });

      expect(state.standardFrom).toBe('acc-asset-1');
      expect(state.standardTo).toBe('acc-exp-1');
    });

    it('assigns unique id to each posting when initialDraft postings lack id', () => {
      const draft = {
        date: '2026-10-01',
        description: 'No IDs',
        postings: [
          { account_id: 'acc-asset-1', amount: 100 },
          { account_id: 'acc-exp-1', amount: -100 },
        ],
      };

      const state = initializeInspectorForm({
        initialDraft: draft,
        defaultFxRate: 1.0,
        leafAccounts: mockAccounts,
      });

      expect(state.postings[0].id).toBeDefined();
      expect(state.postings[1].id).toBeDefined();
      expect(state.postings[0].id).not.toBe(state.postings[1].id);
    });
  });

  describe('math and split helper invariants', () => {
    it('computes totals correctly', () => {
      const postings: PostingInput[] = [
        { id: '1', account_id: 'a', amount: 100, reconcile: 'n' },
        { id: '2', account_id: 'b', amount: 250, reconcile: 'n' },
        { id: '3', account_id: 'c', amount: -350, reconcile: 'n' },
      ];
      const { debitTotal, creditTotal } = computeTotals(postings);
      expect(debitTotal).toBe(350);
      expect(creditTotal).toBe(350);
      expect(computeImbalance(postings)).toBe(0);
    });

    it('builds simple splits with admin fee', () => {
      const splits = buildSimpleSplitsWithFee(
        'acc-bank',
        'acc-vendor',
        100000,
        2500,
        'acc-fee',
        []
      );
      expect(splits).toHaveLength(3);
      expect(splits[0].account_id).toBe('acc-vendor');
      expect(splits[0].amount).toBe(100000);
      expect(splits[1].account_id).toBe('acc-fee');
      expect(splits[1].amount).toBe(2500);
      expect(splits[2].account_id).toBe('acc-bank');
      expect(splits[2].amount).toBe(-102500);
      expect(computeImbalance(splits)).toBe(0);
    });

    it('converts simple to splits without fee', () => {
      const splits = convertSimpleToSplits('acc-src', 'acc-dst', 50000, []);
      expect(splits).toHaveLength(2);
      expect(splits[0].account_id).toBe('acc-dst');
      expect(splits[0].amount).toBe(50000);
      expect(splits[1].account_id).toBe('acc-src');
      expect(splits[1].amount).toBe(-50000);
      expect(computeImbalance(splits)).toBe(0);
    });

    it('extracts simple view from balanced 2-leg postings', () => {
      const postings: PostingInput[] = [
        { id: '1', account_id: 'acc-to', amount: 15000, reconcile: 'n' },
        { id: '2', account_id: 'acc-from', amount: -15000, reconcile: 'n' },
      ];
      const simple = extractSimpleFromSplits(postings, 'IDR');
      expect(simple).not.toBeNull();
      expect(simple?.standardTo).toBe('acc-to');
      expect(simple?.standardFrom).toBe('acc-from');
      expect(simple?.standardAmount).toBe('15000');
    });

    it('autobalances splits by filling an empty posting or appending a balancing leg', () => {
      const initial: PostingInput[] = [
        { id: '1', account_id: 'a', amount: 100, reconcile: 'n' },
        { id: '2', account_id: 'b', amount: 0, reconcile: 'n' },
      ];
      const balanced = autoBalanceSplits(initial, 100);
      expect(balanced[1].amount).toBe(-100);
      expect(computeImbalance(balanced)).toBe(0);

      const appendTest: PostingInput[] = [{ id: '1', account_id: 'a', amount: 50, reconcile: 'n' }];
      const balancedAppend = autoBalanceSplits(appendTest, 50);
      expect(balancedAppend).toHaveLength(2);
      expect(balancedAppend[1].amount).toBe(-50);
      expect(computeImbalance(balancedAppend)).toBe(0);
    });

    it('updates split amount and auto-mirrors 2-leg entries', () => {
      const postings: PostingInput[] = [
        { id: '1', account_id: 'a', amount: 1000, reconcile: 'n' },
        { id: '2', account_id: 'b', amount: -1000, reconcile: 'n' },
      ];
      const updated = updateSplitAmount(postings, '1', 'debit', '2500', 'IDR');
      expect(updated[0].amount).toBe(2500);
      expect(updated[1].amount).toBe(-2500);
      expect(computeImbalance(updated)).toBe(0);
    });
  });

  describe('validateDraft', () => {
    it('validates transfer form requirements', () => {
      const draft = createEmptyJournalDraft('acc-1', 'acc-2');
      draft.description = '';
      const res1 = validateDraft(draft, false, 'acc-1', 'acc-2', '1000');
      expect(res1.valid).toBe(false);

      draft.description = 'Valid description';
      const res2 = validateDraft(draft, false, '', 'acc-2', '1000');
      expect(res2.valid).toBe(false);

      const res3 = validateDraft(draft, false, 'acc-1', 'acc-1', '1000');
      expect(res3.valid).toBe(false);

      const res4 = validateDraft(draft, false, 'acc-1', 'acc-2', '0');
      expect(res4.valid).toBe(false);

      const res5 = validateDraft(draft, false, 'acc-1', 'acc-2', '50000');
      expect(res5.valid).toBe(true);
    });

    it('validates multi-split balance and missing accounts', () => {
      const draft = createEmptyJournalDraft('acc-1', 'acc-2');
      draft.description = 'Split entry';
      draft.postings = [
        { id: '1', account_id: '', amount: 100, reconcile: 'n' },
        { id: '2', account_id: 'acc-2', amount: -100, reconcile: 'n' },
      ];
      const resMissingAcc = validateDraft(draft, true, '', '', '');
      expect(resMissingAcc.valid).toBe(false);

      draft.postings = [
        { id: '1', account_id: 'acc-1', amount: 100, reconcile: 'n' },
        { id: '2', account_id: 'acc-2', amount: -80, reconcile: 'n' },
      ];
      const resUnbalanced = validateDraft(draft, true, '', '', '');
      expect(resUnbalanced.valid).toBe(false);

      draft.postings = [
        { id: '1', account_id: 'acc-1', amount: 100, reconcile: 'n' },
        { id: '2', account_id: 'acc-2', amount: -100, reconcile: 'n' },
      ];
      const resBalanced = validateDraft(draft, true, '', '', '');
      expect(resBalanced.valid).toBe(true);
    });
  });
});
