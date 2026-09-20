use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
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

pub async fn billing(State(state): State<AppState>) -> Html<String> {
    // Stats from orders table
    let stats = sqlx::query(
        r#"SELECT
            COUNT(*) AS total_bills,
            COUNT(*) FILTER (WHERE payment_status = 'PAID') AS paid_bills,
            COUNT(*) FILTER (WHERE payment_status = 'PENDING') AS pending_bills,
            COALESCE(SUM(total) FILTER (WHERE payment_status = 'PAID'), 0)::BIGINT AS total_revenue,
            COALESCE(SUM(total) FILTER (WHERE payment_status = 'PENDING'), 0)::BIGINT AS pending_amount
           FROM orders"#,
    )
    .fetch_one(&state.pool)
    .await;

    let (total_bills, paid_bills, pending_bills, total_revenue, pending_amount) = match stats {
        Ok(row) => (
            row.get::<i64, _>("total_bills"),
            row.get::<i64, _>("paid_bills"),
            row.get::<i64, _>("pending_bills"),
            row.get::<i64, _>("total_revenue"),
            row.get::<i64, _>("pending_amount"),
        ),
        Err(e) => {
            eprintln!("Billing stats error: {e}");
            (0, 0, 0, 0, 0)
        }
    };

    let rows = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.billing_type,
                  o.total, o.payment_status, o.source,
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
        let source: String = row.get("source");
        let pay_method: String = row.try_get("payment_method").unwrap_or_default();
        let date_str: String = row.get("date_str");

        let status_class = match pay_status.as_str() {
            "PAID" => "dash-badge-green",
            "CANCELLED" => "dash-badge-red",
            _ => "dash-badge-yellow",
        };
        let source_class = if source == "ONLINE" {
            "dash-badge-blue"
        } else {
            "dash-badge-yellow"
        };

        let actions = if pay_status == "PAID" {
            format!(
                r#"<a href="/admin/billing/order/{oid}" class="dash-view-link">View</a>
                   <a href="/admin/billing/order/{oid}" class="dash-view-link" onclick="window.open(this.href,'_blank');return false;">Print</a>"#,
                oid = esc(&order_id)
            )
        } else if pay_status == "PENDING" {
            format!(
                r#"<a href="/admin/billing/order/{oid}" class="dash-view-link">View</a>
                   <a href="/admin/billing/order/{oid}/receive-cash" class="dash-view-link" style="background:#dcfce7;color:#166534;border-color:#bbf7d0;">Receive Cash</a>"#,
                oid = esc(&order_id)
            )
        } else {
            format!(
                r#"<a href="/admin/billing/order/{oid}" class="dash-view-link">View</a>"#,
                oid = esc(&order_id)
            )
        };

        table_rows.push_str(&format!(
            r#"<tr>
                <td><a href="/admin/billing/order/{oid}" class="dash-id-link">{inv}</a></td>
                <td><strong>{customer}</strong></td>
                <td>{bt_label}</td>
                <td><strong>&#8377;{total}</strong></td>
                <td>{pay_method}</td>
                <td><span class="dash-badge {status_class}">{pay_status}</span></td>
                <td><span class="dash-badge {source_class}">{source}</span></td>
                <td>{date_str}</td>
                <td><div style="display:flex;gap:6px;">{actions}</div></td>
            </tr>"#,
            oid = esc(&order_id),
            inv = esc(&inv_num),
            customer = esc(&customer),
            bt_label = billing_type_label(&billing_type),
            total = total,
            pay_method = esc(&pay_method),
            status_class = status_class,
            pay_status = esc(&pay_status),
            source_class = source_class,
            source = esc(&source),
            date_str = date_str,
            actions = actions,
        ));
    }

    let empty = if rows.is_empty() {
        r#"<tr><td colspan="9" class="dash-empty-cell">No bills yet. <a href="/admin/billing/new">Create the first bill</a>.</td></tr>"#
    } else {
        ""
    };

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">FINANCE</div>
        <h1>Billing</h1>
        <p>Manage offline bills and track payment status.</p>
    </div>
    <a href="/admin/billing/new" class="admin-primary-button">+ Create New Bill</a>
