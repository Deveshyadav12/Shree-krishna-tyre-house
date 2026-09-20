use axum::{
    extract::{Path, State},
    response::Html,
};
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn billing_type_label(bt: &str) -> &'static str {
    match bt {
        "TYRE_ONLY" => "Tyre Only",
        "SERVICE_ONLY" => "Service Only",
        "TYRE_AND_SERVICE" => "Tyre + Service",
        _ => "Unknown",
    }
}

pub async fn invoices(State(state): State<AppState>) -> Html<String> {
    let stats = sqlx::query(
        r#"SELECT
            COUNT(*) AS total,
            COALESCE(SUM(total) FILTER (WHERE payment_status = 'PAID'), 0)::BIGINT AS paid_amount,
            COUNT(*) FILTER (WHERE payment_status = 'PAID') AS paid_count,
            COUNT(*) FILTER (WHERE payment_status = 'PENDING') AS pending_count
           FROM orders"#,
    )
    .fetch_one(&state.pool)
    .await;

    let (total_inv, paid_amount, paid_count, pending_count) = match stats {
        Ok(row) => (
            row.get::<i64, _>("total"),
            row.get::<i64, _>("paid_amount"),
            row.get::<i64, _>("paid_count"),
            row.get::<i64, _>("pending_count"),
        ),
        Err(e) => {
            eprintln!("Invoices stats error: {e}");
            (0, 0, 0, 0)
        }
    };

    let rows = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.billing_type,
                  o.total, o.payment_status,
                  p.payment_method,
                  TO_CHAR(o.created_at AT TIME ZONE 'UTC', 'DD Mon YYYY') AS date_str
           FROM orders o
           LEFT JOIN payments p ON p.order_id = o.id
           ORDER BY o.created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut table_rows = String::new();
    for row in &rows {
        let order_id: String = row.get("id");
        let inv_num: String = row.get("invoice_number");
        let customer: String = row.get("customer_name");
        let billing_type: String = row.get("billing_type");
        let total: i64 = row.get("total");
        let pay_status: String = row.get("payment_status");
        let pay_method: String = row.try_get("payment_method").unwrap_or_default();
        let date_str: String = row.get("date_str");

        let status_class = match pay_status.as_str() {
            "PAID" => "dash-badge-green",
            "CANCELLED" => "dash-badge-red",
            _ => "dash-badge-yellow",
        };

        table_rows.push_str(&format!(
            r#"<tr>
                <td><a href="/admin/billing/order/{oid}" class="dash-id-link">{inv}</a></td>
                <td><strong>{customer}</strong></td>
                <td>{bt_label}</td>
                <td><strong>&#8377;{total}</strong></td>
                <td>{pay_method}</td>
                <td><span class="dash-badge {status_class}">{pay_status}</span></td>
                <td>{date_str}</td>
                <td>
                    <div style="display:flex;gap:6px;">
                        <a href="/admin/billing/order/{oid}" class="dash-view-link">View</a>
                        <a href="/admin/billing/order/{oid}" class="dash-view-link"
                           onclick="window.open(this.href,'_blank');return false;">Print</a>
                    </div>
                </td>
            </tr>"#,
            oid = esc(&order_id),
            inv = esc(&inv_num),
            customer = esc(&customer),
            bt_label = billing_type_label(&billing_type),
            total = total,
            pay_method = esc(&pay_method),
            status_class = status_class,
            pay_status = esc(&pay_status),
            date_str = date_str,
        ));
    }

    let empty = if rows.is_empty() {
        r#"<tr><td colspan="8" class="dash-empty-cell">No invoices yet.</td></tr>"#
    } else {
        ""
    };

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">FINANCE</div>
        <h1>Invoices</h1>
        <p>All bills and invoices from offline billing.</p>
    </div>
    <a href="/admin/billing/new" class="admin-primary-button">+ New Bill</a>
</div>

<div class="invoice-stat-grid" style="margin-bottom:20px;">
    <div class="invoice-stat-card">
        <span>Total Invoices</span>
        <strong>{total_inv}</strong>
    </div>
    <div class="invoice-stat-card">
        <span>Paid Invoices</span>
        <strong>{paid_count}</strong>
    </div>
    <div class="invoice-stat-card">
        <span>Pending</span>
        <strong>{pending_count}</strong>
    </div>
    <div class="invoice-stat-card">
        <span>Total Revenue</span>
        <strong>&#8377;{paid_amount}</strong>
    </div>
</div>

<div class="admin-panel">
    <div class="admin-panel-header">
        <div><h2>Invoice List</h2></div>
        <a href="/admin/billing" class="admin-secondary-button">Billing</a>
    </div>
    <div class="dash-table-wrap">
        <table class="dash-table">
            <thead>
                <tr>
                    <th>INVOICE</th>
                    <th>CUSTOMER</th>
                    <th>TYPE</th>
                    <th>AMOUNT</th>
                    <th>METHOD</th>
                    <th>STATUS</th>
                    <th>DATE</th>
                    <th>ACTIONS</th>
                </tr>
            </thead>
            <tbody>{table_rows}{empty}</tbody>
        </table>
    </div>
</div>
"#,
        total_inv = total_inv,
        paid_count = paid_count,
        pending_count = pending_count,
        paid_amount = paid_amount,
        table_rows = table_rows,
        empty = empty,
    );

    Html(admin_page("Invoices", &content))
}

