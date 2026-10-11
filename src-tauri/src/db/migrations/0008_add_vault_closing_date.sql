-- Accounting Lock Date / Closing Books
CREATE TABLE IF NOT EXISTS book_closing (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    closing_date TEXT,
    updated_at INTEGER NOT NULL
);

INSERT OR IGNORE INTO book_closing (id, closing_date, updated_at) VALUES (1, NULL, 0);
