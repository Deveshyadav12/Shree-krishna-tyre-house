use axum::{extract::State, response::Json};
use serde_json::{Value, json};
use sqlx::Row;

use crate::AppState;

pub async fn billing_summary(State(state): State<AppState>) -> Json<Value> {
    let stats = sqlx::query(
        r#"SELECT
            COUNT(*) AS total_bills,
            COUNT(*) FILTER (WHERE payment_status = 'PAID') AS paid_bills,
            COUNT(*) FILTER (WHERE payment_status = 'PENDING') AS pending_bills,
            COALESCE(SUM(total) FILTER (WHERE payment_status = 'PAID'), 0)::BIGINT AS total_revenue,
            COALESCE(SUM(total) FILTER (WHERE payment_status = 'PAID' AND created_at >= CURRENT_DATE), 0)::BIGINT AS today_revenue,
            COUNT(*) FILTER (WHERE payment_status = 'PAID' AND created_at >= CURRENT_DATE) AS today_paid,
            COUNT(*) FILTER (WHERE payment_status = 'PAID' AND source = 'ONLINE') AS online_paid,
            COUNT(*) FILTER (WHERE payment_status = 'PAID' AND source = 'OFFLINE') AS offline_paid
           FROM orders"#,
    )
    .fetch_one(&state.pool)
    .await;

    let (
        total_bills,
        paid_bills,
        pending_bills,
        total_revenue,
        today_revenue,
        today_paid,
        online_paid,
        offline_paid,
    ) = match stats {
        Ok(row) => (
            row.get::<i64, _>("total_bills"),
            row.get::<i64, _>("paid_bills"),
            row.get::<i64, _>("pending_bills"),
            row.get::<i64, _>("total_revenue"),
            row.get::<i64, _>("today_revenue"),
            row.get::<i64, _>("today_paid"),
            row.get::<i64, _>("online_paid"),
            row.get::<i64, _>("offline_paid"),
        ),
        Err(e) => {
            eprintln!("Poll stats error: {e}");
            (0, 0, 0, 0, 0, 0, 0, 0)
        }
    };

    // Recent paid orders (last 5 minutes) for notifications
    let recent = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.billing_type, o.total, o.source,
                  p.payment_method
           FROM orders o
           LEFT JOIN payments p ON p.order_id = o.id
           WHERE o.payment_status = 'PAID'
             AND o.updated_at >= NOW() - INTERVAL '5 minutes'
           ORDER BY o.updated_at DESC
           LIMIT 5"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let recent_orders: Vec<Value> = recent
        .iter()
        .map(|row| {
            let order_id: String = row.get("id");
            let inv: String = row.get("invoice_number");
            let customer: String = row.get("customer_name");
            let billing_type: String = row.get("billing_type");
            let total: i64 = row.get("total");
            let source: String = row.get("source");
            let pay_method: String = row.try_get("payment_method").unwrap_or_default();
            json!({
                "order_id": order_id,
                "invoice_number": inv,
                "customer_name": customer,
                "billing_type": billing_type,
                "total": total,
                "source": source,
                "payment_method": pay_method,
            })
        })
        .collect();

    Json(json!({
        "total_bills": total_bills,
        "paid_bills": paid_bills,
        "pending_bills": pending_bills,
        "total_revenue": total_revenue,
        "today_revenue": today_revenue,
        "today_paid": today_paid,
        "online_paid": online_paid,
        "offline_paid": offline_paid,
        "recent_paid_orders": recent_orders,
    }))
}
