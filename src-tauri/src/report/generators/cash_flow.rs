use crate::ledger::currency::{convert_minor_units, normalize_fx_rate, DEFAULT_FX_RATE};
use crate::report::dto::{CashFlowActivityRow, CashFlowReport};
use crate::shared::AppError;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

struct AccountMeta {
    name: String,
    acc_type: String,
    is_liquid: bool,
}

fn is_liquid_cash_account(code: &str, name: &str, acc_type: &str) -> bool {
    if acc_type != "ASSET" {
        return false;
    }
    let name_lower = name.to_lowercase();
    code.starts_with("11")
        || code.starts_with("101")
        || code.starts_with("102")
        || name_lower.contains("cash")
        || name_lower.contains("kas")
        || name_lower.contains("bank")
        || name_lower.contains("wallet")
        || name_lower.contains("dompet")
        || name_lower.contains("rekening")
        || name_lower.contains("checking")
        || name_lower.contains("tabungan")
}

pub fn generate(
    conn: &Connection,
    from_date: Option<&str>,
    to_date: Option<&str>,
) -> Result<CashFlowReport, AppError> {
    let spot_fx: i64 = conn
        .query_row(
            "SELECT fx_rate FROM journal_entries WHERE fx_rate > 0 AND fx_rate != 1000000 ORDER BY date DESC, posted_at DESC LIMIT 1;",
            [],
            |r| r.get(0),
        )
        .unwrap_or(DEFAULT_FX_RATE);

    let mut acct_stmt = conn.prepare(
        "SELECT id, code, name, type
         FROM accounts
         WHERE placeholder = 0;",
    )?;

    let mut accounts: HashMap<String, AccountMeta> = HashMap::new();
    let mut liquid_account_ids: HashSet<String> = HashSet::new();

    let acct_rows = acct_stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    for r in acct_rows {
        let (id, code, name, acc_type) = r?;
        let is_liquid = is_liquid_cash_account(&code, &name, &acc_type);
        if is_liquid {
            liquid_account_ids.insert(id.clone());
        }
        accounts.insert(
            id,
            AccountMeta {
                name,
                acc_type,
                is_liquid,
            },
        );
    }

    let starting_cash: i64 = if let Some(from) = from_date {
        let mut sum_stmt = conn.prepare(
            "SELECT p.account_id, COALESCE(p.currency, a.currency, 'IDR'), p.amount, COALESCE(p.fx_rate, je.fx_rate, 16000), p.cost_amount
             FROM postings p
             JOIN accounts a ON a.id = p.account_id
             JOIN journal_entries je ON je.id = p.entry_id
             WHERE je.date < ?1;",
        )?;
        let rows = sum_stmt.query_map(rusqlite::params![from], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        })?;

        let mut total: i64 = 0;
        for r in rows {
            let (account_id, curr, amount, entry_fx, cost_amount) = r?;
            if liquid_account_ids.contains(&account_id) {
                let in_idr = if let Some(cost) = cost_amount {
                    amount.signum() * cost.abs()
                } else if curr != "IDR" {
                    convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, spot_fx))
                } else {
                    amount
                };
                total += in_idr;
            }
        }
        total
    } else {
        0
    };

    let mut stmt = conn.prepare(
        "SELECT je.id, je.date, je.description, p.account_id, COALESCE(p.currency, a.currency, 'IDR'), p.amount, COALESCE(p.fx_rate, je.fx_rate, 16000), p.cost_amount
         FROM postings p
         JOIN accounts a ON a.id = p.account_id
         JOIN journal_entries je ON je.id = p.entry_id
         WHERE (?1 IS NULL OR je.date >= ?1)
           AND (?2 IS NULL OR je.date <= ?2)
         ORDER BY je.date ASC, je.id ASC;",
    )?;

    let mut current_entry_id: Option<String> = None;
    let mut current_date = String::new();
    let mut current_desc = String::new();
    let mut entry_postings: Vec<(String, i64)> = Vec::new();

    let mut operating_cash_flow: i64 = 0;
    let mut investing_cash_flow: i64 = 0;
    let mut financing_cash_flow: i64 = 0;

    let mut operating_rows: Vec<CashFlowActivityRow> = Vec::new();
    let mut investing_rows: Vec<CashFlowActivityRow> = Vec::new();
    let mut financing_rows: Vec<CashFlowActivityRow> = Vec::new();

    let rows = stmt.query_map(rusqlite::params![from_date, to_date], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, Option<i64>>(7)?,
        ))
    })?;

    let mut process_entry = |date: &str, desc: &str, postings: &[(String, i64)]| {
        let mut liquid_net: i64 = 0;
        let mut non_liquid_postings: Vec<(&AccountMeta, i64)> = Vec::new();

        for (acc_id, amount) in postings {
            if let Some(meta) = accounts.get(acc_id) {
                if meta.is_liquid {
                    liquid_net += *amount;
                } else {
                    non_liquid_postings.push((meta, *amount));
                }
            }
        }

        if liquid_net == 0 {
            return;
        }

        let mut has_asset_contra = false;
        let mut has_financing_contra = false;
        let mut contra_names: Vec<String> = Vec::new();

        for (meta, _) in &non_liquid_postings {
            contra_names.push(meta.name.clone());
            match meta.acc_type.as_str() {
                "ASSET" => has_asset_contra = true,
                "LIABILITY" | "EQUITY" => has_financing_contra = true,
                _ => {}
            }
        }

        let contra_summary = if contra_names.is_empty() {
            desc.to_string()
        } else {
            contra_names.join(", ")
        };

        let row_description = if desc.trim().is_empty() {
            contra_summary.clone()
        } else {
            desc.to_string()
        };

        if has_asset_contra {
            investing_cash_flow += liquid_net;
            investing_rows.push(CashFlowActivityRow {
                date: date.to_string(),
                category: if liquid_net >= 0 {
                    "INVESTING_INFLOW".into()
                } else {
                    "INVESTING_OUTFLOW".into()
                },
                description: row_description,
                account_name: contra_summary,
                amount: liquid_net,
            });
        } else if has_financing_contra {
            financing_cash_flow += liquid_net;
            financing_rows.push(CashFlowActivityRow {
                date: date.to_string(),
                category: if liquid_net >= 0 {
                    "FINANCING_INFLOW".into()
                } else {
                    "FINANCING_OUTFLOW".into()
                },
                description: row_description,
                account_name: contra_summary,
                amount: liquid_net,
            });
        } else {
            operating_cash_flow += liquid_net;
            operating_rows.push(CashFlowActivityRow {
                date: date.to_string(),
                category: if liquid_net >= 0 {
                    "OPERATING_INCOME".into()
                } else {
                    "OPERATING_EXPENSE".into()
                },
                description: row_description,
                account_name: contra_summary,
                amount: liquid_net,
            });
        }
    };

    for r in rows {
        let (entry_id, date, desc, account_id, curr, amount, entry_fx, cost_amount) = r?;
        let in_idr = if let Some(cost) = cost_amount {
            amount.signum() * cost.abs()
        } else if curr != "IDR" {
            convert_minor_units(amount, &curr, "IDR", normalize_fx_rate(entry_fx, spot_fx))
        } else {
            amount
        };

        if let Some(ref cur_id) = current_entry_id {
            if cur_id != &entry_id {
                process_entry(&current_date, &current_desc, &entry_postings);
                entry_postings.clear();
                current_entry_id = Some(entry_id);
                current_date = date;
                current_desc = desc;
            }
        } else {
            current_entry_id = Some(entry_id);
            current_date = date;
            current_desc = desc;
        }
        entry_postings.push((account_id, in_idr));
    }

    if current_entry_id.is_some() && !entry_postings.is_empty() {
        process_entry(&current_date, &current_desc, &entry_postings);
    }

    let net_cash_change = operating_cash_flow + investing_cash_flow + financing_cash_flow;
    let ending_cash = starting_cash + net_cash_change;

    Ok(CashFlowReport {
        from_date: from_date.map(ToString::to_string),
        to_date: to_date.map(ToString::to_string),
        starting_cash,
        operating_cash_flow,
        investing_cash_flow,
        financing_cash_flow,
        net_cash_change,
        ending_cash,
        operating_rows,
        investing_rows,
        financing_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE accounts (
                id TEXT PRIMARY KEY,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'IDR',
                placeholder INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE journal_entries (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                description TEXT NOT NULL,
                currency TEXT NOT NULL DEFAULT 'IDR',
                fx_rate INTEGER NOT NULL DEFAULT 16000,
                posted_at INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE postings (
                id TEXT PRIMARY KEY,
                entry_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                amount INTEGER NOT NULL,
                currency TEXT NOT NULL DEFAULT 'IDR',
                fx_rate INTEGER,
                cost_amount INTEGER
            );",
        )
        .unwrap();

        conn.execute_batch(
            "INSERT INTO accounts (id, code, name, type, placeholder) VALUES
                ('acc_cash', '1110', 'Kas Tunai', 'ASSET', 0),
                ('acc_bank', '1120', 'Bank Operasional', 'ASSET', 0),
                ('acc_equip', '1310', 'Peralatan Kantor', 'ASSET', 0),
                ('acc_loan', '2110', 'Pinjaman Bank', 'LIABILITY', 0),
                ('acc_salary', '4110', 'Pendapatan Gaji', 'INCOME', 0),
                ('acc_food', '5110', 'Makanan & Minuman', 'EXPENSE', 0);",
        )
        .unwrap();

        conn
    }

    #[test]
    fn test_cash_flow_empty() {
        let conn = setup_test_db();
        let report = generate(&conn, None, None).unwrap();
        assert_eq!(report.starting_cash, 0);
        assert_eq!(report.net_cash_change, 0);
        assert_eq!(report.ending_cash, 0);
    }

    #[test]
    fn test_cash_flow_double_entry_activities() {
        let conn = setup_test_db();

        conn.execute_batch(
            "INSERT INTO journal_entries (id, date, description) VALUES
                ('je_1', '2026-01-01', 'Modal Pinjaman Awal'),
                ('je_2', '2026-01-02', 'Gaji Masuk'),
                ('je_3', '2026-01-05', 'Beli Makan'),
                ('je_4', '2026-01-10', 'Beli Laptop Kantor'),
                ('je_5', '2026-01-15', 'Tarik Tunai dari Bank');

             INSERT INTO postings (id, entry_id, account_id, amount) VALUES
                ('p1_1', 'je_1', 'acc_bank', 10000000),
                ('p1_2', 'je_1', 'acc_loan', -10000000),

                ('p2_1', 'je_2', 'acc_bank', 5000000),
                ('p2_2', 'je_2', 'acc_salary', -5000000),

                ('p3_1', 'je_3', 'acc_cash', -200000),
                ('p3_2', 'je_3', 'acc_food', 200000),

                ('p4_1', 'je_4', 'acc_bank', -4000000),
                ('p4_2', 'je_4', 'acc_equip', 4000000),

                ('p5_1', 'je_5', 'acc_bank', -1000000),
                ('p5_2', 'je_5', 'acc_cash', 1000000);",
        )
        .unwrap();

        let report = generate(&conn, Some("2026-01-01"), Some("2026-01-31")).unwrap();

        assert_eq!(report.financing_cash_flow, 10000000);
        assert_eq!(report.operating_cash_flow, 4800000);
        assert_eq!(report.investing_cash_flow, -4000000);
        assert_eq!(report.net_cash_change, 10800000);
        assert_eq!(report.ending_cash, 10800000);

        assert_eq!(report.financing_rows.len(), 1);
        assert_eq!(report.operating_rows.len(), 2);
        assert_eq!(report.investing_rows.len(), 1);
    }
}
