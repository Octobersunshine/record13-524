mod models;
mod repository;
mod handlers;
mod data;

use repository::{SettlementStore, generate_discrepancy_report, mark_discrepancies, sum_total_amount, validate_amount_consistency};
use handlers::create_router;
use data::create_sample_settlements;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "supplier_settlement_api=debug,tower_http=debug,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settlements = create_sample_settlements();

    tracing::info!("开始校验结算单金额精度与差异检测...");
    let mut has_error = false;
    for s in &settlements {
        match validate_amount_consistency(s) {
            Ok(()) => {
                tracing::info!("  {} [{}] 金额校验通过, 合计: {}", s.settlement_no, s.supplier_name, s.total_amount);
            }
            Err(e) => {
                has_error = true;
                tracing::error!("  {} [{}] 金额校验失败: {}", s.settlement_no, s.supplier_name, e);
            }
        }
    }
    if has_error {
        tracing::warn!("存在金额校验失败的单据，请检查数据准确性");
    } else {
        tracing::info!("所有单据金额校验通过，精度完整无误");
    }

    tracing::info!("开始生成差异标记...");
    let marked = mark_discrepancies(settlements.clone());
    let discrepancy_count = marked.iter().filter(|s| s.has_discrepancy == Some(true)).count();
    if discrepancy_count > 0 {
        tracing::warn!("检测到 {} 张存在金额差异的单据：", discrepancy_count);
        for s in marked.iter().filter(|s| s.has_discrepancy == Some(true)) {
            tracing::warn!("  {} [{}]", s.settlement_no, s.supplier_name);
            if let Some(details) = &s.discrepancy_details {
                for d in details {
                    tracing::warn!("    - {}", d);
                }
            }
        }
    } else {
        tracing::info!("所有单据均无金额差异，数据一致");
    }

    let all_total = sum_total_amount(&settlements);
    let valid_total = sum_total_amount(
        &settlements.iter().filter(|s| s.status.is_valid()).cloned().collect::<Vec<_>>()
    );

    let store: SettlementStore = Arc::new(Mutex::new(settlements));
    let report = generate_discrepancy_report(&store).await;

    let total = store.lock().await.len();
    let void_count = store.lock().await.iter().filter(|s| matches!(s.status, models::SettlementStatus::Void)).count();
    let valid_count = total - void_count;

    tracing::info!("已加载 {} 条供应商结算单", total);
    tracing::info!("其中有效单据: {} 条, 作废单据: {} 条", valid_count, void_count);
    tracing::info!("全部单据总金额: {} 元", all_total);
    tracing::info!("有效单据总金额: {} 元", valid_total);
    tracing::info!("差异汇总: 有差异单据 {} 张, 共 {} 处差异, 差异金额总计: {} 元",
        report.summary.settlements_with_discrepancy,
        report.summary.total_discrepancies,
        report.summary.total_discrepancy_amount
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(store).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("服务器启动成功，监听端口: 3000");
    tracing::info!("API 接口:");
    tracing::info!("  GET /health - 健康检查");
    tracing::info!("  GET /api/settlements - 获取有效结算单列表(带差异标记)");
    tracing::info!("  GET /api/settlements/all - 获取全部结算单列表(带差异标记)");
    tracing::info!("  GET /api/settlements/summary - 获取结算单统计信息");
    tracing::info!("  GET /api/settlements/supplier/:supplier_id - 按供应商查询");
    tracing::info!("  GET /api/settlements/discrepancies - 差异检测报告");

    axum::serve(listener, app).await.unwrap();
}
