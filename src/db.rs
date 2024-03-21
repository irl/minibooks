//! Functions for interacting with the backing SQLite database.
//!
//! Only SQLite databases are supported.
//!
//! # Data Types
//!
//! * All currency amounts are stored in the database as signed 8-byte integers.
//! * All dates and times are stored in the database as TEXT values compatible with SQLite's
//!   DATE and TIME functions.
//!
//! # Notes
//!
//! * Many functions in here should work on any of [`sqlx::Pool`], [`sqlx::Connection`] and
//!   [`sqlx::Transaction`] but there's not yet a generic way to allow that with sqlx.
//!   The relevant issue to watch is at:
//!   <https://github.com/launchbadge/sqlx/issues/419>.

use serde::Serialize;
use sqlx::{Error, Pool, Sqlite, SqliteConnection, SqliteExecutor};

use crate::ledger::{AccountType, Journal};
use crate::settings;


/// The result of an [`account_detail_query`].
#[derive(sqlx::FromRow, Debug)]
pub struct AccountDetailResult {
    /// The account name.
    pub account_name: String,
    /// The type of account.
    pub account_type: AccountType,
    /// The total value of all debits made to the account.
    pub total_debits: i64,
    /// The total value of all credits made to the account.
    pub total_credits: i64,
    /// The timestamp, from SQLite, when the result was generated.
    pub timestamp: String,
}

pub async fn account_detail_query(e: impl SqliteExecutor<'_>, account_id: i64) -> Result<AccountDetailResult, Error> {
    sqlx::query_as!(AccountDetailResult,
        r#"SELECT account.name AS "account_name!", account.type AS "account_type!: AccountType",
        IFNULL(SUM(positive), 0) AS "total_debits!: i64",
        IFNULL(SUM(negative), 0) * -1 AS "total_credits!: i64",
        CURRENT_TIMESTAMP AS "timestamp!"
        FROM account
        LEFT JOIN (SELECT account_id,
            max(amount, 0) AS positive, min(amount, 0) AS negative
            FROM entry) e ON account.id = e.account_id
            WHERE account.id=?;"#, account_id)
        .fetch_one(e)
        .await
}

/// The inner result of [`account_list_query`].
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct AccountSummaryResult {
    /// The account ID.
    pub account_id: i64,
    /// The account name.
    pub account_name: String,
    /// The account type.
    pub account_type: AccountType,
    /// The balance of the account.
    /// Positive values mean the account has a debit balance.
    /// Negative values mean the account has a credit balance.
    pub balance: i64,
    /// The timestamp, from SQLite, when the result was generated.
    pub timestamp: String,
}

pub async fn account_list_query(e: impl SqliteExecutor<'_>) -> Result<Vec<AccountSummaryResult>, Error> {
    sqlx::query_as!(AccountSummaryResult,
        r#"SELECT account.id AS "account_id!", name AS "account_name!", type AS "account_type!: AccountType",
        IFNULL(b.balance, 0) AS "balance!: i64", CURRENT_TIMESTAMP AS "timestamp!"
        FROM account
            LEFT JOIN (
                SELECT account_id, SUM(amount) AS balance
                FROM entry
                GROUP BY account_id
            ) b ON account.id = b.account_id;"#)
        .fetch_all(e)
        .await
}

pub async fn account_new_tx(db: &Pool<Sqlite>, account_id: Option<i64>, account_name: &String,
                            account_type: &AccountType) -> Result<i64, Error> {
    let mut transaction = db.begin().await?;
    let next_id_setting_name = format!("nextAccount{account_type:?}");
    let this_account_id = match account_id {
        Some(id) => id,
        None => {
            settings::get_settings_int(&mut *transaction, next_id_setting_name.as_str()).await.unwrap()
        }
    };
    sqlx::query!(
        "INSERT INTO account (id, name, type) VALUES (?, ?, ?)",
        this_account_id, account_name, account_type
    )
        .execute(&mut *transaction)
        .await?;
    if account_id.is_none() {
        settings::set_settings_int(&mut *transaction, next_id_setting_name.as_str(), this_account_id + 1).await;
    }
    transaction.commit().await?;
    Ok(this_account_id)
}

pub async fn batch_new(e: &mut SqliteConnection) -> Result<i64, Error> {
    sqlx::query!("INSERT INTO batch (date) VALUES (DATE('NOW'));").execute(&mut *e).await?;
    Ok(sqlx::query!(r#"SELECT last_insert_rowid() AS "batch_id: i64";"#)
        .fetch_one(&mut *e)
        .await?
        .batch_id)
}

pub async fn journal_new(e: &mut SqliteConnection, batch_id: i64, unstructured_narrative: String) -> Result<i64, Error> {
    sqlx::query!("INSERT INTO journal (batch_id, unstructured_narrative) VALUES (?, ?)",
        batch_id, unstructured_narrative)
        .execute(&mut *e).await?;
    Ok(sqlx::query!(r#"SELECT last_insert_rowid() AS "journal_id: i64";"#)
        .fetch_one(&mut *e)
        .await?
        .journal_id)
}

pub async fn journal_entry_new(e: &mut SqliteConnection, journal_id: i64, account_id: i64, amount: i64) -> Result<i64, Error> {
    sqlx::query!("INSERT INTO entry (journal_id, account_id, amount) VALUES (?, ?, ?);",
            journal_id, account_id, amount)
        .execute(&mut *e)
        .await?;
    Ok(sqlx::query!(r#"SELECT last_insert_rowid() AS "entry_id: i64";"#)
        .fetch_one(&mut *e)
        .await?
        .entry_id)
}

pub async fn batch_new_tx(db: &Pool<Sqlite>, journals: Vec<Journal>) -> Result<(i64, Vec<i64>), Error> {
    let mut transaction = db.begin().await?;
    let batch_id = batch_new(&mut *transaction).await?;
    let mut journal_ids: Vec<i64> = Vec::new();
    for journal in journals {
        let journal_id = journal_new(&mut *transaction, batch_id, journal.unstructured_narrative).await?;
        for entry in journal.entries {
            journal_entry_new(&mut *transaction, journal_id, entry.account, entry.amount).await?;
        }
        journal_ids.push(journal_id);
    }
    transaction.commit().await?;
    Ok((batch_id, journal_ids))
}
