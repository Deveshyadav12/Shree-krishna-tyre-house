use axum::{
    extract::{Path, State},
    response::Html,
};
use sqlx::Row;

use crate::pages::layout::page;
use crate::AppState;

pub async fn order_success(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Html<String> {
    let order = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.billing_type,
                  o.total, o.payment_status,
                  p.payment_method
           FROM orders o
           LEFT JOIN payments p ON p.order_id = o.id
           WHERE o.id = $1"#,
    )
    .bind(&order_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(order) = order else {
        return Html(page(
            "Order Not Found",
            r#"<section class="page-hero"><div class="container">
                <h1>Order Not Found</h1>
                <p>This order does not exist.</p>
                <a href="/" class="btn btn-primary" style="margin-top:20px;">Go Home</a>
            </div></section>"#,
        ));
    };

    let inv_num: String = order.get("invoice_number");
    let customer: String = order.get("customer_name");
    let billing_type: String = order.get("billing_type");
    let total: i64 = order.get("total");
    let pay_status: String = order.get("payment_status");
    let pay_method: String = order.try_get("payment_method").unwrap_or_default();

    let bt_label = match billing_type.as_str() {
        "TYRE_ONLY" => "Tyre Only",
        "SERVICE_ONLY" => "Service Only",
        "TYRE_AND_SERVICE" => "Tyre + Service",
        _ => "Order",
    };

    let (status_color, status_label) = if pay_status == "PAID" {
        ("#166534", "PAID")
    } else {
        ("#92400e", "PENDING")
    };

    let content = format!(
        r#"
<section class="section" style="min-height:70vh;display:flex;align-items:center;">
    <div class="container">
        <div class="order-success-card">
            <div class="order-success-icon" style="background:#dcfce7;color:#16a34a;">&#10004;</div>
            <h1 style="margin:16px 0 8px;color:#111827;">Payment Successful!</h1>
            <p style="color:#64748b;margin-bottom:24px;">Your order has been confirmed and recorded.</p>

            <div class="order-success-details">
                <div class="order-success-row">
                    <span>Invoice Number</span>
                    <strong>{inv_num}</strong>
                </div>
                <div class="order-success-row">
                    <span>Customer</span>
                    <strong>{customer}</strong>
                </div>
                <div class="order-success-row">
                    <span>Order Type</span>
                    <strong>{bt_label}</strong>
                </div>
                <div class="order-success-row">
                    <span>Amount Paid</span>
                    <strong>&#8377;{total}</strong>
                </div>
                <div class="order-success-row">
                    <span>Payment Method</span>
                    <strong>{pay_method}</strong>
                </div>
                <div class="order-success-row">
                    <span>Status</span>
                    <strong style="color:{status_color};">{status_label}</strong>
                </div>
            </div>

            <div style="display:flex;gap:12px;justify-content:center;margin-top:28px;flex-wrap:wrap;">
                <a href="/order/invoice/{order_id}" class="btn btn-primary">View Invoice</a>
                <a href="/order/invoice/{order_id}" class="btn btn-primary"
                   onclick="window.open(this.href,'_blank');return false;">Print Invoice</a>
                <a href="/" class="btn" style="border:1px solid #e2e8f0;background:#fff;">Go Home</a>
            </div>
        </div>
    </div>
</section>
"#,
        inv_num = esc(&inv_num),
        customer = esc(&customer),
        bt_label = bt_label,
        total = total,
        pay_method = esc(&pay_method),
        status_color = status_color,
        status_label = status_label,
        order_id = esc(&order_id),
    );

    Html(page("Order Confirmed", &content))
}

pub async fn order_invoice(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Html<String> {
    let order = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.customer_phone,
                  o.billing_type, o.subtotal, o.discount, o.total,
                  o.payment_status, o.source,
                  p.payment_method, p.razorpay_payment_id,
                  TO_CHAR(o.created_at AT TIME ZONE 'UTC', 'DD Mon YYYY HH24:MI') AS created_str
           FROM orders o
           LEFT JOIN payments p ON p.order_id = o.id
           WHERE o.id = $1"#,
    )
    .bind(&order_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(order) = order else {
        return Html(page("Invoice Not Found", r#"<section class="page-hero"><div class="container"><h1>Invoice Not Found</h1></div></section>"#));
    };

    let inv_num: String = order.get("invoice_number");
    let customer: String = order.get("customer_name");
    let phone: String = order.get("customer_phone");
    let billing_type: String = order.get("billing_type");
    let subtotal: i64 = order.get("subtotal");
    let discount: i64 = order.get("discount");
    let total: i64 = order.get("total");
    let pay_status: String = order.get("payment_status");
    let pay_method: String = order.try_get("payment_method").unwrap_or_default();
    let rzp_payment_id: String = order.try_get("razorpay_payment_id").unwrap_or_default();
    let created_str: String = order.get("created_str");

    let bt_label = match billing_type.as_str() {
        "TYRE_ONLY" => "Tyre Only",
        "SERVICE_ONLY" => "Service Only",
        "TYRE_AND_SERVICE" => "Tyre + Service",
        _ => "Order",
    };

    let items = sqlx::query(
        "SELECT item_type, item_name, quantity, unit_price, total FROM order_items WHERE order_id = $1 ORDER BY id",
    )
    .bind(&order_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut item_rows = String::new();
    for (i, item) in items.iter().enumerate() {
        let itype: String = item.get("item_type");
        let iname: String = item.get("item_name");
        let qty: i32 = item.get("quantity");
        let unit: i64 = item.get("unit_price");
        let line: i64 = item.get("total");
        item_rows.push_str(&format!(
            r#"<tr>
                <td>{n}</td><td><strong>{name}</strong></td><td>{itype}</td>
                <td>{qty}</td><td>&#8377;{unit}</td><td><strong>&#8377;{line}</strong></td>
            </tr>"#,
            n = i + 1,
            name = esc(&iname),
            itype = esc(&itype),
            qty = qty,
            unit = unit,
            line = line,
        ));
    }

    let (pay_color, pay_label) = if pay_status == "PAID" {
        ("#166534", "PAID")
    } else {
        ("#92400e", "PENDING")
    };

    let rzp_row = if !rzp_payment_id.is_empty() {
        format!(
            r#"<div class="order-success-row"><span>Razorpay Payment ID</span><strong style="font-family:monospace;font-size:11px;">{}</strong></div>"#,
            esc(&rzp_payment_id)
        )
    } else {
        String::new()
    };

    let content = format!(
        r#"
<div style="max-width:860px;margin:40px auto;padding:0 20px;">
    <div style="display:flex;gap:10px;margin-bottom:20px;" class="no-print">
        <a href="/" class="btn" style="border:1px solid #e2e8f0;background:#fff;">Home</a>
        <button onclick="window.print()" class="btn btn-primary">Print Invoice</button>
    </div>

    <div class="invoice-document">
        <div class="invoice-header">
            <div style="display:flex;align-items:center;gap:14px;">
                <div class="invoice-company-mark">SK</div>
                <div class="invoice-company">
                    <strong>SHRI KRISHNA TYRE HOUSE</strong>
                    <span>Tyres &amp; Auto Care &mdash; Singhana Road, Buhana, Jhunjhunu, Rajasthan 333502</span>
                </div>
            </div>
            <div class="invoice-title">
                <span>INVOICE</span>
                <strong>{inv_num}</strong>
                <small style="display:block;margin-top:4px;color:#6b7280;">{created_str}</small>
            </div>
        </div>

        <div class="invoice-divider"></div>

        <div class="invoice-meta-grid">
            <div><span>CUSTOMER</span><strong>{customer}</strong><small>{phone}</small></div>
            <div><span>ORDER TYPE</span><strong>{bt_label}</strong></div>
            <div><span>PAYMENT</span><strong>{pay_method}</strong></div>
            <div><span>STATUS</span><strong style="color:{pay_color};">{pay_label}</strong></div>
        </div>

        <div class="invoice-table-wrap">
            <table class="invoice-table">
                <thead>
                    <tr><th>#</th><th>Description</th><th>Type</th><th>Qty</th><th>Rate</th><th>Amount</th></tr>
                </thead>
                <tbody>{item_rows}</tbody>
            </table>
        </div>

        <div class="invoice-total-section">
            <div style="text-align:right;">
                <div style="display:flex;justify-content:space-between;gap:40px;margin-bottom:6px;">
                    <span style="color:#6b7280;font-size:12px;">Subtotal</span><span>&#8377;{subtotal}</span>
                </div>
                <div style="display:flex;justify-content:space-between;gap:40px;margin-bottom:6px;">
                    <span style="color:#6b7280;font-size:12px;">Discount</span><span>&#8377;{discount}</span>
                </div>
            </div>
            <div style="text-align:right;">
                <div class="invoice-total-label">Grand Total</div>
                <div class="invoice-total-value">&#8377;{total}</div>
            </div>
        </div>

        <div class="invoice-paid-box" style="border-color:{pay_color};background:#f0fdf4;">
            <strong style="color:{pay_color};">PAYMENT STATUS: {pay_label}</strong>
            <span>Method: {pay_method} &nbsp;|&nbsp; Source: ONLINE</span>
        </div>

        {rzp_row}

        <div class="invoice-footer">
            <div>
                <strong>Thank you for choosing Shri Krishna Tyre House.</strong>
                <span>Please keep this invoice for your records.</span>
            </div>
            <div class="invoice-footer-right">
                <strong style="color:{pay_color};">{pay_label}</strong>
                <span>{inv_num}</span>
            </div>
        </div>
    </div>
</div>
"#,
        inv_num = esc(&inv_num),
        customer = esc(&customer),
        phone = esc(&phone),
        bt_label = bt_label,
        created_str = created_str,
        pay_method = esc(&pay_method),
        pay_label = pay_label,
        pay_color = pay_color,
        subtotal = subtotal,
        discount = discount,
        total = total,
        item_rows = item_rows,
        rzp_row = rzp_row,
    );

    Html(page(&format!("Invoice {}", esc(&inv_num)), &content))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
