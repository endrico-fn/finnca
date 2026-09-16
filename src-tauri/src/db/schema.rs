use crate::shared::AppError;
use rusqlite::Connection;

const MIGRATION_0001: &str = include_str!("migrations/0001_init.sql");
const MIGRATION_0002: &str = include_str!("migrations/0002_add_budgets.sql");
const MIGRATION_0003: &str = include_str!("migrations/0003_add_audit_log.sql");
const MIGRATION_0004: &str = include_str!("migrations/0004_add_plans.sql");
const MIGRATION_0005: &str = include_str!("migrations/0005_fix_fx_rate_scale.sql");

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
        assert_eq!(version, 5);

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

        run_migrations(&mut conn).expect("re-run migrations — must be idempotent");
        let version2: u32 = conn
            .query_row("PRAGMA user_version;", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version2, 5);
    }
}
