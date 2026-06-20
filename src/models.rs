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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementItem {
    pub item_id: Uuid,
    pub product_name: String,
    pub quantity: u32,
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_price: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
}

impl SettlementItem {
    pub fn calculate_amount(&self) -> Decimal {
        Decimal::from(self.quantity) * self.unit_price
    }

    pub fn is_amount_valid(&self) -> bool {
        let calculated = self.calculate_amount();
        (calculated - self.amount).abs() < Decimal::new(1, 6)
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
}

impl SupplierSettlement {
    pub fn calculate_total_amount(&self) -> Decimal {
        self.items
            .iter()
            .fold(Decimal::zero(), |acc, item| acc + item.amount)
    }

    pub fn is_total_valid(&self) -> bool {
        let calculated = self.calculate_total_amount();
        (calculated - self.total_amount).abs() < Decimal::new(1, 6)
    }

    pub fn is_amount_consistent(&self) -> bool {
        self.items.iter().all(|item| item.is_amount_valid())
            && self.is_total_valid()
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
