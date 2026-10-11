-- Add currency, fx_rate, and cost_amount to postings for multi-currency double-entry support
ALTER TABLE postings ADD COLUMN currency TEXT NOT NULL DEFAULT 'IDR';
ALTER TABLE postings ADD COLUMN fx_rate INTEGER;
ALTER TABLE postings ADD COLUMN cost_amount INTEGER;
