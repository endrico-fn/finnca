-- Add reference_no and due_date to journal_entries
ALTER TABLE journal_entries ADD COLUMN reference_no TEXT;
ALTER TABLE journal_entries ADD COLUMN due_date TEXT;

CREATE INDEX IF NOT EXISTS idx_journal_entries_ref ON journal_entries(reference_no);
CREATE INDEX IF NOT EXISTS idx_journal_entries_due ON journal_entries(due_date);
