use crate::models::{SettlementStatus, SettlementListResponse, SupplierSettlement};
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
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

pub fn sum_total_amount(settlements: &[SupplierSettlement]) -> Decimal {
    settlements
        .iter()
        .fold(Decimal::zero(), |acc, s| acc + s.total_amount)
}

pub fn validate_amount_consistency(settlement: &SupplierSettlement) -> Result<(), String> {
    if settlement.is_amount_consistent() {
        return Ok(());
    }

    for (idx, item) in settlement.items.iter().enumerate() {
        if !item.is_amount_valid() {
            let calculated = item.calculate_amount();
            return Err(format!(
                "明细项[{}]金额不一致: 单价={} × 数量={} = 计算金额={}, 记录金额={}",
                idx + 1, item.unit_price, item.quantity, calculated, item.amount
            ));
        }
    }

    if !settlement.is_total_valid() {
        let calculated = settlement.calculate_total_amount();
        return Err(format!(
            "合计金额不一致: 明细合计={}, 单据合计={}",
            calculated, settlement.total_amount
        ));
    }

    Ok(())
}

pub async fn get_settlements_with_stats(store: &SettlementStore) -> SettlementListResponse {
    let all = load_settlements(store).await;
    let valid_count = all.iter().filter(|s| s.status.is_valid()).count();
    let void_count = all.iter().filter(|s| matches!(s.status, SettlementStatus::Void)).count();
    let total_amount = sum_total_amount(&all);
    let valid_settlements: Vec<SupplierSettlement> = all.iter()
        .filter(|s| s.status.is_valid())
        .cloned()
        .collect();
    let valid_total_amount = sum_total_amount(&valid_settlements);

    SettlementListResponse {
        total: all.len(),
        valid_count,
        void_count,
        total_amount,
        valid_total_amount,
        settlements: valid_settlements,
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
