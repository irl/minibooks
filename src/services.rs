use std::str::FromStr;
use serde::{Deserialize, Serialize};
use actix_web::{get, post, web};
use actix_web::web::Data;
use crate::{AppState, ledger};
use crate::db::{AccountType};
use crate::ledger::JournalEntry;

#[derive(Serialize)]
struct AccountDetailResponse {
    account_id: String,
    account_name: String,
    account_type: String,
    total_credits: String,
    total_debits: String,
    balance: String,
    timestamp: String,
}

#[get("/account/{account_id}")]
pub async fn account_detail(state: Data<AppState>, path: web::Path<(i64, )>) -> web::Json<AccountDetailResponse> {
    let account_id = path.into_inner().0;
    let result = ledger::account_detail(&state.db, account_id).await.unwrap();
    web::Json(AccountDetailResponse {
        account_name: result.account_name,
        account_id: format!("{account_id:<08}"),
        account_type: format!("{:?}", result.account_type),
        total_debits: result.total_debits.to_string(),
        total_credits: result.total_credits.to_string(),
        balance: result.balance.to_string(),
        timestamp: result.timestamp.to_string(),
    })
}

#[derive(Serialize)]
struct AccountListResponse {
    accounts: Vec<AccountListAccountResponse>,
    timestamp: String,
}

#[derive(Serialize)]
struct AccountListAccountResponse {
    id: String,
    name: String,
    balance: String,
}

#[get("/account/list")]
pub async fn account_list(state: Data<AppState>) -> web::Json<AccountListResponse> {
    let results = ledger::account_list(&state.db).await.unwrap();
    let mut accounts: Vec<AccountListAccountResponse> = Vec::new();
    let mut timestamp: String = "".to_string();
    for result in results {
        accounts.push(AccountListAccountResponse {
            id: format!("{:<08}", result.account_id),
            name: result.account_name,
            balance: result.account_balance.to_string(),
        });
        timestamp = result.timestamp;
    }
    web::Json(AccountListResponse {
        accounts,
        timestamp,
    })
}

#[derive(Clone, Deserialize)]
struct AccountCreateData {
    account_id: Option<i64>,
    account_name: String,
    account_type: String,
}

#[derive(Serialize)]
struct AccountCreateResponse {
    pub account_id: String,
    pub account_name: String,
    pub account_type: String,
}

#[post("/account/new")]
pub async fn account_new(state: Data<AppState>, item: web::Json<AccountCreateData>) -> web::Json<AccountCreateResponse> {
    let account_id = item.account_id;
    let account_name = item.account_name.clone();
    let account_type = AccountType::from_str(item.account_type.as_str()).unwrap();
    let created_account_id = ledger::account_new(&state.db, account_id, &account_name, &account_type).await.unwrap();
    web::Json(AccountCreateResponse {
        account_id: format!("{created_account_id:<08}"),
        account_name: account_name.to_string(),
        account_type: format!("{account_type:?}"),
    })
}

#[derive(Clone, Deserialize)]
pub struct JournalCreateData {
    unstructured_narrative: Option<String>,
    entries: Vec<JournalCreateEntryData>,
}

#[derive(Clone, Deserialize)]
pub struct JournalCreateEntryData {
    account: i64,
    amount: i64,
}

#[derive(Serialize)]
struct JournalCreateResponse;

#[post("/journal/new")]
pub async fn journal_new(state: Data<AppState>, item: web::Json<JournalCreateData>) -> web::Json<JournalCreateResponse> {
    let journal_entries: Vec<JournalEntry> = item.clone().entries
        .into_iter()
        .map(|e| JournalEntry {
            account: e.account,
            amount: e.amount,
        })
        .collect();
    ledger::journal_new(&state.db, &item.unstructured_narrative, journal_entries).await.unwrap();
    web::Json(JournalCreateResponse {})
}
