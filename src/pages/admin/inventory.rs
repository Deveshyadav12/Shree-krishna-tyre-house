use axum::{
    extract::{Form, Path, State},
    response::{Html, Redirect},
};
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

#[derive(serde::Deserialize)]
pub struct InventoryForm {
    pub name: String,
    pub brand: String,
    pub size: String,
    pub tyre_type: String,
    pub price: i64,
    pub stock: i32,
    pub low_stock_limit: i32,
}

#[derive(serde::Deserialize)]
pub struct StockForm {
    pub movement_type: String,
    pub quantity: i32,
    pub notes: String,
}

pub async fn inventory(State(state): State<AppState>) -> Html<String> {
    let rows = match sqlx::query(
        "SELECT id, name, brand, size, tyre_type, price, stock, low_stock_limit
         FROM inventory ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Inventory query failed: {}", e);
            return Html(admin_page(
                "Tyre Inventory",
                &error_html("Unable to load inventory."),
            ));
        }
    };

    let total_tyres = rows.len();
    let total_stock: i32 = rows.iter().map(|r| r.get::<i32, _>("stock")).sum();
    let low_stock: usize = rows
        .iter()
        .filter(|r| {
            let s: i32 = r.get("stock");
            let l: i32 = r.get("low_stock_limit");
            s > 0 && s <= l
        })
        .count();
    let out_of_stock: usize = rows
        .iter()
        .filter(|r| r.get::<i32, _>("stock") == 0)
        .count();

    let mut table_rows = String::new();
    for row in &rows {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let brand: String = row.get("brand");
        let size: String = row.get("size");
        let tyre_type: String = row.get("tyre_type");
        let price: i64 = row.get("price");
        let stock: i32 = row.get("stock");
        let low_stock_limit: i32 = row.get("low_stock_limit");

        let (badge_class, status_label) = if stock == 0 {
            ("stock-badge stock-out", "Out of Stock")
        } else if stock <= low_stock_limit {
            ("stock-badge stock-low", "Low Stock")
        } else {
            ("stock-badge stock-good", "In Stock")
        };

        table_rows.push_str(&format!(
            r#"<tr>
    <td>
        <div class="brand-table-name">
            <div class="brand-logo-box">{abbr}</div>
            <div>
                <strong>{name}</strong>
                <small>{tyre_type}</small>
            </div>
        </div>
    </td>
    <td>{size}</td>
    <td>{brand}</td>
    <td>&#8377;{price}</td>
    <td><strong>{stock}</strong></td>
    <td><span class="{badge_class}">{status_label}</span></td>
    <td>
        <div class="brand-table-actions">
            <a href="/admin/inventory/{id}" class="brand-action">View</a>
            <a href="/admin/inventory/{id}/edit" class="brand-action edit">Edit</a>
        </div>
    </td>
</tr>"#,
            abbr = name.chars().take(2).collect::<String>().to_uppercase(),
            name = escape_html(&name),
            tyre_type = escape_html(&tyre_type),
            size = escape_html(&size),
            brand = escape_html(&brand),
            price = price,
            stock = stock,
            badge_class = badge_class,
            status_label = status_label,
            id = id,
        ));
    }

    let content = format!(
        r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">INVENTORY MANAGEMENT</span>
            <h1>Tyre Inventory</h1>
            <p>Manage tyre stock, pricing and inventory levels.</p>
        </div>
        <div class="admin-page-actions">
            <a href="/admin/inventory/history" class="admin-secondary-button">Stock History</a>
            <a href="/admin/inventory/new" class="admin-primary-button">+ Add Tyre</a>
        </div>
    </div>

    <div class="inventory-stat-grid">
        <div class="inventory-stat-card">
            <span>TOTAL TYRES</span>
            <strong>{total_tyres}</strong>
        </div>
        <div class="inventory-stat-card">
            <span>TOTAL STOCK</span>
            <strong>{total_stock}</strong>
        </div>
        <div class="inventory-stat-card">
            <span>LOW STOCK</span>
            <strong>{low_stock}</strong>
        </div>
        <div class="inventory-stat-card">
            <span>OUT OF STOCK</span>
            <strong>{out_of_stock}</strong>
        </div>
    </div>

    <div class="admin-panel">
        <div class="brand-admin-table-wrapper">
            <table class="brand-admin-table">
                <thead>
                    <tr>
                        <th>TYRE NAME</th>
                        <th>SIZE</th>
                        <th>BRAND</th>
                        <th>PRICE</th>
                        <th>STOCK</th>
                        <th>STATUS</th>
                        <th>ACTIONS</th>
                    </tr>
                </thead>
                <tbody>{table_rows}</tbody>
            </table>
        </div>
        <div class="brand-admin-footer">
            <span>Showing {total_tyres} tyres</span>
        </div>
    </div>
</section>
"#,
        total_tyres = total_tyres,
        total_stock = total_stock,
        low_stock = low_stock,
        out_of_stock = out_of_stock,
        table_rows = table_rows,
    );

    Html(admin_page("Tyre Inventory", &content))
}