pub async fn invoice_detail(
    State(state): State<AppState>,
    Path(invoice_id): Path<String>,
) -> Html<String> {
    // invoice_id may be an order_id or an invoice_number — try both
    let order = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.customer_phone,
                  o.billing_type, o.subtotal, o.discount, o.total,
                  o.payment_status, o.source, o.notes,
                  p.payment_method,
                  TO_CHAR(o.created_at AT TIME ZONE 'UTC', 'DD Mon YYYY HH24:MI') AS created_str
           FROM orders o
           LEFT JOIN payments p ON p.order_id = o.id
           WHERE o.id = $1 OR o.invoice_number = $1"#,
    )
    .bind(&invoice_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(order) = order else {
        return Html(admin_page(
            "Invoices",
            r#"<div class="admin-page-header"><div><h1>Invoice Not Found</h1></div>
               <a href="/admin/invoices" class="admin-secondary-button">Back to Invoices</a></div>"#,
        ));
    };

    let order_id: String = order.get("id");
    let inv_num: String = order.get("invoice_number");
    let customer: String = order.get("customer_name");
    let phone: String = order.get("customer_phone");
    let billing_type: String = order.get("billing_type");
    let subtotal: i64 = order.get("subtotal");
    let discount: i64 = order.get("discount");
    let total: i64 = order.get("total");
    let pay_status: String = order.get("payment_status");
    let source: String = order.get("source");
    let notes: String = order.get("notes");
    let pay_method: String = order.try_get("payment_method").unwrap_or_default();
    let created_str: String = order.get("created_str");

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
                <td>{n}</td>
                <td><strong>{name}</strong></td>
                <td>{itype}</td>
                <td>{qty}</td>
                <td>&#8377;{unit}</td>
                <td><strong>&#8377;{line}</strong></td>
            </tr>"#,
            n = i + 1,
            name = esc(&iname),
            itype = esc(&itype),
            qty = qty,
            unit = unit,
            line = line,
        ));
    }

    let (pay_label, pay_color, pay_bg) = match pay_status.as_str() {
        "PAID" => ("PAID", "#166534", "#f0fdf4"),
        "CANCELLED" => ("CANCELLED", "#991b1b", "#fef2f2"),
        _ => ("CASH PENDING", "#92400e", "#fffbeb"),
    };

    let content = format!(
        r#"
<div class="invoice-page">
    <div class="invoice-toolbar no-print" style="display:flex;gap:10px;margin-bottom:20px;">
        <a href="/admin/invoices" class="admin-secondary-button">Back</a>
        <button type="button" class="invoice-print-button" onclick="window.print()">Print Invoice</button>
    </div>

    <div class="invoice-document">
        <div class="invoice-header">
            <div>
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
            <div>
                <span>CUSTOMER</span>
                <strong>{customer}</strong>
                <small>{phone}</small>
            </div>
            <div>
                <span>BILLING TYPE</span>
                <strong>{bt_label}</strong>
            </div>
            <div>
                <span>PAYMENT METHOD</span>
                <strong>{pay_method}</strong>
            </div>
            <div>
                <span>PAYMENT STATUS</span>
                <strong style="color:{pay_color};">{pay_label}</strong>
            </div>
        </div>

        <div class="invoice-table-wrap">
            <table class="invoice-table">
                <thead>
                    <tr>
                        <th>#</th><th>Description</th><th>Type</th>
                        <th>Qty</th><th>Rate</th><th>Amount</th>
                    </tr>
                </thead>
                <tbody>{item_rows}</tbody>
            </table>
        </div>

        <div class="invoice-total-section">
            <div style="text-align:right;">
                <div style="display:flex;justify-content:space-between;gap:40px;margin-bottom:6px;">
                    <span style="color:#6b7280;font-size:12px;">Subtotal</span>
                    <span>&#8377;{subtotal}</span>
                </div>
                <div style="display:flex;justify-content:space-between;gap:40px;margin-bottom:6px;">
                    <span style="color:#6b7280;font-size:12px;">Discount</span>
                    <span>&#8377;{discount}</span>
                </div>
            </div>
            <div style="text-align:right;">
                <div class="invoice-total-label">Grand Total</div>
                <div class="invoice-total-value">&#8377;{total}</div>
            </div>
        </div>

        <div class="invoice-paid-box" style="border-color:{pay_color};background:{pay_bg};">
            <strong style="color:{pay_color};">PAYMENT STATUS: {pay_label}</strong>
            <span>Method: {pay_method} &nbsp;|&nbsp; Source: {source}</span>
        </div>

        <div class="invoice-footer">
            <div>
                <strong>Thank you for choosing Shri Krishna Tyre House.</strong>
                <span>Please keep this invoice for your records.</span>
                {notes_html}
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
        bt_label = billing_type_label(&billing_type),
        created_str = created_str,
        pay_method = esc(&pay_method),
        pay_label = pay_label,
        pay_color = pay_color,
        pay_bg = pay_bg,
        subtotal = subtotal,
        discount = discount,
        total = total,
        source = esc(&source),
        item_rows = item_rows,
        notes_html = if notes.is_empty() {
            String::new()
        } else {
            format!(
                "<span style='display:block;margin-top:6px;color:#6b7280;'>Note: {}</span>",
                esc(&notes)
            )
        },
    );

    Html(admin_page("Invoices", &content))
}
