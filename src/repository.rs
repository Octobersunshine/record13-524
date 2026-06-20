use crate::models::{SettlementStatus, SettlementListResponse, SupplierSettlement};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type SettlementStore = Arc<Mutex<Vec<SupplierSettlement>>>;

pub async fn load_settlements(store: &SettlementStore) -> Vec<SupplierSettlement> {
    store.lock().await.clone()
}

pub async fn load_valid_settlements(store: &SettlementStore) -> Vec<SupplierSettlement> {
    store
        .lock()
        .await
        .iter()
        .filter(|s| s.status.is_valid())
        .cloned()
        .collect()
}

pub async fn get_settlements_with_stats(store: &SettlementStore) -> SettlementListResponse {
    let all = load_settlements(store).await;
    let valid_count = all.iter().filter(|s| s.status.is_valid()).count();
    let void_count = all.iter().filter(|s| matches!(s.status, SettlementStatus::Void)).count();

    SettlementListResponse {
        total: all.len(),
        valid_count,
        void_count,
        settlements: all.into_iter().filter(|s| s.status.is_valid()).collect(),
    }
}

pub async fn filter_by_supplier(
    store: &SettlementStore,
    supplier_id: &str,
) -> Vec<SupplierSettlement> {
    load_valid_settlements(store)
        .await
        .into_iter()
        .filter(|s| s.supplier_id == supplier_id)
        .collect()
}
