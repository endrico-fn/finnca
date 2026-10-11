UPDATE journal_entries SET fx_rate = 16000 WHERE fx_rate = 1000000 OR fx_rate IS NULL OR fx_rate <= 0;