</div>

<div class="dash-stat-grid" style="margin-bottom:20px;">
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-blue">BL</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">TOTAL BILLS</div>
            <div class="dash-stat-value">{total_bills}</div>
            <div class="dash-stat-sub">All time</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-green">PD</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">PAID BILLS</div>
            <div class="dash-stat-value">{paid_bills}</div>
            <div class="dash-stat-sub">Cash confirmed</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-yellow">PN</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">PENDING</div>
            <div class="dash-stat-value">{pending_bills}</div>
            <div class="dash-stat-sub">Awaiting cash</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-green">RV</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">TOTAL REVENUE</div>
            <div class="dash-stat-value">&#8377;{total_revenue}</div>
            <div class="dash-stat-sub">Paid only</div>
        </div>
    </div>
</div>

<div class="admin-panel">
    <div class="admin-panel-header">
        <div>
            <h2>All Bills</h2>
            <p>Pending amount: &#8377;{pending_amount}</p>
        </div>
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
                    <th>SOURCE</th>
                    <th>DATE</th>
                    <th>ACTIONS</th>
                </tr>
            </thead>
            <tbody>{table_rows}{empty}</tbody>
        </table>
    </div>
</div>
"#,
        total_bills = total_bills,
        paid_bills = paid_bills,
        pending_bills = pending_bills,
        total_revenue = total_revenue,
        pending_amount = pending_amount,
        table_rows = table_rows,
        empty = empty,
    );

    Html(admin_page("Billing", &content))
}

pub async fn order_detail(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Html<String> {
    let order = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.customer_phone,
                  o.billing_type, o.subtotal, o.discount, o.total,
                  o.payment_status, o.order_status, o.source, o.notes,
                  p.payment_method, p.paid_at,
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
        return Html(admin_page(
            "Billing",
            r#"<div class="admin-page-header"><div><h1>Bill Not Found</h1></div>
               <a href="/admin/billing" class="admin-secondary-button">Back</a></div>"#,
        ));
    };

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

    let (pay_status_label, pay_status_color) = match pay_status.as_str() {
        "PAID" => ("PAID", "#166534"),
        "CANCELLED" => ("CANCELLED", "#991b1b"),
        _ => ("CASH PENDING", "#92400e"),
    };

    let receive_cash_btn = if pay_status == "PENDING" {
        format!(
            r#"<a href="/admin/billing/order/{oid}/receive-cash"
               class="admin-primary-button" style="background:#16a34a;">
               Receive Cash
            </a>"#,
            oid = esc(&order_id)
        )
    } else {
        String::new()
    };

    let content = format!(
        r#"
<div class="admin-page-header no-print">
    <div>
        <div class="admin-eyebrow">BILLING</div>
        <h1>{inv_num}</h1>
        <p>{customer} &mdash; {bt_label}</p>
    </div>
    <div style="display:flex;gap:10px;align-items:center;">
        <a href="/admin/billing" class="admin-secondary-button">Back</a>
        {receive_cash_btn}
        <button type="button" class="admin-primary-button" onclick="window.print()">Print Invoice</button>
    </div>
</div>

<div class="invoice-page">
    <div class="invoice-document">
        <div class="invoice-header">
            <div>
                <div class="invoice-company-mark">SK</div>
                <div class="invoice-company">
                    <strong>SHRI KRISHNA TYRE HOUSE</strong>
                    <span>Tyres &amp; Auto Care &mdash; Singhana Road, Buhana, Jhunjhunu</span>
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
        pay_label = pay_status_label,
        pay_color = pay_status_color,
        pay_bg = if pay_status == "PAID" {
            "#f0fdf4"
        } else {
            "#fffbeb"
        },
        subtotal = subtotal,
        discount = discount,
        total = total,
        source = esc(&source),
        item_rows = item_rows,
        receive_cash_btn = receive_cash_btn,
        notes_html = if notes.is_empty() {
            String::new()
        } else {
            format!(
                "<span style='display:block;margin-top:6px;color:#6b7280;'>Note: {}</span>",
                esc(&notes)
            )
        },
    );

    Html(admin_page("Billing", &content))
}

