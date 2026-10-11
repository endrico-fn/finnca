CREATE INDEX IF NOT EXISTS idx_postings_account_entry ON postings(account_id, entry_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_plan ON journal_entries(plan_id);
CREATE INDEX IF NOT EXISTS idx_journal_entries_due ON journal_entries(due_date);
CREATE INDEX IF NOT EXISTS idx_journal_entries_date_posted ON journal_entries(date, posted_at);
