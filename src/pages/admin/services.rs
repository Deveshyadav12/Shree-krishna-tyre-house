use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect},
};
use serde::Deserialize;
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

#[derive(Deserialize)]
pub struct ServiceForm {
    pub name: String,
    pub description: String,
    pub price: String,
}

pub async fn services(State(state): State<AppState>) -> Html<String> {
    let rows =
        sqlx::query("SELECT id, name, description, price, active FROM services ORDER BY name")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let total: usize = rows.len();
    let active: usize = rows.iter().filter(|r| r.get::<bool, _>("active")).count();

    let mut table_rows = String::new();
    for row in &rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let price: i64 = row.get("price");
        let active_val: bool = row.get("active");
        let status = if active_val { "Active" } else { "Inactive" };

        table_rows.push_str(&format!(
            r#"<tr>
    <td><span class="table-id">{id}</span></td>
    <td>
        <div class="service-table-name">
            <div class="service-mini-icon">SV</div>
            <div>
                <strong>{name}</strong>
                <small>{description}</small>
            </div>
        </div>
    </td>
    <td><strong>&#8377;{price}</strong></td>
    <td><span class="service-status">&#9679; {status}</span></td>
    <td>
        <div class="brand-table-actions">
            <form method="post" action="/admin/services/{id}/delete" style="display:inline">
                <button class="brand-action delete" type="submit" onclick="return confirm('Delete this service?')">Delete</button>
            </form>
        </div>
    </td>
</tr>"#,
            id = escape_html(&id),
            name = escape_html(&name),
            description = escape_html(&description),
            price = price,
            status = status,
        ));
    }

    let content = format!(
        r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">SERVICE MANAGEMENT</span>
            <h1>Services</h1>
            <p>Manage tyre and wheel services and pricing.</p>
        </div>
        <div class="admin-page-actions">
            <button type="button" class="admin-primary-button" onclick="document.getElementById('addServiceForm').style.display='block'">
                + Add Service
            </button>
        </div>
    </div>

    <div class="af-stat-row" style="margin-bottom:20px">
        <div class="brand-admin-stat">
            <span class="brand-admin-stat-icon">SV</span>
            <div><strong>{total}</strong><small>Total Services</small></div>
        </div>
        <div class="brand-admin-stat">
            <span class="brand-admin-stat-icon">AC</span>
            <div><strong>{active}</strong><small>Active</small></div>
        </div>
    </div>

    <div id="addServiceForm" style="display:none;margin-bottom:20px" class="admin-panel">
        <div class="admin-panel-header af-panel-header">
            <div><h2>Add Service</h2><p>Add a new service to the catalogue.</p></div>
        </div>
        <form method="post" action="/admin/services" class="af-form">
            <div class="af-grid af-body">
                <div class="admin-form-group">
                    <label>Service Name</label>
                    <input name="name" type="text" required placeholder="Wheel Alignment">
                </div>
                <div class="admin-form-group">
                    <label>Price (&#8377;)</label>
                    <input name="price" type="number" min="0" required placeholder="500">
                </div>
                <div class="admin-form-group af-full">
                    <label>Description</label>
                    <input name="description" type="text" placeholder="Professional service">
                </div>
            </div>
            <div class="af-actions af-actions-pad">
                <button type="button" onclick="document.getElementById('addServiceForm').style.display='none'" class="admin-secondary-button">Cancel</button>
                <button type="submit" class="admin-primary-button">Save Service</button>
            </div>
        </form>
    </div>

    <div class="admin-panel">
        <div class="admin-panel-header af-panel-header">
            <div>
                <h2>Service Directory</h2>
                <p>All services currently configured in the system.</p>
            </div>
        </div>
        <div class="brand-admin-table-wrapper">
            <table class="brand-admin-table">
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>SERVICE</th>
                        <th>PRICE</th>
                        <th>STATUS</th>
                        <th>ACTIONS</th>
                    </tr>
                </thead>
                <tbody>{table_rows}</tbody>
            </table>
        </div>
        <div class="brand-admin-footer">
            <span>Showing {total} of {total} services</span>
        </div>
    </div>
</section>
"#,
        total = total,
        active = active,
        table_rows = table_rows,
    );

    Html(admin_page("Services", &content))
}

pub async fn create_service(
    State(state): State<AppState>,
    Form(form): Form<ServiceForm>,
) -> Redirect {
    let price = form.price.trim().parse::<i64>().unwrap_or(0);
    let id = format!("SRV-{}", chrono::Utc::now().timestamp_millis());

    let _ = sqlx::query(
        "INSERT INTO services (id, name, description, price, active) VALUES ($1, $2, $3, $4, TRUE)",
    )
    .bind(&id)
    .bind(form.name.trim())
    .bind(form.description.trim())
    .bind(price)
    .execute(&state.pool)
    .await;

    Redirect::to("/admin/services")
}

pub async fn delete_service(State(state): State<AppState>, Path(id): Path<String>) -> Redirect {
    let _ = sqlx::query("DELETE FROM services WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await;

    Redirect::to("/admin/services")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
