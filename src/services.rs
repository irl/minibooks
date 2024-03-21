use std::str::FromStr;

use actix_web::{get, HttpResponse, post, Responder, web};
use actix_web::web::Data;
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{AppState, ledger};
use crate::ledger::{AccountSummary, AccountType, JournalEntry};
use crate::settings::get_settings_str;

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
    let item = item.clone();
    let journal_entries: Vec<JournalEntry> = item.entries
        .into_iter()
        .map(|e| JournalEntry {
            account: e.account,
            amount: e.amount,
        })
        .collect();
    ledger::journal_new(&state.db, item.unstructured_narrative.unwrap_or("".to_string()), journal_entries).await.unwrap();
    web::Json(JournalCreateResponse {})
}

fn filter_accounts_list<F>(accounts: &Vec<AccountSummary>, f: F) -> Vec<AccountSummary>
    where F: Fn(&AccountSummary) -> bool {
    accounts
        .clone()
        .into_iter()
        .filter(f)
        .collect()
}

fn sum_filter_accounts_list<F>(accounts: &Vec<AccountSummary>, f: F) -> i64
    where F: Fn(&AccountSummary) -> bool
{
    accounts
        .clone()
        .into_iter()
        .filter(f)
        .map(|a| a.account_balance)
        .sum()
}

#[get("/report/balance")]
pub async fn report_balance_sheet(state: Data<AppState>) -> impl Responder {
    let mut ctx = Context::new();
    let entity_name = get_settings_str(&state.db, "entityName").await.unwrap();
    ctx.insert("entity_name", &entity_name);
    let accounts = ledger::account_list(&state.db).await.unwrap();

    let cash: Vec<AccountSummary> = filter_accounts_list(&accounts, |a| a.account_type == AccountType::Cash);
    ctx.insert("cash", &cash);

    let total_cash = sum_filter_accounts_list(
        &accounts, |a| a.account_type == AccountType::Cash);
    ctx.insert("total_cash", &total_cash);

    let current_assets: Vec<AccountSummary> = filter_accounts_list(&accounts, |a| {
        match a.account_type {
            AccountType::CurrentAsset => true,
            AccountType::Inventory => true,
            AccountType::Prepayments => true,
            _ => false,
        }
    });
    ctx.insert("current_assets", &current_assets);

    let total_current_assets = sum_filter_accounts_list(&accounts, |a| match a.account_type {
            AccountType::Cash => true,
            AccountType::CurrentAsset => true,
            AccountType::Inventory => true,
            AccountType::Prepayments => true,
            _ => false,
        });
    ctx.insert("total_current_assets", &total_current_assets);

    let current_liabilities: Vec<AccountSummary> = filter_accounts_list(
        &accounts, |a| a.account_type == AccountType::CurrentLiability);
    ctx.insert("current_liabilities", &current_liabilities);

    let total_current_liabilities: i64 = sum_filter_accounts_list(
        &accounts, |a| a.account_type == AccountType::CurrentLiability);
    ctx.insert("total_current_liabilities", &total_current_liabilities);

    let net_assets = total_current_assets + total_current_liabilities;
    ctx.insert("net_assets", &net_assets);

    let rendered = state.tmpl.render("balance_sheet.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}
