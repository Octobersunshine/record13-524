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
    pub unit_price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierSettlement {
    pub id: Uuid,
    pub settlement_no: String,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: SettlementStatus,
    pub total_amount: f64,
    pub settlement_date: String,
    pub due_date: String,
    pub items: Vec<SettlementItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SettlementListResponse {
    pub total: usize,
    pub valid_count: usize,
    pub void_count: usize,
    pub settlements: Vec<SupplierSettlement>,
}