pub async fn receive_cash_form(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Html<String> {
    let order = sqlx::query(
        r#"SELECT o.id, o.invoice_number, o.customer_name, o.total, o.payment_status,
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
        return Html(admin_page(
            "Billing",
            r#"<div class="admin-page-header"><div><h1>Bill Not Found</h1></div></div>"#,
        ));
    };

    let inv_num: String = order.get("invoice_number");
    let customer: String = order.get("customer_name");
    let total: i64 = order.get("total");
    let pay_status: String = order.get("payment_status");
    let pay_method: String = order.try_get("payment_method").unwrap_or_default();

    if pay_status == "PAID" {
        return Html(admin_page(
            "Billing",
            &format!(
                r#"<div class="admin-page-header">
                    <div><h1>Already Paid</h1><p>{inv} has already been marked as PAID.</p></div>
                    <a href="/admin/billing/order/{oid}" class="admin-secondary-button">View Invoice</a>
                </div>"#,
                inv = esc(&inv_num),
                oid = esc(&order_id),
            ),
        ));
    }

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">BILLING</div>
        <h1>Receive Cash</h1>
        <p>Confirm that cash has been physically received from the customer.</p>
    </div>
    <a href="/admin/billing/order/{oid}" class="admin-secondary-button">Cancel</a>
</div>

<div class="admin-panel" style="max-width:560px;margin:0 auto;">
    <div class="admin-panel-header">
        <div>
            <h2>Confirm Cash Receipt</h2>
            <p>This action marks the payment as PAID and updates inventory.</p>
        </div>
    </div>
    <div style="padding:28px;">
        <div style="background:#f8fafc;border:1px solid #e5e7eb;border-radius:10px;padding:20px;margin-bottom:24px;">
            <div style="display:flex;justify-content:space-between;margin-bottom:10px;">
                <span style="color:#6b7280;font-size:12px;">Invoice</span>
                <strong>{inv_num}</strong>
            </div>
            <div style="display:flex;justify-content:space-between;margin-bottom:10px;">
                <span style="color:#6b7280;font-size:12px;">Customer</span>
                <strong>{customer}</strong>
            </div>
            <div style="display:flex;justify-content:space-between;margin-bottom:10px;">
                <span style="color:#6b7280;font-size:12px;">Payment Method</span>
                <strong>{pay_method}</strong>
            </div>
            <div style="display:flex;justify-content:space-between;padding-top:12px;border-top:1px solid #e5e7eb;">
                <span style="color:#6b7280;font-size:13px;font-weight:700;">Amount Due</span>
                <strong style="font-size:22px;color:#111827;">&#8377;{total}</strong>
            </div>
        </div>

        <form method="post" action="/admin/billing/order/{oid}/receive-cash">
            <button type="submit" class="admin-primary-button"
                style="width:100%;padding:14px;font-size:14px;background:#16a34a;border-radius:10px;">
                Confirm Cash Received &mdash; &#8377;{total}
            </button>
        </form>

        <p style="margin-top:14px;text-align:center;color:#9ca3af;font-size:11px;">
            Only click this after the customer has physically handed over the cash.
        </p>
    </div>
</div>
"#,
        oid = esc(&order_id),
        inv_num = esc(&inv_num),
        customer = esc(&customer),
        pay_method = esc(&pay_method),
        total = total,
    );

    Html(admin_page("Billing", &content))
}

