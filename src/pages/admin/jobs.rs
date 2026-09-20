use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

pub async fn jobs(State(state): State<AppState>) -> Html<String> {
    let rows = sqlx::query(
        r#"SELECT j.id, j.customer, j.vehicle, j.vehicle_type, j.registration,
                  j.status, j.total,
                  (SELECT COUNT(*) FROM job_items WHERE job_id = j.id) AS item_count
           FROM jobs j ORDER BY j.created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut table_rows = String::new();
    for row in &rows {
        let id: String = row.get("id");
        let customer: String = row.get("customer");
        let vehicle: String = row.get("vehicle");
        let vehicle_type: String = row.get("vehicle_type");
        let registration: String = row.get("registration");
        let status: String = row.get("status");
        let total: i64 = row.get("total");
        let item_count: i64 = row.get("item_count");

        table_rows.push_str(&format!(
            r#"<tr>
    <td><a href="/admin/jobs/{id}" class="job-id-link">{id}</a></td>
    <td>
        <strong>{customer}</strong>
        <small>{registration}</small>
    </td>
    <td>
        <strong>{vehicle}</strong>
        <small>{vehicle_type}</small>
    </td>
    <td>{item_count}</td>
    <td><strong>&#8377;{total}</strong></td>
    <td><span class="job-status">{status}</span></td>
    <td><a href="/admin/jobs/{id}" class="job-view-button">View</a></td>
</tr>"#,
            id = id,
            customer = escape_html(&customer),
            registration = escape_html(&registration),
            vehicle = escape_html(&vehicle),
            vehicle_type = escape_html(&vehicle_type),
            item_count = item_count,
            total = total,
            status = escape_html(&status),
        ));
    }

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">OPERATIONS</div>
        <h1>Job Orders</h1>
        <p>Manage tyre work, services and vehicle jobs.</p>
    </div>
    <a href="/admin/jobs/new" class="admin-primary-button">+ Create Job</a>
</div>

<div class="admin-panel">
    <div class="job-admin-toolbar">
        <input type="search" class="admin-search-input" placeholder="Search customer, vehicle or job ID...">
        <select class="admin-filter-select">
            <option>All Status</option>
            <option>NEW</option>
            <option>WORK IN PROGRESS</option>
            <option>COMPLETED</option>
        </select>
    </div>
    <div class="job-admin-table-wrap">
        <table class="job-admin-table">
            <thead>
                <tr>
                    <th>Job ID</th>
                    <th>Customer</th>
                    <th>Vehicle</th>
                    <th>Items</th>
                    <th>Total</th>
                    <th>Status</th>
                    <th>Action</th>
                </tr>
            </thead>
            <tbody>{table_rows}</tbody>
        </table>
    </div>
</div>
"#,
        table_rows = table_rows,
    );

    Html(admin_page("Job Orders", &content))
}

