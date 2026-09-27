use crate::shared::AppError;
use rusqlite::Connection;

const MIGRATION_0001: &str = include_str!("migrations/0001_init.sql");
const MIGRATION_0002: &str = include_str!("migrations/0002_add_budgets.sql");
const MIGRATION_0003: &str = include_str!("migrations/0003_add_audit_log.sql");
const MIGRATION_0004: &str = include_str!("migrations/0004_add_plans.sql");
const MIGRATION_0005: &str = include_str!("migrations/0005_fix_fx_rate_scale.sql");
const MIGRATION_0006: &str = include_str!("migrations/0006_add_journal_entry_ref_and_due_date.sql");
const MIGRATION_0007: &str =
    include_str!("migrations/0007_add_plan_recurring_execution_fields.sql");
const MIGRATION_0008: &str = include_str!("migrations/0008_add_vault_closing_date.sql");
const MIGRATION_0009: &str = include_str!("migrations/0009_add_perf_indexes.sql");
const MIGRATION_0010: &str = include_str!("migrations/0010_add_audit_log_hash_chain.sql");
const MIGRATION_0011: &str = include_str!("migrations/0011_add_reconcile_rules.sql");
const MIGRATION_0012: &str = include_str!("migrations/0012_add_currency_and_cost_to_postings.sql");

pub fn run_migrations(conn: &mut Connection) -> Result<(), AppError> {
    let current_version: u32 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;

    if current_version < 1 {
        conn.execute_batch(MIGRATION_0001)?;
        conn.pragma_update(None, "user_version", 1)?;
    }

    if current_version < 2 {
        conn.execute_batch(MIGRATION_0002)?;
        conn.pragma_update(None, "user_version", 2)?;
    }

    if current_version < 3 {
        conn.execute_batch(MIGRATION_0003)?;
        conn.pragma_update(None, "user_version", 3)?;
    }

    if current_version < 4 {
        conn.execute_batch(MIGRATION_0004)?;
        conn.pragma_update(None, "user_version", 4)?;
    }

    if current_version < 5 {
        conn.execute_batch(MIGRATION_0005)?;
        conn.pragma_update(None, "user_version", 5)?;
    }

    if current_version < 6 {
        conn.execute_batch(MIGRATION_0006)?;
        conn.pragma_update(None, "user_version", 6)?;
    }

    if current_version < 7 {
        conn.execute_batch(MIGRATION_0007)?;
        conn.pragma_update(None, "user_version", 7)?;
    }

    if current_version < 8 {
        conn.execute_batch(MIGRATION_0008)?;
        conn.pragma_update(None, "user_version", 8)?;
    }

    if current_version < 9 {
        conn.execute_batch(MIGRATION_0009)?;
        conn.pragma_update(None, "user_version", 9)?;
    }

    if current_version < 10 {
        conn.execute_batch(MIGRATION_0010)?;
        conn.pragma_update(None, "user_version", 10)?;
    }

    if current_version < 11 {
        conn.execute_batch(MIGRATION_0011)?;
        conn.pragma_update(None, "user_version", 11)?;
    }

    if current_version < 12 {
        conn.execute_batch(MIGRATION_0012)?;
        conn.pragma_update(None, "user_version", 12)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_apply_cleanly() {
        let mut conn = Connection::open_in_memory().expect("open in-memory db");
        run_migrations(&mut conn).expect("run migrations");

        let version: u32 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 12);

        let accounts_count: i64 = conn
            .query_row("SELECT count(*) FROM accounts;", [], |r| r.get(0))
            .expect("query accounts table");
        assert_eq!(accounts_count, 0);

        let budgets_count: i64 = conn
            .query_row("SELECT count(*) FROM budgets;", [], |r| r.get(0))
            .expect("query budgets table");
        assert_eq!(budgets_count, 0);

        let audit_count: i64 = conn
            .query_row("SELECT count(*) FROM audit_log;", [], |r| r.get(0))
            .expect("query audit_log table");
        assert_eq!(audit_count, 0);

        let plans_count: i64 = conn
            .query_row("SELECT count(*) FROM plans;", [], |r| r.get(0))
            .expect("query plans table");
        assert_eq!(plans_count, 0);

        let closing_count: i64 = conn
            .query_row("SELECT count(*) FROM book_closing;", [], |r| r.get(0))
            .expect("query book_closing table");
        assert_eq!(closing_count, 1);

        let rules_count: i64 = conn
            .query_row("SELECT count(*) FROM reconcile_rules;", [], |r| r.get(0))
            .expect("query reconcile_rules table");
        assert_eq!(rules_count, 0);

        run_migrations(&mut conn).expect("re-run migrations — must be idempotent");
        let version2: u32 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version2, 12);
    }
}
