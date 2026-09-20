use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect},
};
use serde::Deserialize;
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

#[derive(Deserialize)]
pub struct BrandForm {
    pub name: String,
    pub description: String,
}

pub async fn brands(State(state): State<AppState>) -> Html<String> {
    let rows = sqlx::query(
        r#"SELECT b.id, b.name, b.description, b.active, b.created_at,
                  (SELECT COUNT(*) FROM inventory WHERE brand = b.name) AS tyre_count
           FROM brands b ORDER BY b.name"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let total = rows.len();
    let active_count: usize = rows.iter().filter(|r| r.get::<bool, _>("active")).count();
    let total_tyres: i64 = rows.iter().map(|r| r.get::<i64, _>("tyre_count")).sum();

    let mut table_rows = String::new();
    for row in &rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let active: bool = row.get("active");
        let tyre_count: i64 = row.get("tyre_count");
        let status = if active { "Active" } else { "Inactive" };
        let status_class = if active {
            "brand-active"
        } else {
            "brand-inactive"
        };
        let abbr: String = name.chars().take(2).collect::<String>().to_uppercase();

        table_rows.push_str(&format!(
            r#"<tr>
    <td>
        <div class="brand-table-name">
            <div class="brand-logo-box">{abbr}</div>
            <div>
                <strong>{name}</strong>
                <small>{description}</small>
            </div>
        </div>
    </td>
    <td>{id}</td>
    <td><strong>{tyre_count}</strong></td>
    <td>
        <span class="brand-status {status_class}">{status}</span>
    </td>
    <td>
        <div class="brand-table-actions">
            <form method="post" action="/admin/brands/{id}/delete" style="display:inline">
                <button class="brand-action delete" type="submit" onclick="return confirm('Delete this brand?')">Delete</button>
            </form>
        </div>
    </td>
</tr>"#,
            abbr = abbr,
            id = escape_html(&id),
            name = escape_html(&name),
            description = escape_html(&description),
            tyre_count = tyre_count,
            status = status,
            status_class = status_class,
        ));
    }

    let content = format!(
        r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">CATALOG MANAGEMENT</span>
            <h1>Brands</h1>
            <p>Manage tyre brands available in Shri Krishna Tyre House inventory.</p>
        </div>
        <div class="admin-page-actions">
            <button class="admin-primary-button" onclick="document.getElementById('addBrandForm').style.display='block'">
                + Add Brand
            </button>
        </div>
    </div>

    <div class="brand-admin-stats">
        <div class="brand-admin-stat">
            <span class="brand-admin-stat-icon">BR</span>
            <div><strong>{total}</strong><small>Total Brands</small></div>
        </div>
        <div class="brand-admin-stat">
            <span class="brand-admin-stat-icon">AC</span>
            <div><strong>{active_count}</strong><small>Active Brands</small></div>
        </div>
        <div class="brand-admin-stat">
            <span class="brand-admin-stat-icon">TY</span>
            <div><strong>{total_tyres}</strong><small>Total Tyres</small></div>
        </div>
    </div>

    <div id="addBrandForm" style="display:none;margin-bottom:20px" class="admin-panel">
        <div class="admin-panel-header af-panel-header"><div><h2>Add Brand</h2><p>Add a new tyre brand to the catalogue.</p></div></div>
        <form method="post" action="/admin/brands" class="af-form">
            <div class="af-grid af-body">
                <div class="admin-form-group">
                    <label>Brand Name</label>
                    <input name="name" type="text" required placeholder="MRF">
                </div>
                <div class="admin-form-group">
                    <label>Description</label>
                    <input name="description" type="text" placeholder="Madras Rubber Factory">
                </div>
            </div>
            <div class="af-actions af-actions-pad">
                <button type="button" onclick="document.getElementById('addBrandForm').style.display='none'" class="admin-secondary-button">Cancel</button>
                <button type="submit" class="admin-primary-button">Save Brand</button>
            </div>
        </form>
    </div>

    <div class="admin-panel">
        <div class="admin-panel-header">
            <div>
                <h2>Brand Directory</h2>
                <p>All tyre brands currently configured in the system.</p>
            </div>
        </div>
        <div class="brand-admin-table-wrapper">
            <table class="brand-admin-table">
                <thead>
                    <tr>
                        <th>Brand</th>
                        <th>Code</th>
                        <th>Tyres</th>
                        <th>Status</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>{table_rows}</tbody>
            </table>
        </div>
        <div class="brand-admin-footer">
            <span>Showing {total} of {total} brands</span>
        </div>
    </div>
</section>
"#,
        total = total,
        active_count = active_count,
        total_tyres = total_tyres,
        table_rows = table_rows,
    );

    Html(admin_page("Brands", &content))
}

pub async fn create_brand(State(state): State<AppState>, Form(form): Form<BrandForm>) -> Redirect {
    let id = format!("BR-{}", chrono::Utc::now().timestamp_millis());

    let _ =
        sqlx::query("INSERT INTO brands (id, name, description, active) VALUES ($1, $2, $3, TRUE)")
            .bind(&id)
            .bind(form.name.trim())
            .bind(form.description.trim())
            .execute(&state.pool)
            .await;

    Redirect::to("/admin/brands")
}

pub async fn delete_brand(State(state): State<AppState>, Path(id): Path<String>) -> Redirect {
    let _ = sqlx::query("DELETE FROM brands WHERE id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await;

    Redirect::to("/admin/brands")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
