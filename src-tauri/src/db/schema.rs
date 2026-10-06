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

    let migrations: &[(u32, &str)] = &[
        (1, MIGRATION_0001),
        (2, MIGRATION_0002),
        (3, MIGRATION_0003),
        (4, MIGRATION_0004),
        (5, MIGRATION_0005),
        (6, MIGRATION_0006),
        (7, MIGRATION_0007),
        (8, MIGRATION_0008),
        (9, MIGRATION_0009),
        (10, MIGRATION_0010),
        (11, MIGRATION_0011),
        (12, MIGRATION_0012),
    ];

    for &(version, sql) in migrations {
        if current_version < version {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
        }
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

    #[test]
    fn test_migration_failure_rolls_back_atomically() {
        let mut conn = Connection::open_in_memory().expect("open in-memory db");
        {
            let tx = conn.transaction().expect("begin tx");
            tx.execute_batch("CREATE TABLE test_table (id TEXT PRIMARY KEY);")
                .expect("create table");
            tx.pragma_update(None, "user_version", 1)
                .expect("update version");
            tx.commit().expect("commit");
        }

        let initial_version: u32 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(initial_version, 1);

        // Attempt a failing migration within a transaction
        let res = (|| -> Result<(), rusqlite::Error> {
            let tx = conn.transaction()?;
            tx.execute_batch("INSERT INTO test_table (id) VALUES ('row1');")?;
            tx.execute_batch("THIS IS INVALID SQL SYNTAX AND MUST FAIL;")?;
            tx.pragma_update(None, "user_version", 2)?;
            tx.commit()?;
            Ok(())
        })();

        assert!(res.is_err(), "Migration with invalid SQL syntax must fail");

        // Verify atomicity: user_version must still be 1, and 'row1' must be rolled back
        let rolled_back_version: u32 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rolled_back_version, 1);

        let row_count: i64 = conn
            .query_row("SELECT count(*) FROM test_table;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(row_count, 0, "Partial writes must be rolled back on failure");
    }
}
