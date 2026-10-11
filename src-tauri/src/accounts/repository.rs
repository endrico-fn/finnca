use super::models::{Account, AccountType};
use crate::shared::AppError;
use rusqlite::{params, Connection, Row};
use std::collections::HashMap;
use std::str::FromStr;

fn map_row_to_account(row: &Row<'_>) -> rusqlite::Result<Account> {
    let type_str: String = row.get(3)?;
    let account_type = AccountType::from_str(&type_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    let placeholder_int: i64 = row.get(6)?;
    let hidden_int: i64 = row.get(7)?;

    Ok(Account {
        id: row.get(0)?,
        code: row.get(1)?,
        name: row.get(2)?,
        account_type,
        parent_id: row.get(4)?,
        currency: row.get(5)?,
        placeholder: placeholder_int != 0,
        hidden: hidden_int != 0,
        color: row.get(8)?,
        note: row.get(9)?,
        description: row.get(10)?,
        interest_rate: row.get(11)?,
        created_at: row.get(12)?,
    })
}

pub fn insert(conn: &Connection, acc: &Account) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO accounts (
            id, code, name, type, parent_id, currency, placeholder, hidden,
            color, note, description, interest_rate, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13);",
        params![
            acc.id,
            acc.code,
            acc.name,
            acc.account_type.as_str(),
            acc.parent_id,
            acc.currency,
            if acc.placeholder { 1 } else { 0 },
            if acc.hidden { 1 } else { 0 },
            acc.color,
            acc.note,
            acc.description,
            acc.interest_rate,
            acc.created_at,
        ],
    )?;
    Ok(())
}

pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Account>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, code, name, type, parent_id, currency, placeholder, hidden,
                color, note, description, interest_rate, created_at
         FROM accounts WHERE id = ?1;",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_row_to_account(row)?))
    } else {
        Ok(None)
    }
}

pub fn get_by_code(conn: &Connection, code: &str) -> Result<Option<Account>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, code, name, type, parent_id, currency, placeholder, hidden,
                color, note, description, interest_rate, created_at
         FROM accounts WHERE code = ?1;",
    )?;

    let mut rows = stmt.query(params![code])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_row_to_account(row)?))
    } else {
        Ok(None)
    }
}

pub fn list_all(conn: &Connection) -> Result<Vec<Account>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, code, name, type, parent_id, currency, placeholder, hidden,
                color, note, description, interest_rate, created_at
         FROM accounts ORDER BY code ASC;",
    )?;

    let rows = stmt.query_map([], map_row_to_account)?;
    let mut accounts = Vec::new();
    for acc_res in rows {
        accounts.push(acc_res?);
    }
    Ok(accounts)
}

pub fn update(conn: &Connection, acc: &Account) -> Result<(), AppError> {
    conn.execute(
        "UPDATE accounts SET
            code = ?2,
            name = ?3,
            type = ?4,
            parent_id = ?5,
            currency = ?6,
            placeholder = ?7,
            hidden = ?8,
            color = ?9,
            note = ?10,
            description = ?11,
            interest_rate = ?12
         WHERE id = ?1;",
        params![
            acc.id,
            acc.code,
            acc.name,
            acc.account_type.as_str(),
            acc.parent_id,
            acc.currency,
            if acc.placeholder { 1 } else { 0 },
            if acc.hidden { 1 } else { 0 },
            acc.color,
            acc.note,
            acc.description,
            acc.interest_rate,
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM accounts WHERE id = ?1;", params![id])?;
    Ok(())
}

pub fn has_children(conn: &Connection, account_id: &str) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM accounts WHERE parent_id = ?1;",
        params![account_id],
        |r| r.get(0),
    )?;
    Ok(count > 0)
}

pub fn has_postings(conn: &Connection, account_id: &str) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM postings WHERE account_id = ?1;",
        params![account_id],
        |r| r.get(0),
    )?;
    Ok(count > 0)
}

pub fn get_all_direct_balances(conn: &Connection) -> Result<HashMap<String, i64>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT account_id, COALESCE(SUM(amount), 0) FROM postings GROUP BY account_id;",
    )?;

    let rows = stmt.query_map([], |row| {
        let account_id: String = row.get(0)?;
        let balance: i64 = row.get(1)?;
        Ok((account_id, balance))
    })?;

    let mut map = HashMap::new();
    for item in rows {
        let (k, v) = item?;
        map.insert(k, v);
    }
    Ok(map)
}
