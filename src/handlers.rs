use crate::repository::{filter_by_supplier, get_settlements_with_stats, load_valid_settlements, SettlementStore};
use crate::models::{SettlementListResponse, SupplierSettlement};
use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};

pub async fn get_valid_settlements(
    State(store): State<SettlementStore>,
) -> Json<Vec<SupplierSettlement>> {
    let result = load_valid_settlements(&store).await;
    Json(result)
}

pub async fn get_settlements_summary(
    State(store): State<SettlementStore>,
) -> Json<SettlementListResponse> {
    let result = get_settlements_with_stats(&store).await;
    Json(result)
}

pub async fn get_settlements_by_supplier(
    State(store): State<SettlementStore>,
    Path(supplier_id): Path<String>,
) -> Json<Vec<SupplierSettlement>> {
    let result = filter_by_supplier(&store, &supplier_id).await;
    Json(result)
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

pub fn create_router(store: SettlementStore) -> axum::Router {
    axum::Router::new()
        .route("/health", axum::routing::get(health_check))
        .route("/api/settlements", axum::routing::get(get_valid_settlements))
        .route("/api/settlements/summary", axum::routing::get(get_settlements_summary))
        .route("/api/settlements/supplier/:supplier_id", axum::routing::get(get_settlements_by_supplier))
        .with_state(store)
}
