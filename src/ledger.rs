use std::ops::Index;
use std::str::FromStr;
use anyhow::{bail, Result};
use serde::Serialize;
use sqlx::{Pool, Sqlite, SqliteExecutor};

use crate::db;
use crate::error::Error::{InstructionError, JournalBalanceError};

/// Ledger account type.
///
/// # Reports
///
/// The type of account used will affect how the account is displayed in reports, and may limit the
/// types of transactions that can be posted to the account.
///
/// *Profit and Loss:*
///
/// * Revenue
/// * Expense
///
/// *Balance Sheet:*
///
/// * Asset
/// * Liability
/// * Equity
#[derive(Clone, Debug, Eq, sqlx::Type, PartialEq, Serialize)]
#[sqlx(rename_all = "camelCase")]
pub enum AccountType {
    /// Asset account representing cash in hand or at bank.
    Cash,
    /// Asset account representing a current asset.
    CurrentAsset,
    /// Liability account representing a current liability.
    CurrentLiability,
    /// Equity account.
    Equity,
    /// Expense account representing direct expenses (cost of sales).
    DirectExpense,
    /// Expense account representing indirect expenses (overheads).
    IndirectExpense,
    /// Asset account representing inventory.
    Inventory,
    /// Asset account representing a non-current asset.
    NonCurrentAsset,
    /// Liability account representing a non-current liability.
    NonCurrentLiability,
    /// Revenue account representing income from sources other than normal business activity.
    OtherIncome,
    /// Asset account representing prepayments for a future accounting period.
    Prepayments,
    /// Revenue account representing income from normal business activity.
    Revenue,
    /// Virtual account used internally. Transactions cannot be posted to these accounts. Examples
    /// are Unrealised Exchange Rate Gains and Retained Profit.
    System,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAccountTypeError;

impl FromStr for AccountType {
    type Err = ParseAccountTypeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Cash" => Ok(AccountType::Cash),
            "CurrentAsset" => Ok(AccountType::CurrentAsset),
            "CurrentLiability" => Ok(AccountType::CurrentLiability),
            "DirectExpense" => Ok(AccountType::DirectExpense),
            "Equity" => Ok(AccountType::Equity),
            "IndirectExpense" => Ok(AccountType::IndirectExpense),
            "Inventory" => Ok(AccountType::Inventory),
            "NonCurrentAsset" => Ok(AccountType::NonCurrentAsset),
            "NonCurrentLiability" => Ok(AccountType::NonCurrentLiability),
            "OtherIncome" => Ok(AccountType::OtherIncome),
            "Prepayments" => Ok(AccountType::Prepayments),
            "Revenue" => Ok(AccountType::Revenue),
            _ => Err(ParseAccountTypeError),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct AccountSummary {
    pub account_id: i64,
    pub account_name: String,
    pub account_type: AccountType,
    pub account_balance: i64,
    pub timestamp: String,
}

pub async fn account_list(e: impl SqliteExecutor<'_>) -> Result<Vec<AccountSummary>> {
    let results = db::account_list_query(e).await?;
    Ok(results
        .into_iter()
        .map(|r| AccountSummary {
            account_id: r.account_id,
            account_name: r.account_name,
            account_type: r.account_type,
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

pub async fn account_detail(e: impl SqliteExecutor<'_>, account_id: i64) -> Result<AccountDetail> {
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

pub async fn account_new(e: &Pool<Sqlite>, account_id: Option<i64>, account_name: &String, account_type: &AccountType) -> Result<i64> {
    match account_id {
        Some(id) => if id < 1 || id > 990 {
            bail!(InstructionError("account id out of range (1-999)".to_string()));
        },
        None => {}
    }
    if account_name.len() > 140 {
        bail!(InstructionError("account name over 140 chars".to_string()))
    }
    Ok(db::account_new_tx(e, account_id, &account_name, &account_type).await?)
}

pub struct Journal {
    pub unstructured_narrative: String,
    pub entries: Vec<JournalEntry>,
}

pub struct JournalEntry {
    pub account: i64,
    pub amount: i64,
}

/// Post a batch of journals and return the batch ID and a [`Vec`] of journal IDs created.
///
/// The journals will be validated to ensure that they balance and that the narrative length is 140
/// characters or less.
pub async fn batch_new(e: &Pool<Sqlite>, journals: Vec<Journal>) -> Result<(i64, Vec<i64>)> {
    for journal in &journals {
        if journal.unstructured_narrative.len() > 140 {
                bail!(InstructionError("unstructured narrative over 140 chars".to_string()));
        }
        let balance: i64 = journal.entries
            .iter()
            .map(|e| e.amount)
            .sum();
        if balance != 0 {
            bail!(JournalBalanceError);
        }
    }
    Ok(db::batch_new_tx(e, journals).await?)
}

/// Wrapper function for [`batch_new`] to allow posting a batch with a single journal.
///
/// Returns the journal ID created.
pub async fn journal_new(e: &Pool<Sqlite>, unstructured_narrative: String, entries: Vec<JournalEntry>) -> Result<i64> {
    Ok(*batch_new(e, vec![Journal {
        unstructured_narrative,
        entries,
    }]).await?.1.index(0))
}