pub async fn job_detail(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    let job = sqlx::query(
        "SELECT id, customer, vehicle, vehicle_type, registration, notes, status, total, payment_status FROM jobs WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(job) = job else {
        return Html(admin_page(
            "Job Orders",
            r#"<div class="admin-page-header"><div><h1>Job Not Found</h1></div></div>
               <div class="admin-panel"><a href="/admin/jobs" class="admin-secondary-button">Back to Jobs</a></div>"#,
        ));
    };

    let job_id: String = job.get("id");
    let customer: String = job.get("customer");
    let vehicle: String = job.get("vehicle");
    let vehicle_type: String = job.get("vehicle_type");
    let registration: String = job.get("registration");
    let notes: String = job.get("notes");
    let status: String = job.get("status");
    let total: i64 = job.get("total");

    let items = sqlx::query(
        "SELECT name, item_type, quantity, price FROM job_items WHERE job_id = $1 ORDER BY id",
    )
    .bind(&job_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut items_html = String::new();
    for item in &items {
        let name: String = item.get("name");
        let item_type: String = item.get("item_type");
        let quantity: i32 = item.get("quantity");
        let price: i64 = item.get("price");
        let subtotal = price * quantity as i64;
        items_html.push_str(&format!(
            r#"<tr>
                <td>{name}</td>
                <td>{item_type}</td>
                <td>{quantity}</td>
                <td>&#8377;{price}</td>
                <td>&#8377;{subtotal}</td>
            </tr>"#,
            name = escape_html(&name),
            item_type = escape_html(&item_type),
            quantity = quantity,
            price = price,
            subtotal = subtotal,
        ));
    }

    let action = match status.as_str() {
        "NEW" => format!(
            r#"<form method="post" action="/admin/jobs/{id}/start">
                <button type="submit" class="job-action-button job-start-button">Start Job</button>
            </form>"#,
            id = job_id
        ),
        "WORK IN PROGRESS" => format!(
            r#"<form method="post" action="/admin/jobs/{id}/complete">
                <button type="submit" class="job-action-button job-complete-button">Mark Completed</button>
            </form>"#,
            id = job_id
        ),
        _ => r#"<div class="job-completed-message">Job Completed</div>"#.to_string(),
    };

    let notes_display = if notes.is_empty() {
        "No notes added."
    } else {
        &notes
    };

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">JOB ORDER</div>
        <h1>{job_id}</h1>
        <p>{customer} - {vehicle}</p>
    </div>
    <div class="job-detail-actions">
        <a href="/admin/jobs" class="admin-secondary-button">Back to Jobs</a>
        {action}
    </div>
</div>

<div class="job-detail-grid">
    <div>
        <div class="admin-panel">
            <div class="admin-panel-header">
                <div>
                    <h2>Job Items</h2>
                    <p>Tyres and services included in this job.</p>
                </div>
                <span class="job-status">{status}</span>
            </div>
            <div class="job-admin-table-wrap">
                <table class="job-admin-table">
                    <thead>
                        <tr>
                            <th>Item</th><th>Type</th><th>Quantity</th><th>Price</th><th>Subtotal</th>
                        </tr>
                    </thead>
                    <tbody>{items_html}</tbody>
                </table>
            </div>
        </div>

        <div class="admin-panel">
            <div class="admin-panel-header"><div><h2>Job Notes</h2></div></div>
            <div class="job-notes-box">{notes_display}</div>
        </div>
    </div>

    <div>
        <div class="job-detail-card">
            <div class="job-detail-card-header">Customer</div>
            <strong>{customer}</strong>
            <small>{registration}</small>
        </div>
        <div class="job-detail-card">
            <div class="job-detail-card-header">Vehicle</div>
            <strong>{vehicle}</strong>
            <small>{vehicle_type}</small>
            <small>{registration}</small>
        </div>
        <div class="job-detail-card job-total-card">
            <span>Estimated Total</span>
            <strong>&#8377;{total}</strong>
        </div>
    </div>
</div>
"#,
        job_id = job_id,
        customer = escape_html(&customer),
        vehicle = escape_html(&vehicle),
        vehicle_type = escape_html(&vehicle_type),
        registration = escape_html(&registration),
        status = escape_html(&status),
        notes_display = escape_html(notes_display),
        total = total,
        items_html = items_html,
        action = action,
    );

    Html(admin_page("Job Orders", &content))
}

pub async fn start_job(State(state): State<AppState>, Path(id): Path<String>) -> Redirect {
    let _ =
        sqlx::query("UPDATE jobs SET status = 'WORK IN PROGRESS' WHERE id = $1 AND status = 'NEW'")
            .bind(&id)
            .execute(&state.pool)
            .await;

    Redirect::to(&format!("/admin/jobs/{}", id))
}

pub async fn complete_job(State(state): State<AppState>, Path(id): Path<String>) -> Redirect {
    let _ = sqlx::query(
        "UPDATE jobs SET status = 'COMPLETED' WHERE id = $1 AND status = 'WORK IN PROGRESS'",
    )
    .bind(&id)
    .execute(&state.pool)
    .await;

    Redirect::to(&format!("/admin/jobs/{}", id))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
