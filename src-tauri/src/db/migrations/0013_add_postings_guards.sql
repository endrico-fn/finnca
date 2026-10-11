-- Defense-in-depth: reject zero-amount postings at DB layer.
-- Per-entry balance SUM(amount)==0 stays enforced in Rust
-- (ledger::validation) because multi-row atomicity cannot be
-- expressed per-row in SQLite triggers.
CREATE TRIGGER IF NOT EXISTS trg_postings_no_zero_amount_insert
BEFORE INSERT ON postings
WHEN NEW.amount = 0
BEGIN
    SELECT RAISE(ABORT, 'Posting amount 0 is not allowed');
END;

CREATE TRIGGER IF NOT EXISTS trg_postings_no_zero_amount_update
BEFORE UPDATE OF amount ON postings
WHEN NEW.amount = 0
BEGIN
    SELECT RAISE(ABORT, 'Posting amount 0 is not allowed');
END;
