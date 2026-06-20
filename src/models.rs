use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SettlementStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "paid")]
    Paid,
    #[serde(rename = "void")]
    Void,
}

impl SettlementStatus {
    pub fn is_valid(&self) -> bool {
        !matches!(self, SettlementStatus::Void)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DiscrepancyType {
    #[serde(rename = "item_amount_mismatch")]
    ItemAmountMismatch,
    #[serde(rename = "total_mismatch")]
    TotalMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementItem {
    pub item_id: Uuid,
    pub product_name: String,
    pub quantity: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_discrepancy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "rust_decimal::serde::str_option")]
    pub calculated_amount: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "rust_decimal::serde::str_option")]
    pub discrepancy_amount: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discrepancy_message: Option<String>,
}

impl SettlementItem {
    pub fn calculate_amount(&self) -> Decimal {
        Decimal::from(self.quantity) * self.unit_price
    }

    pub fn is_amount_valid(&self) -> bool {
        let calculated = self.calculate_amount();
        (calculated - self.amount).abs() < Decimal::new(1, 6)
    }

    pub fn mark_discrepancy(&mut self) {
        let calculated = self.calculate_amount();
        let discrepancy = calculated - self.amount;
        self.has_discrepancy = Some(!self.is_amount_valid());
        self.calculated_amount = Some(calculated);
        self.discrepancy_amount = Some(discrepancy);
        if !self.is_amount_valid() {
            self.discrepancy_message = Some(format!(
                "金额不一致：单价({}) × 数量({}) = 计算金额({})，记录金额({})，差异({})",
                self.unit_price, self.quantity, calculated, self.amount, discrepancy
            ));
        } else {
            self.discrepancy_message = None;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierSettlement {
    pub id: Uuid,
    pub settlement_no: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: SettlementStatus,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    pub settlement_date: String,
    pub due_date: String,
    pub items: Vec<SettlementItem>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_discrepancy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "rust_decimal::serde::str_option")]
    pub calculated_total: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "rust_decimal::serde::str_option")]
    pub total_discrepancy: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discrepancy_details: Option<Vec<String>>,
}

impl SupplierSettlement {
    pub fn calculate_total_amount(&self) -> Decimal {
        self.items
            .iter()
            .fold(Decimal::zero(), |acc, item| acc + item.amount)
    }

    pub fn calculate_total_from_calculated(&self) -> Decimal {
        self.items
            .iter()
            .fold(Decimal::zero(), |acc, item| acc + item.calculate_amount())
    }

    pub fn is_total_valid(&self) -> bool {
        let calculated = self.calculate_total_amount();
        (calculated - self.total_amount).abs() < Decimal::new(1, 6)
    }

    pub fn is_amount_consistent(&self) -> bool {
        self.items.iter().all(|item| item.is_amount_valid())
            && self.is_total_valid()
    }

    pub fn mark_all_discrepancies(&mut self) {
        let mut details: Vec<String> = Vec::new();

        for (idx, item) in self.items.iter_mut().enumerate() {
            item.mark_discrepancy();
            if item.has_discrepancy == Some(true) {
                details.push(format!(
                    "明细[{}]{}: {}",
                    idx + 1,
                    item.product_name,
                    item.discrepancy_message.clone().unwrap_or_default()
                ));
            }
        }

        let calculated_total = self.calculate_total_from_calculated();
        let total_discrepancy = calculated_total - self.total_amount;
        let has_total_issue = total_discrepancy.abs() >= Decimal::new(1, 6);

        if has_total_issue {
            details.push(format!(
                "合计金额不一致：明细计算合计({})，单据合计({})，差异({})",
                calculated_total, self.total_amount, total_discrepancy
            ));
        }

        let has_item_issue = self.items.iter().any(|i| i.has_discrepancy == Some(true));
        self.has_discrepancy = Some(has_item_issue || has_total_issue);
        self.calculated_total = Some(calculated_total);
        self.total_discrepancy = Some(total_discrepancy);
        if details.is_empty() {
            self.discrepancy_details = None;
        } else {
            self.discrepancy_details = Some(details);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SettlementListResponse {
    pub total: usize,
    pub valid_count: usize,
    pub void_count: usize,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_amount: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub valid_total_amount: Decimal,
    pub settlements: Vec<SupplierSettlement>,
}

#[derive(Debug, Serialize)]
pub struct DiscrepancySummary {
    pub total_settlements: usize,
    pub settlements_with_discrepancy: usize,
    pub settlements_ok: usize,
    pub total_item_discrepancies: usize,
    pub total_discrepancies: usize,
    #[serde(with = "rust_decimal::serde::str")]
    pub total_discrepancy_amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct DiscrepancyReportResponse {
    pub summary: DiscrepancySummary,
    pub discrepancy_settlements: Vec<SupplierSettlement>,
}