pub async fn inventory_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Html<String> {
    let item = match sqlx::query(
        "SELECT id, name, brand, size, tyre_type, price, stock, low_stock_limit
         FROM inventory WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(item) => item,
        Err(e) => {
            eprintln!("Inventory detail query failed: {}", e);
            return Html(admin_page(
                "Tyre Inventory",
                &error_html("Unable to load inventory item."),
            ));
        }
    };

    let Some(item) = item else {
        return Html(admin_page(
            "Tyre Inventory",
            &error_html("Inventory item not found."),
        ));
    };

    let item_id: String = item.get("id");
    let name: String = item.get("name");
    let brand: String = item.get("brand");
    let size: String = item.get("size");
    let tyre_type: String = item.get("tyre_type");
    let price: i64 = item.get("price");
    let stock: i32 = item.get("stock");
    let low_stock_limit: i32 = item.get("low_stock_limit");
    let value = price * stock as i64;

    let (badge_class, status_label) = if stock == 0 {
        ("stock-badge stock-out", "Out of Stock")
    } else if stock <= low_stock_limit {
        ("stock-badge stock-low", "Low Stock")
    } else {
        ("stock-badge stock-good", "In Stock")
    };

    let content = format!(
        r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">TYRE INVENTORY</span>
            <h1>{name}</h1>
            <p>{brand} &middot; {size} &middot; {tyre_type}</p>
        </div>
        <div class="admin-page-actions">
            <a href="/admin/inventory/{item_id}/edit" class="admin-primary-button">Edit Tyre</a>
            <a href="/admin/inventory" class="admin-secondary-button">Back</a>
        </div>
    </div>

    <div class="inventory-detail-grid">
        <div><span>Brand</span><strong>{brand}</strong></div>
        <div><span>Size</span><strong>{size}</strong></div>
        <div><span>Type</span><strong>{tyre_type}</strong></div>
        <div><span>Price</span><strong>&#8377;{price}</strong></div>
        <div><span>Current Stock</span><strong>{stock}</strong></div>
        <div><span>Low Stock Limit</span><strong>{low_stock_limit}</strong></div>
        <div><span>Status</span><strong><span class="{badge_class}">{status_label}</span></strong></div>
        <div><span>Inventory Value</span><strong>&#8377;{value}</strong></div>
        <div><span>Item ID</span><strong>{item_id}</strong></div>
    </div>

    <div class="admin-panel stock-adjustment-panel">
        <div class="admin-panel-header">
            <div>
                <h2>Stock Adjustment</h2>
                <p>Update stock levels for this tyre.</p>
            </div>
        </div>
        <form method="post" action="/admin/inventory/{item_id}/adjust" style="padding:22px">
            <div class="create-job-form-grid">
                <div class="admin-form-group">
                    <label>Movement Type</label>
                    <select name="movement_type" required>
                        <option value="Stock In">Stock In</option>
                        <option value="Stock Out">Stock Out</option>
                        <option value="Adjustment">Adjustment</option>
                    </select>
                </div>
                <div class="admin-form-group">
                    <label>Quantity</label>
                    <input type="number" name="quantity" min="1" required placeholder="1">
                </div>
                <div class="admin-form-group" style="grid-column:1/-1">
                    <label>Notes</label>
                    <textarea name="notes" class="admin-form-textarea" rows="3" placeholder="Optional notes..."></textarea>
                </div>
            </div>
            <div class="stock-adjustment-actions">
                <button type="submit" class="admin-primary-button">Update Stock</button>
            </div>
        </form>
    </div>
</section>
"#,
        name = escape_html(&name),
        brand = escape_html(&brand),
        size = escape_html(&size),
        tyre_type = escape_html(&tyre_type),
        price = price,
        stock = stock,
        low_stock_limit = low_stock_limit,
        badge_class = badge_class,
        status_label = status_label,
        value = value,
        item_id = item_id,
    );

    Html(admin_page("Tyre Inventory", &content))
}

