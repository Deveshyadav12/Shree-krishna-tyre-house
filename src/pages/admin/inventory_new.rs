use axum::{
    extract::{Form, State},
    response::{Html, Redirect},
};
use serde::Deserialize;
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

#[derive(Deserialize)]
pub struct InventoryForm {
    pub brand: String,
    pub name: String,
    pub size: String,
    pub tyre_type: String,
    pub price: String,
    pub stock: String,
    pub low_stock_limit: String,
}

pub async fn inventory_new(State(state): State<AppState>) -> Html<String> {
    let brands = sqlx::query("SELECT name FROM brands WHERE active = TRUE ORDER BY name")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let brand_options = brands
        .iter()
        .map(|row| {
            let name: String = row.get("name");
            format!(r#"<option>{}</option>"#, escape_html(&name))
        })
        .collect::<Vec<_>>()
        .join("\n");

    let fallback_brands = if brand_options.is_empty() {
        r#"<option>MRF</option><option>CEAT</option><option>Apollo</option>
           <option>JK Tyre</option><option>Michelin</option><option>Bridgestone</option>
           <option>Goodyear</option><option>Yokohama</option><option>Continental</option>
           <option>Other</option>"#
            .to_string()
    } else {
        brand_options
    };

    let content = format!(
        r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">INVENTORY MANAGEMENT</span>
            <h1>Add Tyre</h1>
            <p>Add a new tyre product to your inventory.</p>
        </div>
        <div class="admin-page-actions">
            <a href="/admin/inventory" class="admin-secondary-button">Back to Inventory</a>
        </div>
    </div>

    <form method="post" action="/admin/inventory/new" class="inv-new-form">

        <div class="admin-panel" style="margin-bottom:20px">
            <div class="admin-panel-header" style="border-bottom:1px solid #e5e7eb">
                <div>
                    <h2>Product Information</h2>
                    <p>Enter the tyre product details.</p>
                </div>
            </div>
            <div class="inv-form-grid">
                <div class="admin-form-group">
                    <label for="brand">Brand</label>
                    <select id="brand" name="brand" required>
                        <option value="">Select brand</option>
                        {brand_options}
                    </select>
                </div>
                <div class="admin-form-group">
                    <label for="name">Tyre Name</label>
                    <input id="name" name="name" type="text" placeholder="Example: MRF ZVTV" required>
                </div>
                <div class="admin-form-group">
                    <label for="size">Tyre Size</label>
                    <input id="size" name="size" type="text" placeholder="Example: 185/65 R15" required>
                </div>
                <div class="admin-form-group">
                    <label for="tyre_type">Tyre Type</label>
                    <select id="tyre_type" name="tyre_type" required>
                        <option value="">Select type</option>
                        <option>Tubeless</option>
                        <option>Tube Type</option>
                        <option>Run Flat</option>
                        <option>Other</option>
                    </select>
                </div>
            </div>
        </div>

        <div class="admin-panel" style="margin-bottom:20px">
            <div class="admin-panel-header" style="border-bottom:1px solid #e5e7eb">
                <div>
                    <h2>Pricing &amp; Stock</h2>
                    <p>Set selling price and opening stock.</p>
                </div>
            </div>
            <div class="inv-form-grid">
                <div class="admin-form-group">
                    <label for="price">Selling Price</label>
                    <div class="input-prefix">
                        <span>&#8377;</span>
                        <input id="price" name="price" type="number" min="0" step="1" placeholder="6250" required>
                    </div>
                </div>
                <div class="admin-form-group">
                    <label for="stock">Initial Stock</label>
                    <input id="stock" name="stock" type="number" min="0" step="1" placeholder="20" required>
                </div>
                <div class="admin-form-group">
                    <label for="low_stock_limit">Low Stock Limit</label>
                    <input id="low_stock_limit" name="low_stock_limit" type="number" min="0" step="1" value="5" required>
                </div>
            </div>
        </div>

        <div class="inv-form-actions">
            <a href="/admin/inventory" class="admin-secondary-button">Cancel</a>
            <button type="submit" class="admin-primary-button">Save Tyre</button>
        </div>

    </form>
</section>
"#,
        brand_options = fallback_brands,
    );

    Html(admin_page("Add Tyre", &content))
}

pub async fn create_inventory(
    State(state): State<AppState>,
    Form(form): Form<InventoryForm>,
) -> Redirect {
    let price = form.price.trim().parse::<i64>().unwrap_or(0);
    let stock = form.stock.trim().parse::<i32>().unwrap_or(0);
    let low_stock_limit = form.low_stock_limit.trim().parse::<i32>().unwrap_or(5);

    let id = format!("TYR-{}", chrono::Utc::now().timestamp_millis());

    let result = sqlx::query(
        r#"INSERT INTO inventory (id, brand, name, size, tyre_type, price, stock, low_stock_limit)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
    )
    .bind(&id)
    .bind(form.brand.trim())
    .bind(form.name.trim())
    .bind(form.size.trim())
    .bind(form.tyre_type.trim())
    .bind(price)
    .bind(stock)
    .bind(low_stock_limit)
    .execute(&state.pool)
    .await;

    if let Err(e) = result {
        eprintln!("Create inventory error: {}", e);
        return Redirect::to("/admin/inventory");
    }

    if stock > 0 {
        let mv_id = format!("STK-{}", chrono::Utc::now().timestamp_millis());
        let _ = sqlx::query(
            r#"INSERT INTO stock_movements
               (id, tyre_id, tyre_name, movement_type, quantity, stock_before, stock_after, reference, notes)
               VALUES ($1, $2, $3, 'Stock In', $4, 0, $4, 'NEW TYRE', 'Opening stock added.')"#,
        )
        .bind(&mv_id)
        .bind(&id)
        .bind(form.name.trim())
        .bind(stock)
        .execute(&state.pool)
        .await;
    }

    Redirect::to("/admin/inventory")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
