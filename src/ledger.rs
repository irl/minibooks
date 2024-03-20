use sqlx::{Pool, Sqlite, SqliteExecutor};

use crate::db;
use crate::db::AccountType;

pub struct AccountSummary {
    pub account_id: i64,
    pub account_name: String,
    pub account_balance: i64,
    pub timestamp: String,
}

pub async fn account_list(e: impl SqliteExecutor<'_>) -> Result<Vec<AccountSummary>, sqlx::Error> {
    let results = db::account_list_query(e).await?;
    Ok(results
        .into_iter()
        .map(|r| AccountSummary {
            account_id: r.account_id,
            account_name: r.account_name,
            account_balance: r.balance,
            timestamp: r.timestamp
        })
        .collect())
}

pub struct AccountDetail {
    pub account_id: i64,
    pub account_name: String,
    pub account_type: AccountType,
    pub total_credits: i64,
    pub total_debits: i64,
    pub balance: i64,
    pub timestamp: String,
}

pub async fn account_detail(e: impl SqliteExecutor<'_>, account_id: i64) -> Result<AccountDetail, sqlx::Error> {
    let result = db::account_detail_query(e, account_id).await?;
    Ok(AccountDetail {
        account_id,
        account_name: result.account_name,
        account_type: result.account_type,
        total_credits: result.total_credits,
        total_debits: result.total_debits,
        balance: result.total_debits - result.total_credits,
        timestamp: result.timestamp,
    })
}

pub async fn account_new(e: &Pool<Sqlite>, account_id: Option<i64>, account_name: &String, account_type: &AccountType) -> Result<i64, sqlx::Error> {
    /* TODO: Check account ID is in range */
    /* TODO: Check length of the account name is < 140 */
    Ok(db::account_new(e, account_id, &account_name, &account_type).await?)
}

pub struct JournalEntry {
    pub account: i64,
    pub amount: i64,
}

pub async fn journal_new(e: &Pool<Sqlite>, unstructured_narrative: &Option<String>, entries: Vec<JournalEntry>) -> Result<(), sqlx::Error> {
    /* TODO: Check length of narrative is < 140 */
    /* TODO: Ensure the entries balance */
    let unstructured_narrative = unstructured_narrative.clone().unwrap_or("".to_string());
    Ok(db::journal_new(e, unstructured_narrative, entries).await?)
}
