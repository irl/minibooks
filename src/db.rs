use std::str::FromStr;
use sqlx::{Error, Pool, Sqlite, SqliteExecutor};
use crate::ledger::JournalEntry;

use crate::settings;

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(rename_all = "camelCase")]
pub enum AccountType {
    Cash,
    CurrentAsset,
    CurrentLiability,
    Equity,
    Expense,
    NonCurrentAsset,
    NonCurrentLiability,
    OtherIncome,
    Revenue,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAccountTypeError;

impl FromStr for AccountType {
    type Err = ParseAccountTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Cash" => Ok(AccountType::Cash),
            "CurrentAsset" => Ok(AccountType::CurrentAsset),
            "CurrentLiability" => Ok(AccountType::CurrentLiability),
            "Equity" => Ok(AccountType::Equity),
            "Expense" => Ok(AccountType::Expense),
            "NonCurrentAsset" => Ok(AccountType::NonCurrentAsset),
            "NonCurrentLiability" => Ok(AccountType::NonCurrentLiability),
            "OtherIncome" => Ok(AccountType::OtherIncome),
            "Revenue" => Ok(AccountType::Revenue),
            _ => Err(ParseAccountTypeError),
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct AccountDetailResult {
    pub account_name: String,
    pub account_type: AccountType,
    pub total_debits: i64,
    pub total_credits: i64,
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

#[derive(sqlx::FromRow, Debug)]
pub struct AccountListResult {
    pub account_id: i64,
    pub account_name: String,
    pub balance: i64,
    pub timestamp: String,
}

pub async fn account_list_query(e: impl SqliteExecutor<'_>) -> Result<Vec<AccountListResult>, Error> {
    sqlx::query_as!(AccountListResult,
        r#"SELECT account.id AS "account_id!", name AS "account_name!",
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

pub async fn account_new(db: &Pool<Sqlite>, account_id: Option<i64>, account_name: &String,
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

pub async fn journal_new(db: &Pool<Sqlite>, unstructured_narrative: String, entries: Vec<JournalEntry>) -> Result<(), sqlx::Error> {
    let mut transaction = db.begin().await?;
    sqlx::query!("INSERT INTO batch (date) VALUES (DATE('NOW'));").execute(&mut *transaction).await?;
    let batch_id = sqlx::query!("SELECT last_insert_rowid() AS batch_id;")
        .fetch_one(&mut *transaction)
        .await?
        .batch_id;
    sqlx::query!("INSERT INTO journal (batch_id, unstructured_narrative) VALUES (?, ?)",
        batch_id, unstructured_narrative)
        .execute(&mut *transaction).await?;
    dbg!(batch_id);
    let journal_id = sqlx::query!("SELECT last_insert_rowid() AS journal_id;")
        .fetch_one(&mut *transaction)
        .await?
        .journal_id;
    for entry in entries {
        sqlx::query!("INSERT INTO entry (journal_id, account_id, amount) VALUES (?, ?, ?);",
            journal_id, entry.account, entry.amount)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok({})
}
