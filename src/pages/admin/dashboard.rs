use axum::{extract::State, response::Html};
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

pub async fn dashboard(State(state): State<AppState>) -> Html<String> {
    // ── Primary stats ────────────────────────────────────────────────────────
    let stats = sqlx::query(
        r#"
        SELECT
            (SELECT COUNT(*)                        FROM inventory)                                              AS total_tyres,
            (SELECT COALESCE(SUM(stock), 0)::BIGINT FROM inventory)                                              AS total_stock,
            (SELECT COUNT(*)                        FROM inventory WHERE stock > 0 AND stock <= low_stock_limit) AS low_stock,
            (SELECT COUNT(*)                        FROM inventory WHERE stock = 0)                              AS out_of_stock,
            (SELECT COUNT(*)                        FROM customers)                                              AS total_customers,
            (SELECT COUNT(*)                        FROM jobs WHERE status != 'COMPLETED')                       AS active_jobs,
            (SELECT COUNT(*)                        FROM jobs WHERE status = 'COMPLETED')                        AS completed_jobs,
            (SELECT COALESCE(SUM(total), 0)::BIGINT FROM jobs WHERE payment_status = 'PAID')                     AS total_sales
        "#,
    )
    .fetch_one(&state.pool)
    .await;

    let (
        total_tyres,
        total_stock,
        low_stock_count,
        out_of_stock,
        total_customers,
        active_jobs,
        completed_jobs,
        total_sales,
    ) = match stats {
        Ok(row) => (
            row.get::<i64, _>("total_tyres"),
            row.get::<i64, _>("total_stock"),
            row.get::<i64, _>("low_stock"),
            row.get::<i64, _>("out_of_stock"),
            row.get::<i64, _>("total_customers"),
            row.get::<i64, _>("active_jobs"),
            row.get::<i64, _>("completed_jobs"),
            row.get::<i64, _>("total_sales"),
        ),
        Err(e) => {
            eprintln!("Dashboard stats error: {e}");
            (0, 0, 0, 0, 0, 0, 0, 0)
        }
    };

    // ── Revenue: today / this week / this month ──────────────────────────────
    let revenue = sqlx::query(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN created_at >= CURRENT_DATE                   THEN total ELSE 0 END), 0)::BIGINT AS today,
            COALESCE(SUM(CASE WHEN created_at >= date_trunc('week',  NOW())     THEN total ELSE 0 END), 0)::BIGINT AS this_week,
            COALESCE(SUM(CASE WHEN created_at >= date_trunc('month', NOW())     THEN total ELSE 0 END), 0)::BIGINT AS this_month
        FROM invoices
        "#,
    )
    .fetch_one(&state.pool)
    .await;

    let (rev_today, rev_week, rev_month) = match revenue {
        Ok(row) => (
            row.get::<i64, _>("today"),
            row.get::<i64, _>("this_week"),
            row.get::<i64, _>("this_month"),
        ),
        Err(e) => {
            eprintln!("Dashboard revenue error: {e}");
            (0, 0, 0)
        }
    };

    // ── Today's activity ─────────────────────────────────────────────────────
    let today_activity = sqlx::query(
        r#"
        SELECT
            (SELECT COUNT(*) FROM customers       WHERE created_at >= CURRENT_DATE)                              AS new_customers,
            (SELECT COUNT(*) FROM jobs            WHERE created_at >= CURRENT_DATE)                              AS new_jobs,
            (SELECT COUNT(*) FROM jobs            WHERE status = 'COMPLETED' AND created_at >= CURRENT_DATE)     AS completed_today,
            (SELECT COUNT(*) FROM invoices        WHERE created_at >= CURRENT_DATE)                              AS payments_today,
            (SELECT COUNT(*) FROM stock_movements WHERE created_at >= CURRENT_DATE)                              AS stock_movements_today
        "#,
    )
    .fetch_one(&state.pool)
    .await;

    let (new_customers_today, new_jobs_today, completed_today, payments_today, stock_moves_today) =
        match today_activity {
            Ok(row) => (
                row.get::<i64, _>("new_customers"),
                row.get::<i64, _>("new_jobs"),
                row.get::<i64, _>("completed_today"),
                row.get::<i64, _>("payments_today"),
                row.get::<i64, _>("stock_movements_today"),
            ),
            Err(e) => {
                eprintln!("Dashboard today activity error: {e}");
                (0, 0, 0, 0, 0)
            }
        };

    // ── Low stock alerts ─────────────────────────────────────────────────────
    let low_stock_rows = sqlx::query(
        r#"SELECT id, name, brand, size, stock, low_stock_limit
           FROM inventory
           WHERE stock <= low_stock_limit
           ORDER BY stock ASC
           LIMIT 10"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut low_stock_html = String::new();
    if low_stock_rows.is_empty() {
        low_stock_html.push_str(
            r#"<tr><td colspan="7" class="dash-empty-cell">No low stock items</td></tr>"#,
        );
    } else {
        for row in &low_stock_rows {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let brand: String = row.get("brand");
            let size: String = row.get("size");
            let stock: i32 = row.get("stock");
            let limit: i32 = row.get("low_stock_limit");
            let (status_label, status_class) = if stock == 0 {
                ("Out of Stock", "dash-badge-red")
            } else {
                ("Low Stock", "dash-badge-yellow")
            };
            low_stock_html.push_str(&format!(
                r#"<tr>
                    <td><strong>{name}</strong></td>
                    <td>{brand}</td>
                    <td>{size}</td>
                    <td><span class="dash-stock-num">{stock}</span></td>
                    <td>{limit}</td>
                    <td><span class="dash-badge {status_class}">{status_label}</span></td>
                    <td><a href="/admin/inventory/{id}" class="dash-view-link">View</a></td>
                </tr>"#,
                name = esc(&name),
                brand = esc(&brand),
                size = esc(&size),
                stock = stock,
                limit = limit,
                status_label = status_label,
                status_class = status_class,
                id = esc(&id),
            ));
        }
    }

    // ── Recent jobs ──────────────────────────────────────────────────────────
    let recent_jobs = sqlx::query(
        r#"SELECT id, customer, vehicle, registration, status, total,
                  TO_CHAR(created_at AT TIME ZONE 'UTC', 'DD Mon YYYY') AS date_str
           FROM jobs
           ORDER BY created_at DESC
           LIMIT 5"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut recent_jobs_html = String::new();
    if recent_jobs.is_empty() {
        recent_jobs_html
            .push_str(r#"<tr><td colspan="7" class="dash-empty-cell">No job orders yet</td></tr>"#);
    } else {
        for row in &recent_jobs {
            let id: String = row.get("id");
            let customer: String = row.get("customer");
            let vehicle: String = row.get("vehicle");
            let registration: String = row.get("registration");
            let status: String = row.get("status");
            let total: i64 = row.get("total");
            let date_str: String = row.get("date_str");

            let status_class = match status.as_str() {
                "COMPLETED" => "dash-badge-green",
                "WORK IN PROGRESS" => "dash-badge-blue",
                _ => "dash-badge-yellow",
            };

            recent_jobs_html.push_str(&format!(
                r#"<tr>
                    <td><a href="/admin/jobs/{id}" class="dash-id-link">{id}</a></td>
                    <td><strong>{customer}</strong></td>
                    <td>{vehicle}<br><small>{registration}</small></td>
                    <td><span class="dash-badge {status_class}">{status}</span></td>
                    <td><strong>&#8377;{total}</strong></td>
                    <td>{date_str}</td>
                    <td><a href="/admin/jobs/{id}" class="dash-view-link">View</a></td>
                </tr>"#,
                id = esc(&id),
                customer = esc(&customer),
                vehicle = esc(&vehicle),
                registration = esc(&registration),
                status = esc(&status),
                status_class = status_class,
                total = total,
                date_str = date_str,
            ));
        }
    }

    // ── Recent customers ─────────────────────────────────────────────────────
    let recent_customers = sqlx::query(
        r#"SELECT c.id, c.name, c.phone, c.vehicle, c.registration,
                  TO_CHAR(c.created_at AT TIME ZONE 'UTC', 'DD Mon YYYY') AS joined_str,
                  (SELECT COUNT(*) FROM jobs WHERE customer = c.name) AS job_count
           FROM customers c
           ORDER BY c.created_at DESC
           LIMIT 5"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut recent_customers_html = String::new();
    if recent_customers.is_empty() {
        recent_customers_html
            .push_str(r#"<tr><td colspan="6" class="dash-empty-cell">No customers yet</td></tr>"#);
    } else {
        for row in &recent_customers {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let phone: String = row.get("phone");
            let vehicle: String = row.get("vehicle");
            let registration: String = row.get("registration");
            let job_count: i64 = row.get("job_count");
            let joined: String = row.get("joined_str");
            let initial = name
                .chars()
                .next()
                .unwrap_or('C')
                .to_uppercase()
                .next()
                .unwrap_or('C');

            recent_customers_html.push_str(&format!(
                r#"<tr>
                    <td>
                        <div class="dash-customer-cell">
                            <div class="dash-avatar">{initial}</div>
                            <div>
                                <strong>{name}</strong>
                                <small>{phone}</small>
                            </div>
                        </div>
                    </td>
                    <td>{phone}</td>
                    <td>{vehicle}<br><small>{registration}</small></td>
                    <td><span class="dash-job-count">{job_count}</span></td>
                    <td>{joined}</td>
                    <td><a href="/admin/customers/{id}" class="dash-view-link">View</a></td>
                </tr>"#,
                initial = initial,
                name = esc(&name),
                phone = esc(&phone),
                vehicle = esc(&vehicle),
                registration = esc(&registration),
                job_count = job_count,
                joined = joined,
                id = esc(&id),
            ));
        }
    }

    // ── Inventory bar widths ─────────────────────────────────────────────────
    let inv_total = total_tyres.max(1) as f64;
    let good_stock = (total_tyres - low_stock_count - out_of_stock).max(0);
    let good_pct = (good_stock as f64 / inv_total * 100.0) as u32;
    let low_pct = (low_stock_count as f64 / inv_total * 100.0) as u32;
    let out_pct = (out_of_stock as f64 / inv_total * 100.0) as u32;

    // ── Build page content ───────────────────────────────────────────────────
    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">OVERVIEW</div>
        <h1>Dashboard</h1>
        <p>Overview of Shri Krishna Tyre House operations.</p>
    </div>
    <a href="/admin/jobs/new" class="admin-primary-button">+ New Job Order</a>
</div>

<!-- PRIMARY STAT CARDS -->
<div class="dash-stat-grid">
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-blue">TY</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">TOTAL TYRES</div>
            <div class="dash-stat-value">{total_tyres}</div>
            <div class="dash-stat-sub">Total tyre products</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-green">ST</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">TOTAL STOCK</div>
            <div class="dash-stat-value">{total_stock}</div>
            <div class="dash-stat-sub">Total physical units</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-yellow">LW</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">LOW STOCK</div>
            <div class="dash-stat-value">{low_stock_count}</div>
            <div class="dash-stat-sub">Below reorder limit</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-red">OS</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">OUT OF STOCK</div>
            <div class="dash-stat-value">{out_of_stock}</div>
            <div class="dash-stat-sub">Zero units remaining</div>
        </div>
    </div>
</div>

<!-- BUSINESS STAT CARDS -->
<div class="dash-stat-grid" style="margin-top:14px;">
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-purple">CU</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">TOTAL CUSTOMERS</div>
            <div class="dash-stat-value">{total_customers}</div>
            <div class="dash-stat-sub">Registered customers</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-blue">AJ</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">ACTIVE JOBS</div>
            <div class="dash-stat-value">{active_jobs}</div>
            <div class="dash-stat-sub">In progress or new</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-green">CJ</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">COMPLETED JOBS</div>
            <div class="dash-stat-value">{completed_jobs}</div>
            <div class="dash-stat-sub">All time completed</div>
        </div>
    </div>
    <div class="dash-stat-card">
        <div class="dash-stat-icon dash-icon-green">TS</div>
        <div class="dash-stat-body">
            <div class="dash-stat-label">TOTAL SALES</div>
            <div class="dash-stat-value">&#8377;{total_sales}</div>
            <div class="dash-stat-sub">From paid invoices</div>
        </div>
    </div>
</div>

<!-- INVENTORY OVERVIEW + QUICK ACTIONS -->
<div class="dash-two-col" style="margin-top:20px;">

    <div class="admin-panel">
        <div class="admin-panel-header">
            <div>
                <h2>Inventory Overview</h2>
                <p>Stock distribution across all tyre products.</p>
            </div>
            <a href="/admin/inventory" class="dash-view-link">View All</a>
        </div>
        <div class="dash-inv-body">
            <div class="dash-inv-row">
                <div class="dash-inv-meta">
                    <span>Total Products</span>
                    <strong>{total_tyres}</strong>
                </div>
                <div class="dash-inv-bar-wrap">
                    <div class="dash-inv-bar" style="width:100%;background:#e5e7eb;"></div>
                </div>
            </div>
            <div class="dash-inv-row">
                <div class="dash-inv-meta">
                    <span>Total Stock Units</span>
                    <strong>{total_stock}</strong>
                </div>
                <div class="dash-inv-bar-wrap">
                    <div class="dash-inv-bar" style="width:100%;background:#2563eb;"></div>
                </div>
            </div>
            <div class="dash-inv-row">
                <div class="dash-inv-meta">
                    <span>Good Stock</span>
                    <strong>{good_stock}</strong>
                </div>
                <div class="dash-inv-bar-wrap">
                    <div class="dash-inv-bar" style="width:{good_pct}%;background:#16a34a;"></div>
                </div>
            </div>
            <div class="dash-inv-row">
                <div class="dash-inv-meta">
                    <span>Low Stock</span>
                    <strong>{low_stock_count}</strong>
                </div>
                <div class="dash-inv-bar-wrap">
                    <div class="dash-inv-bar" style="width:{low_pct}%;background:#f59e0b;"></div>
                </div>
            </div>
            <div class="dash-inv-row">
                <div class="dash-inv-meta">
                    <span>Out of Stock</span>
                    <strong>{out_of_stock}</strong>
                </div>
                <div class="dash-inv-bar-wrap">
                    <div class="dash-inv-bar" style="width:{out_pct}%;background:#dc2626;"></div>
                </div>
            </div>
        </div>
    </div>

    <div class="admin-panel">
        <div class="admin-panel-header">
            <div>
                <h2>Quick Actions</h2>
                <p>Common tasks and shortcuts.</p>
            </div>
        </div>
        <div class="dash-quick-actions">
            <a href="/admin/inventory/new" class="dash-action-item">
                <div class="dash-action-icon dash-icon-blue">TY</div>
                <div>
                    <strong>+ Add Tyre</strong>
                    <span>Add new tyre to inventory</span>
                </div>
            </a>
            <a href="/admin/customers/new" class="dash-action-item">
                <div class="dash-action-icon dash-icon-purple">CU</div>
                <div>
                    <strong>+ New Customer</strong>
                    <span>Register a new customer</span>
                </div>
            </a>
            <a href="/admin/jobs/new" class="dash-action-item">
                <div class="dash-action-icon dash-icon-green">JB</div>
                <div>
                    <strong>+ New Job Order</strong>
                    <span>Create a new job order</span>
                </div>
            </a>
            <a href="/admin/billing" class="dash-action-item">
                <div class="dash-action-icon dash-icon-yellow">BL</div>
                <div>
                    <strong>View Billing</strong>
                    <span>Process payments</span>
                </div>
            </a>
            <a href="/admin/invoices" class="dash-action-item">
                <div class="dash-action-icon dash-icon-blue">IN</div>
                <div>
                    <strong>View Invoices</strong>
                    <span>Browse all invoices</span>
                </div>
            </a>
        </div>
    </div>

</div>

<!-- LOW STOCK ALERTS -->
<div class="admin-panel" style="margin-top:20px;">
    <div class="admin-panel-header">
        <div>
            <h2>Low Stock Alerts</h2>
            <p>Tyres at or below their reorder limit.</p>
        </div>
        <a href="/admin/inventory" class="dash-view-link">View Inventory</a>
    </div>
    <div class="dash-table-wrap">
        <table class="dash-table">
            <thead>
                <tr>
                    <th>TYRE</th>
                    <th>BRAND</th>
                    <th>SIZE</th>
                    <th>STOCK</th>
                    <th>LIMIT</th>
                    <th>STATUS</th>
                    <th>ACTION</th>
                </tr>
            </thead>
            <tbody>{low_stock_html}</tbody>
        </table>
    </div>
</div>

<!-- RECENT JOB ORDERS -->
<div class="admin-panel" style="margin-top:20px;">
    <div class="admin-panel-header">
        <div>
            <h2>Recent Job Orders</h2>
            <p>Latest 5 job orders.</p>
        </div>
        <a href="/admin/jobs" class="dash-view-link">View All</a>
    </div>
    <div class="dash-table-wrap">
        <table class="dash-table">
            <thead>
                <tr>
                    <th>JOB NUMBER</th>
                    <th>CUSTOMER</th>
                    <th>VEHICLE</th>
                    <th>STATUS</th>
                    <th>TOTAL</th>
                    <th>DATE</th>
                    <th>ACTION</th>
                </tr>
            </thead>
            <tbody>{recent_jobs_html}</tbody>
        </table>
    </div>
</div>

<!-- RECENT CUSTOMERS -->
<div class="admin-panel" style="margin-top:20px;">
    <div class="admin-panel-header">
        <div>
            <h2>Recent Customers</h2>
            <p>Latest 5 registered customers.</p>
        </div>
        <a href="/admin/customers" class="dash-view-link">View All</a>
    </div>
    <div class="dash-table-wrap">
        <table class="dash-table">
            <thead>
                <tr>
                    <th>CUSTOMER</th>
                    <th>PHONE</th>
                    <th>VEHICLE</th>
                    <th>JOBS</th>
                    <th>JOINED</th>
                    <th>ACTION</th>
                </tr>
            </thead>
            <tbody>{recent_customers_html}</tbody>
        </table>
    </div>
</div>

<!-- BOTTOM ROW: TODAY'S ACTIVITY + SALES OVERVIEW -->
<div class="dash-two-col" style="margin-top:20px;">

    <div class="admin-panel">
        <div class="admin-panel-header">
            <div>
                <h2>Today's Activity</h2>
                <p>Database activity for today.</p>
            </div>
        </div>
        <div class="dash-activity-list">
            <div class="dash-activity-item">
                <div class="dash-activity-icon dash-icon-purple">CU</div>
                <div class="dash-activity-text">
                    <strong>New Customers</strong>
                    <span>Registered today</span>
                </div>
                <div class="dash-activity-count">{new_customers_today}</div>
            </div>
            <div class="dash-activity-item">
                <div class="dash-activity-icon dash-icon-blue">JB</div>
                <div class="dash-activity-text">
                    <strong>New Jobs</strong>
                    <span>Created today</span>
                </div>
                <div class="dash-activity-count">{new_jobs_today}</div>
            </div>
            <div class="dash-activity-item">
                <div class="dash-activity-icon dash-icon-green">CJ</div>
                <div class="dash-activity-text">
                    <strong>Completed Jobs</strong>
                    <span>Finished today</span>
                </div>
                <div class="dash-activity-count">{completed_today}</div>
            </div>
            <div class="dash-activity-item">
                <div class="dash-activity-icon dash-icon-green">PY</div>
                <div class="dash-activity-text">
                    <strong>Payments</strong>
                    <span>Invoices paid today</span>
                </div>
                <div class="dash-activity-count">{payments_today}</div>
            </div>
            <div class="dash-activity-item">
                <div class="dash-activity-icon dash-icon-yellow">SM</div>
                <div class="dash-activity-text">
                    <strong>Stock Movements</strong>
                    <span>Inventory changes today</span>
                </div>
                <div class="dash-activity-count">{stock_moves_today}</div>
            </div>
        </div>
    </div>

    <div class="admin-panel">
        <div class="admin-panel-header">
            <div>
                <h2>Sales Overview</h2>
                <p>Revenue from paid invoices.</p>
            </div>
        </div>
        <div class="dash-revenue-list">
            <div class="dash-revenue-item">
                <div>
                    <strong>Today</strong>
                    <span>Revenue collected today</span>
                </div>
                <div class="dash-revenue-value">&#8377;{rev_today}</div>
            </div>
            <div class="dash-revenue-item">
                <div>
                    <strong>This Week</strong>
                    <span>Revenue this week</span>
                </div>
                <div class="dash-revenue-value">&#8377;{rev_week}</div>
            </div>
            <div class="dash-revenue-item dash-revenue-highlight">
                <div>
                    <strong>This Month</strong>
                    <span>Revenue this month</span>
                </div>
                <div class="dash-revenue-value">&#8377;{rev_month}</div>
            </div>
        </div>
    </div>

</div>
"#,
        total_tyres = total_tyres,
        total_stock = total_stock,
        low_stock_count = low_stock_count,
        out_of_stock = out_of_stock,
        total_customers = total_customers,
        active_jobs = active_jobs,
        completed_jobs = completed_jobs,
        total_sales = total_sales,
        good_stock = good_stock,
        good_pct = good_pct,
        low_pct = low_pct,
        out_pct = out_pct,
        low_stock_html = low_stock_html,
        recent_jobs_html = recent_jobs_html,
        recent_customers_html = recent_customers_html,
        new_customers_today = new_customers_today,
        new_jobs_today = new_jobs_today,
        completed_today = completed_today,
        payments_today = payments_today,
        stock_moves_today = stock_moves_today,
        rev_today = rev_today,
        rev_week = rev_week,
        rev_month = rev_month,
    );

    Html(admin_page("Dashboard", &content))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
