import { describe, it, expect, beforeEach } from 'vitest';
import { modalState } from './modal.svelte';
import type { JournalEntryView } from '$lib/core/ipc/bindings';

describe('modalState', () => {
  beforeEach(() => {
    modalState.closeInspector();
    modalState.closeSettings();
    modalState.closeCommandPalette();
    modalState.closeHealthPulse();
    modalState.closeConfirm();
  });

  it('opens new entry with instant in-app inspector', () => {
    expect(modalState.inspectorOpen).toBe(false);
    modalState.openNewEntry();
    expect(modalState.inspectorOpen).toBe(true);
    expect(modalState.inspectorMode).toBe('journal');
    expect(modalState.inspectorIsNew).toBe(true);
    expect(modalState.inspectorDraft).not.toBeNull();
  });

  it('opens transfer mode with instant in-app inspector', () => {
    modalState.openTransfer('acc-1', 'acc-2');
    expect(modalState.inspectorOpen).toBe(true);
    expect(modalState.inspectorMode).toBe('transfer');
    expect(modalState.inspectorInitialFrom).toBe('acc-1');
    expect(modalState.inspectorInitialTo).toBe('acc-2');
    expect(modalState.isTransferMode).toBe(true);
  });

  it('opens new entry with explicit intent helper openNewEntry', () => {
    modalState.openNewEntry();
    expect(modalState.inspectorOpen).toBe(true);
    expect(modalState.inspectorMode).toBe('journal');
    expect(modalState.inspectorIsNew).toBe(true);
    expect(modalState.inspectorDraft).not.toBeNull();
    expect(modalState.isTransferMode).toBe(false);
  });

  it('opens existing entry with explicit intent helper openExistingEntry', () => {
    const mockEntry: JournalEntryView = {
      id: 'tx-simple',
      date: '2026-10-01',
      description: 'Simple salary',
      notes: null,
      reference_no: null,
      due_date: null,
      plan_id: null,
      currency: 'IDR',
      fx_rate: 1.0,
      posted_at: 1700000000,
      postings: [
        {
          id: 'p1',
          entry_id: 'tx-simple',
          account_id: 'a1',
          account_code: '1000',
          account_name: 'Cash',
          account_type: 'ASSET',
          amount: 50000,
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
          entry_id: 'tx-simple',
          account_id: 'a2',
          account_code: '4000',
          account_name: 'Salary',
          account_type: 'INCOME',
          amount: -50000,
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

    modalState.openExistingEntry(mockEntry);
    expect(modalState.inspectorOpen).toBe(true);
    expect(modalState.inspectorIsNew).toBe(false);
    expect(modalState.inspectorEntry).toBe(mockEntry);
    expect(modalState.inspectorMode).toBe('transfer'); // 2 postings defaults to transfer mode
    expect(modalState.isTransferMode).toBe(true);
  });

  it('opens existing entry for edit and resolves multi-split mode', () => {
    const mockEntry: JournalEntryView = {
      id: 'tx-123',
      date: '2026-10-01',
      description: 'Split test',
      notes: null,
      reference_no: null,
      due_date: null,
      plan_id: null,
      currency: 'IDR',
      fx_rate: 1.0,
      posted_at: 1700000000,
      postings: [
        {
          id: 'p1',
          entry_id: 'tx-123',
          account_id: 'a1',
          account_code: '1000',
          account_name: 'Cash',
          account_type: 'ASSET',
          amount: 10000,
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
          entry_id: 'tx-123',
          account_id: 'a2',
          account_code: '5000',
          account_name: 'Food',
          account_type: 'EXPENSE',
          amount: -5000,
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
          entry_id: 'tx-123',
          account_id: 'a3',
          account_code: '5001',
          account_name: 'Transport',
          account_type: 'EXPENSE',
          amount: -5000,
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

    modalState.openInspector({ entry: mockEntry, isNew: false });
    expect(modalState.inspectorOpen).toBe(true);
    expect(modalState.inspectorEntry).toBe(mockEntry);
    expect(modalState.inspectorMode).toBe('journal');
    expect(modalState.inspectorIsNew).toBe(false);
  });

  it('resets inspector state cleanly on closeInspector', () => {
    modalState.openTransfer('acc-1', 'acc-2');
    expect(modalState.inspectorOpen).toBe(true);

    modalState.closeInspector();
    expect(modalState.inspectorOpen).toBe(false);
    expect(modalState.inspectorDraft).toBeNull();
    expect(modalState.inspectorEntry).toBeNull();
    expect(modalState.inspectorInitialFrom).toBe('');
    expect(modalState.inspectorInitialTo).toBe('');
    expect(modalState.inspectorIsNew).toBe(true);
  });

  it('handles confirm modal config and close', () => {
    let confirmed = false;
    let cancelled = false;

    modalState.confirm({
      title: 'Delete',
      message: 'Sure?',
      onConfirm: () => {
        confirmed = true;
      },
      onCancel: () => {
        cancelled = true;
      },
    });

    expect(modalState.confirmConfig).not.toBeNull();
    expect(modalState.confirmConfig?.title).toBe('Delete');

    modalState.closeConfirm();
    expect(cancelled).toBe(true);
    expect(confirmed).toBe(false);
    expect(modalState.confirmConfig).toBeNull();
  });

  it('correctly reports isAnyModalOpen across various modal states', () => {
    expect(modalState.isAnyModalOpen).toBe(false);

    modalState.openNewEntry();
    expect(modalState.isAnyModalOpen).toBe(true);
    modalState.closeInspector();
    expect(modalState.isAnyModalOpen).toBe(false);

    modalState.openSettings();
    expect(modalState.isAnyModalOpen).toBe(true);
    modalState.closeSettings();
    expect(modalState.isAnyModalOpen).toBe(false);

    modalState.openCommandPalette();
    expect(modalState.isAnyModalOpen).toBe(true);
    modalState.closeCommandPalette();
    expect(modalState.isAnyModalOpen).toBe(false);

    modalState.openHealthPulse();
    expect(modalState.isAnyModalOpen).toBe(true);
    modalState.closeHealthPulse();
    expect(modalState.isAnyModalOpen).toBe(false);

    modalState.confirm({
      title: 'Test',
      message: 'Test message',
      onConfirm: () => {},
    });
    expect(modalState.isAnyModalOpen).toBe(true);
    modalState.closeConfirm();
    expect(modalState.isAnyModalOpen).toBe(false);
  });

  it('detects mode correctly based on number of postings in draft', () => {
    modalState.openInspector({
      draft: {
        id: 'd1',
        date: '2026-10-01',
        description: 'Transfer',
        notes: null,
        reference_no: null,
        due_date: null,
        plan_id: null,
        currency: 'IDR',
        fx_rate: 1.0,
        postings: [
          { account_id: 'a1', amount: 1000 },
          { account_id: 'a2', amount: -1000 },
        ],
      },
      isNew: true,
      mode: 'transfer',
    });
    expect(modalState.inspectorMode).toBe('transfer');

    modalState.closeInspector();

    modalState.openInspector({
      draft: {
        id: 'd2',
        date: '2026-10-01',
        description: 'Split',
        notes: null,
        reference_no: null,
        due_date: null,
        plan_id: null,
        currency: 'IDR',
        fx_rate: 1.0,
        postings: [
          { account_id: 'a1', amount: 2000 },
          { account_id: 'a2', amount: -1000 },
          { account_id: 'a3', amount: -1000 },
        ],
      },
      isNew: true,
      mode: 'transfer',
    });
    // Should auto-resolve to 'journal' because > 2 postings
    expect(modalState.inspectorMode).toBe('journal');
  });
});
