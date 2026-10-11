CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('RECEIVABLE', 'PAYABLE', 'RECURRING')),
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK(status IN ('ACTIVE', 'COMPLETED', 'OVERDUE', 'ARCHIVED')),
    total_amount INTEGER NOT NULL,
    installment_amount INTEGER NOT NULL,
    frequency TEXT NOT NULL CHECK(frequency IN ('DAILY', 'WEEKLY', 'MONTHLY')),
    start_date TEXT NOT NULL,
    due_date TEXT,
    day_of_month INTEGER,
    from_account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    to_account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    notes TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_plans_status ON plans(status);
CREATE INDEX IF NOT EXISTS idx_plans_from_account ON plans(from_account_id);
CREATE INDEX IF NOT EXISTS idx_plans_to_account ON plans(to_account_id);

ALTER TABLE journal_entries ADD COLUMN plan_id TEXT REFERENCES plans(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_journal_entries_plan ON journal_entries(plan_id);