pub async fn inventory_edit(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    let item = match sqlx::query(
        "SELECT id, name, brand, size, tyre_type, price, stock, low_stock_limit
         FROM inventory WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(item) => item,
        Err(e) => {
            eprintln!("Inventory edit query failed: {}", e);
            return Html(admin_page(
                "Tyre Inventory",
                &error_html("Unable to load inventory item."),
            ));
        }
    };

    let Some(item) = item else {
        return Html(admin_page(
            "Tyre Inventory",
            &error_html("Inventory item not found."),
        ));
    };

    let item_id: String = item.get("id");
    let name: String = item.get("name");
    let brand: String = item.get("brand");
    let size: String = item.get("size");
    let tyre_type: String = item.get("tyre_type");
    let price: i64 = item.get("price");
    let stock: i32 = item.get("stock");
    let low_stock_limit: i32 = item.get("low_stock_limit");

    let content = format!(
        r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">TYRE INVENTORY</span>
            <h1>Edit Tyre</h1>
            <p>Update inventory information for {name}.</p>
        </div>
        <a href="/admin/inventory/{item_id}" class="admin-secondary-button">Cancel</a>
    </div>

    <div class="admin-panel">
        <div class="admin-panel-header">
            <div>
                <h2>Tyre Details</h2>
                <p>Update the tyre product information below.</p>
            </div>
        </div>
        <form method="post" action="/admin/inventory/{item_id}/edit" style="padding:22px">
            <div class="create-job-form-grid">
                <div class="admin-form-group">
                    <label>Tyre Name</label>
                    <input type="text" name="name" value="{name}" required>
                </div>
                <div class="admin-form-group">
                    <label>Brand</label>
                    <input type="text" name="brand" value="{brand}" required>
                </div>
                <div class="admin-form-group">
                    <label>Size</label>
                    <input type="text" name="size" value="{size}" required>
                </div>
                <div class="admin-form-group">
                    <label>Tyre Type</label>
                    <input type="text" name="tyre_type" value="{tyre_type}" required>
                </div>
                <div class="admin-form-group">
                    <label>Price (&#8377;)</label>
                    <input type="number" name="price" value="{price}" min="0" required>
                </div>
                <div class="admin-form-group">
                    <label>Stock</label>
                    <input type="number" name="stock" value="{stock}" min="0" required>
                </div>
                <div class="admin-form-group">
                    <label>Low Stock Limit</label>
                    <input type="number" name="low_stock_limit" value="{low_stock_limit}" min="0" required>
                </div>
            </div>
            <div class="stock-adjustment-actions" style="margin-top:22px">
                <button type="submit" class="admin-primary-button">Save Changes</button>
            </div>
        </form>
    </div>
</section>
"#,
        name = escape_html(&name),
        brand = escape_html(&brand),
        size = escape_html(&size),
        tyre_type = escape_html(&tyre_type),
        price = price,
        stock = stock,
        low_stock_limit = low_stock_limit,
        item_id = item_id,
    );

    Html(admin_page("Tyre Inventory", &content))
}

pub async fn update_inventory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<InventoryForm>,
) -> Redirect {
    let result = sqlx::query(
        "UPDATE inventory
         SET name = $1, brand = $2, size = $3, tyre_type = $4,
             price = $5, stock = $6, low_stock_limit = $7, updated_at = NOW()
         WHERE id = $8",
    )
    .bind(&form.name)
    .bind(&form.brand)
    .bind(&form.size)
    .bind(&form.tyre_type)
    .bind(form.price)
    .bind(form.stock)
    .bind(form.low_stock_limit)
    .bind(&id)
    .execute(&state.pool)
    .await;

    if let Err(e) = result {
        eprintln!("Inventory update failed: {}", e);
    }

    Redirect::to(&format!("/admin/inventory/{}", id))
}

pub async fn adjust_stock(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<StockForm>,
) -> Redirect {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Transaction start failed: {}", e);
            return Redirect::to(&format!("/admin/inventory/{}", id));
        }
    };

    let item = match sqlx::query("SELECT name, stock FROM inventory WHERE id = $1 FOR UPDATE")
        .bind(&id)
        .fetch_optional(&mut *tx)
        .await
    {
        Ok(item) => item,
        Err(e) => {
            eprintln!("Stock lookup failed: {}", e);
            return Redirect::to(&format!("/admin/inventory/{}", id));
        }
    };

    let Some(item) = item else {
        return Redirect::to("/admin/inventory");
    };

    let item_name: String = item.get("name");
    let stock_before: i32 = item.get("stock");

    let stock_after = match form.movement_type.as_str() {
        "Stock In" => stock_before + form.quantity,
        "Stock Out" => (stock_before - form.quantity).max(0),
        "Adjustment" => form.quantity,
        _ => stock_before,
    };

    let _ = sqlx::query("UPDATE inventory SET stock = $1, updated_at = NOW() WHERE id = $2")
        .bind(stock_after)
        .bind(&id)
        .execute(&mut *tx)
        .await;

    let movement_id = format!("STK-{}", chrono::Utc::now().timestamp_millis());

    let _ = sqlx::query(
        "INSERT INTO stock_movements
         (id, tyre_id, tyre_name, movement_type, quantity, stock_before, stock_after, reference, notes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(&movement_id)
    .bind(&id)
    .bind(&item_name)
    .bind(&form.movement_type)
    .bind(form.quantity)
    .bind(stock_before)
    .bind(stock_after)
    .bind("")
    .bind(&form.notes)
    .execute(&mut *tx)
    .await;

    if let Err(e) = tx.commit().await {
        eprintln!("Stock transaction commit failed: {}", e);
    }

    Redirect::to(&format!("/admin/inventory/{}", id))
}

fn error_html(message: &str) -> String {
    format!(
        r#"<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">TYRE INVENTORY</span>
            <h1>Error</h1>
        </div>
        <a href="/admin/inventory" class="admin-secondary-button">Back to Inventory</a>
    </div>
    <div class="admin-panel">
        <div class="admin-panel-header">
            <div><p>{}</p></div>
        </div>
    </div>
</section>"#,
        escape_html(message)
    )
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