pub async fn confirm_cash_received(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Redirect {
    // Begin transaction with row lock
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Cash confirm tx error: {e}");
            return Redirect::to(&format!("/admin/billing/order/{}", order_id));
        }
    };

    // Lock and verify order is still PENDING
    let order =
        sqlx::query("SELECT id, total, payment_status FROM orders WHERE id = $1 FOR UPDATE")
            .bind(&order_id)
            .fetch_optional(&mut *tx)
            .await
            .unwrap_or(None);

    let Some(order) = order else {
        return Redirect::to("/admin/billing");
    };

    let pay_status: String = order.get("payment_status");
    if pay_status == "PAID" {
        // Already paid — idempotent, just redirect
        return Redirect::to(&format!("/admin/billing/order/{}", order_id));
    }
    if pay_status != "PENDING" {
        return Redirect::to(&format!("/admin/billing/order/{}", order_id));
    }

    // Mark order as PAID
    let _ = sqlx::query(
        "UPDATE orders SET payment_status = 'PAID', order_status = 'CONFIRMED', updated_at = NOW() WHERE id = $1",
    )
    .bind(&order_id)
    .execute(&mut *tx)
    .await;

    // Mark payment as PAID
    let _ = sqlx::query(
        "UPDATE payments SET payment_status = 'PAID', paid_at = NOW() WHERE order_id = $1 AND payment_status = 'PENDING'",
    )
    .bind(&order_id)
    .execute(&mut *tx)
    .await;

    // Reduce tyre stock for TYRE items
    let tyre_items = sqlx::query(
        "SELECT inventory_id, item_name, quantity FROM order_items WHERE order_id = $1 AND item_type = 'TYRE'",
    )
    .bind(&order_id)
    .fetch_all(&mut *tx)
    .await
    .unwrap_or_default();

    // Collect before loop to avoid borrow issues
    let tyre_data: Vec<(String, String, i32)> = tyre_items
        .iter()
        .filter_map(|r| {
            let inv_id: Option<String> = r.try_get("inventory_id").ok();
            let name: String = r.get("item_name");
            let qty: i32 = r.get("quantity");
            inv_id.map(|id| (id, name, qty))
        })
        .collect();

    for (inv_id, item_name, qty) in &tyre_data {
        let tyre = sqlx::query("SELECT id, stock FROM inventory WHERE id = $1 FOR UPDATE")
            .bind(inv_id)
            .fetch_optional(&mut *tx)
            .await
            .unwrap_or(None);

        if let Some(tyre) = tyre {
            let stock_before: i32 = tyre.get("stock");
            let stock_after = (stock_before - qty).max(0);

            let _ =
                sqlx::query("UPDATE inventory SET stock = $1, updated_at = NOW() WHERE id = $2")
                    .bind(stock_after)
                    .bind(inv_id)
                    .execute(&mut *tx)
                    .await;

            let mv_id = format!("STK-{}-{}", chrono::Utc::now().timestamp_millis(), inv_id);
            let _ = sqlx::query(
                r#"INSERT INTO stock_movements
                   (id, tyre_id, tyre_name, movement_type, quantity, stock_before, stock_after, reference, notes)
                   VALUES ($1,$2,$3,'Stock Out',$4,$5,$6,$7,$8)"#,
            )
            .bind(&mv_id)
            .bind(inv_id)
            .bind(item_name)
            .bind(-qty)
            .bind(stock_before)
            .bind(stock_after)
            .bind(&order_id)
            .bind(format!("Cash sale for order {}", order_id))
            .execute(&mut *tx)
            .await;
        }
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Cash confirm commit error: {e}");
        return Redirect::to(&format!("/admin/billing/order/{}", order_id));
    }

    Redirect::to(&format!("/admin/billing/order/{}", order_id))
}

// Keep old job-based billing handlers for backward compatibility
pub async fn bill_detail(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    // Try new orders system first
    let order = sqlx::query("SELECT id FROM orders WHERE id = $1")
        .bind(&id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

    if order.is_some() {
        return order_detail(State(state), Path(id)).await;
    }

    // Fall back to old job-based billing
    Html(admin_page(
        "Billing",
        r#"<div class="admin-page-header"><div><h1>Bill Not Found</h1></div>
           <a href="/admin/billing" class="admin-secondary-button">Back to Billing</a></div>"#,
    ))
}

pub async fn process_payment(State(_state): State<AppState>, Path(id): Path<String>) -> Redirect {
    Redirect::to(&format!("/admin/billing/order/{}", id))
}
