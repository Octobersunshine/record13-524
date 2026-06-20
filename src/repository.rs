use crate::models::{
    DiscrepancyReportResponse, DiscrepancySummary, SettlementListResponse, SettlementStatus,
    SupplierSettlement,
};
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

pub fn mark_discrepancies(mut settlements: Vec<SupplierSettlement>) -> Vec<SupplierSettlement> {
    for s in settlements.iter_mut() {
        s.mark_all_discrepancies();
    }
    settlements
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
    let valid_marked = mark_discrepancies(valid_settlements);

    SettlementListResponse {
        total: all.len(),
        valid_count,
        void_count,
        total_amount,
        valid_total_amount,
        settlements: valid_marked,
    }
}

pub async fn filter_by_supplier(
    store: &SettlementStore,
    supplier_id: &str,
) -> Vec<SupplierSettlement> {
    let filtered = load_valid_settlements(store)
        .await
        .into_iter()
        .filter(|s| s.supplier_id == supplier_id)
        .collect();
    mark_discrepancies(filtered)
}

pub async fn get_valid_settlements_marked(store: &SettlementStore) -> Vec<SupplierSettlement> {
    let valid = load_valid_settlements(store).await;
    mark_discrepancies(valid)
}

pub async fn get_all_settlements_marked(store: &SettlementStore) -> Vec<SupplierSettlement> {
    let all = load_settlements(store).await;
    mark_discrepancies(all)
}

pub async fn generate_discrepancy_report(store: &SettlementStore) -> DiscrepancyReportResponse {
    let all_marked = get_all_settlements_marked(store).await;

    let total_settlements = all_marked.len();
    let discrepancy_settlements: Vec<SupplierSettlement> = all_marked
        .into_iter()
        .filter(|s| s.has_discrepancy == Some(true))
        .collect();
    let settlements_with_discrepancy = discrepancy_settlements.len();
    let settlements_ok = total_settlements - settlements_with_discrepancy;

    let mut total_item_discrepancies = 0usize;
    let mut total_discrepancy_amount = Decimal::zero();

    for s in &discrepancy_settlements {
        for item in &s.items {
            if item.has_discrepancy == Some(true) {
                total_item_discrepancies += 1;
            }
        }
        if let Some(disc) = s.total_discrepancy {
            total_discrepancy_amount += disc.abs();
        }
    }

    let total_discrepancies = total_item_discrepancies
        + discrepancy_settlements
            .iter()
            .filter(|s| s.total_discrepancy.map_or(false, |d| d.abs() >= Decimal::new(1, 6)))
            .count();

    DiscrepancyReportResponse {
        summary: DiscrepancySummary {
            total_settlements,
            settlements_with_discrepancy,
            settlements_ok,
            total_item_discrepancies,
            total_discrepancies,
            total_discrepancy_amount,
        },
        discrepancy_settlements,
    }
}
