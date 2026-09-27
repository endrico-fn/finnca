CREATE TABLE IF NOT EXISTS reconcile_rules (
    id TEXT PRIMARY KEY NOT NULL,
    pattern TEXT NOT NULL,
    is_regex INTEGER NOT NULL DEFAULT 0,
    match_type TEXT NOT NULL DEFAULT 'ANY' CHECK(match_type IN ('ANY', 'INFLOW', 'OUTFLOW')),
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    priority INTEGER NOT NULL DEFAULT 0,
    description_override TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reconcile_rules_priority ON reconcile_rules(priority DESC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_reconcile_rules_account ON reconcile_rules(account_id);
