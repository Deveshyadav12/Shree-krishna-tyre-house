use axum::{
    extract::{Path, State},
    response::Html,
};
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

pub async fn customers(State(state): State<AppState>) -> Html<String> {
    let rows = sqlx::query(
        "SELECT id, name, phone, email, address, vehicle, registration FROM customers ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let total = rows.len();
    let mut table_rows = String::new();

    for row in &rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let phone: String = row.get("phone");
        let email: String = row.get("email");
        let address: String = row.get("address");
        let vehicle: String = row.get("vehicle");
        let registration: String = row.get("registration");

        table_rows.push_str(&format!(
            r#"<tr>
    <td><a href="/admin/customers/{id}" class="customer-id-link">{id}</a></td>
    <td>
        <div class="job-customer-cell">
            <strong>{name}</strong>
            <small>{phone}</small>
        </div>
    </td>
    <td>
        <div class="job-customer-cell">
            <strong>{vehicle}</strong>
            <small>{registration}</small>
        </div>
    </td>
    <td>{email}</td>
    <td>{address}</td>
    <td><a href="/admin/customers/{id}" class="job-view-button">View</a></td>
</tr>"#,
            id = id,
            name = escape_html(&name),
            phone = escape_html(&phone),
            email = escape_html(&email),
            address = escape_html(&address),
            vehicle = escape_html(&vehicle),
            registration = escape_html(&registration),
        ));
    }

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">MAIN</div>
        <h1>Customers</h1>
        <p>Manage customer information and vehicles.</p>
    </div>
    <a href="/admin/customers/new" class="admin-primary-button">+ Add Customer</a>
</div>

<div class="customer-stat-grid">
    <div class="customer-stat-card">
        <span>Total Customers</span>
        <strong>{total}</strong>
    </div>
    <div class="customer-stat-card">
        <span>Vehicles</span>
        <strong>{total}</strong>
    </div>
</div>

<div class="admin-panel">
    <div class="job-admin-toolbar">
        <input type="search" class="admin-search-input" placeholder="Search name, phone, vehicle or registration...">
    </div>
    <div class="job-admin-table-wrap">
        <table class="job-admin-table">
            <thead>
                <tr>
                    <th>Customer ID</th>
                    <th>Customer</th>
                    <th>Vehicle</th>
                    <th>Email</th>
                    <th>Address</th>
                    <th>Action</th>
                </tr>
            </thead>
            <tbody>{table_rows}</tbody>
        </table>
    </div>
</div>
"#,
        total = total,
        table_rows = table_rows,
    );

    Html(admin_page("Customers", &content))
}

pub async fn customer_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Html<String> {
    let row = sqlx::query(
        "SELECT id, name, phone, email, address, vehicle, registration FROM customers WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(row) = row else {
        return Html(admin_page(
            "Customers",
            r#"<div class="admin-page-header"><div><h1>Customer Not Found</h1></div></div>
               <div class="admin-panel"><a href="/admin/customers" class="admin-secondary-button">Back to Customers</a></div>"#,
        ));
    };

    let cid: String = row.get("id");
    let name: String = row.get("name");
    let phone: String = row.get("phone");
    let email: String = row.get("email");
    let address: String = row.get("address");
    let vehicle: String = row.get("vehicle");
    let registration: String = row.get("registration");
    let initial = name.chars().next().unwrap_or('C');

    let job_rows = sqlx::query(
        "SELECT id, status, total, payment_status FROM jobs WHERE customer = $1 ORDER BY created_at DESC LIMIT 10",
    )
    .bind(&name)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let total_spent: i64 = job_rows
        .iter()
        .filter(|r| r.get::<String, _>("payment_status") == "PAID")
        .map(|r| r.get::<i64, _>("total"))
        .sum();

    let outstanding: i64 = job_rows
        .iter()
        .filter(|r| {
            r.get::<String, _>("status") == "COMPLETED"
                && r.get::<String, _>("payment_status") == "UNPAID"
        })
        .map(|r| r.get::<i64, _>("total"))
        .sum();

    let mut jobs_html = String::new();
    for r in &job_rows {
        let jid: String = r.get("id");
        let status: String = r.get("status");
        let total: i64 = r.get("total");
        let pay_status: String = r.get("payment_status");
        jobs_html.push_str(&format!(
            r#"<tr>
                <td><a href="/admin/jobs/{jid}">{jid}</a></td>
                <td>{status}</td>
                <td>&#8377;{total}</td>
                <td>{pay_status}</td>
            </tr>"#,
            jid = jid,
            status = escape_html(&status),
            total = total,
            pay_status = escape_html(&pay_status),
        ));
    }

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">CUSTOMER</div>
        <h1>{name}</h1>
        <p>Customer ID: {cid}</p>
    </div>
    <a href="/admin/customers" class="admin-secondary-button">Back to Customers</a>
</div>

<div class="customer-detail-grid">
    <div>
        <div class="admin-panel">
            <div class="admin-panel-header">
                <div>
                    <h2>Customer Information</h2>
                    <p>Basic customer contact information.</p>
                </div>
            </div>
            <div class="customer-info-grid">
                <div><span>Name</span><strong>{name}</strong></div>
                <div><span>Phone</span><strong>{phone}</strong></div>
                <div><span>Email</span><strong>{email}</strong></div>
                <div><span>Address</span><strong>{address}</strong></div>
            </div>
        </div>

        <div class="admin-panel">
            <div class="admin-panel-header">
                <div>
                    <h2>Vehicle</h2>
                    <p>Vehicle registered to this customer.</p>
                </div>
            </div>
            <div class="customer-vehicle-card">
                <div class="customer-vehicle-icon">VH</div>
                <div>
                    <strong>{vehicle}</strong>
                    <span>{registration}</span>
                </div>
            </div>
        </div>

        <div class="admin-panel">
            <div class="admin-panel-header">
                <div>
                    <h2>Job History</h2>
                    <p>Total spent: &#8377;{total_spent} &nbsp;|&nbsp; Outstanding: &#8377;{outstanding}</p>
                </div>
            </div>
            <table class="job-admin-table">
                <thead>
                    <tr><th>Job</th><th>Status</th><th>Total</th><th>Payment</th></tr>
                </thead>
                <tbody>{jobs_html}</tbody>
            </table>
        </div>
    </div>

    <aside>
        <div class="customer-profile-card">
            <div class="customer-profile-avatar">{initial}</div>
            <strong>{name}</strong>
            <span>{phone}</span>
            <a href="/admin/jobs/new" class="admin-primary-button">Create Job</a>
        </div>
    </aside>
</div>
"#,
        cid = cid,
        name = escape_html(&name),
        phone = escape_html(&phone),
        email = escape_html(&email),
        address = escape_html(&address),
        vehicle = escape_html(&vehicle),
        registration = escape_html(&registration),
        initial = initial,
        total_spent = total_spent,
        outstanding = outstanding,
        jobs_html = jobs_html,
    );

    Html(admin_page("Customers", &content))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
