//! Usage query Tauri commands

use crate::api::usage::{get_account_usage, refresh_all_usage, warmup_account as send_warmup};
use crate::auth::{get_account, load_accounts};
use crate::types::{UsageInfo, WarmupSummary};

/// Get usage info for a specific account
#[tauri::command]
pub async fn get_usage(account_id: String) -> Result<UsageInfo, String> {
    let account = get_account(&account_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Account not found: {account_id}"))?;

    get_account_usage(&account).await.map_err(|e| e.to_string())
}

/// Refresh usage info for all accounts
#[tauri::command]
pub async fn refresh_all_accounts_usage() -> Result<Vec<UsageInfo>, String> {
    let store = load_accounts().map_err(|e| e.to_string())?;
    Ok(refresh_all_usage(&store.accounts).await)
}

/// Send a minimal warm-up request for one account
#[tauri::command]
pub async fn warmup_account(account_id: String) -> Result<(), String> {
    let account = get_account(&account_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Account not found: {account_id}"))?;

    send_warmup(&account).await.map_err(|e| e.to_string())
}

/// Send minimal warm-up requests for all accounts
#[tauri::command]
pub async fn warmup_all_accounts() -> Result<WarmupSummary, String> {
    let store = load_accounts().map_err(|e| e.to_string())?;
    let total_accounts = store.accounts.len();

    let futures = store.accounts.iter().map(|account| async move {
        let account_id = account.id.clone();
        let result = send_warmup(account).await;
        (account_id, result)
    });

    let results = futures::future::join_all(futures).await;
    let failed_account_ids: Vec<String> = results
        .into_iter()
        .filter_map(|(account_id, result)| {
            if result.is_err() {
                Some(account_id)
            } else {
                None
            }
        })
        .collect();

    let warmed_accounts = total_accounts.saturating_sub(failed_account_ids.len());
    Ok(WarmupSummary {
        total_accounts,
        warmed_accounts,
        failed_account_ids,
    })
}
