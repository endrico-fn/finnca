-- Accounts (Shallow Tree)
CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('ASSET', 'LIABILITY', 'EQUITY', 'INCOME', 'EXPENSE')),
    parent_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    currency TEXT NOT NULL DEFAULT 'IDR',
    placeholder INTEGER NOT NULL DEFAULT 0,
    hidden INTEGER NOT NULL DEFAULT 0,
    color TEXT,
    note TEXT,
    description TEXT,
    interest_rate REAL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_accounts_parent ON accounts(parent_id);
CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(type);
CREATE INDEX IF NOT EXISTS idx_accounts_code ON accounts(code);

-- Journal Entries (Transaction Header)
CREATE TABLE IF NOT EXISTS journal_entries (
    id TEXT PRIMARY KEY NOT NULL,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    notes TEXT,
    currency TEXT NOT NULL DEFAULT 'IDR',
    fx_rate INTEGER NOT NULL DEFAULT 1000000,
    posted_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_journal_entries_date ON journal_entries(date);

-- Postings / Splits (Double-Entry Legs)
CREATE TABLE IF NOT EXISTS postings (
    id TEXT PRIMARY KEY NOT NULL,
    entry_id TEXT NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    amount INTEGER NOT NULL,
    memo TEXT,
    action TEXT,
    reconciled TEXT NOT NULL DEFAULT 'n' CHECK(reconciled IN ('y', 'n', 'c')),
    reconciled_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_postings_entry ON postings(entry_id);
CREATE INDEX IF NOT EXISTS idx_postings_account ON postings(account_id);

-- Orthogonal Dimensions (Tags, Payees, Projects)
CREATE TABLE IF NOT EXISTS dimensions (
    id TEXT PRIMARY KEY NOT NULL,
    type TEXT NOT NULL,
    name TEXT NOT NULL,
    UNIQUE(type, name)
);

CREATE TABLE IF NOT EXISTS posting_dimensions (
    posting_id TEXT NOT NULL REFERENCES postings(id) ON DELETE CASCADE,
    dimension_id TEXT NOT NULL REFERENCES dimensions(id) ON DELETE RESTRICT,
    PRIMARY KEY (posting_id, dimension_id)
);
