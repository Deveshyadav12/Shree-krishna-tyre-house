use axum::{extract::State, response::Html};
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

pub async fn stock_history(State(state): State<AppState>) -> Html<String> {
    let movements = sqlx::query(
        r#"SELECT id, tyre_id, tyre_name, movement_type, quantity, stock_before, stock_after, reference, notes, created_at
           FROM stock_movements ORDER BY created_at DESC"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut rows = String::new();
    for row in &movements {
        let tyre_name: String = row.get("tyre_name");
        let tyre_id: String = row.get("tyre_id");
        let movement_type: String = row.get("movement_type");
        let quantity: i32 = row.get("quantity");
        let stock_before: i32 = row.get("stock_before");
        let stock_after: i32 = row.get("stock_after");
        let reference: String = row.get("reference");
        let notes: String = row.get("notes");

        let quantity_class = if quantity >= 0 {
            "stock-in"
        } else {
            "stock-out"
        };
        let quantity_text = if quantity >= 0 {
            format!("+{}", quantity)
        } else {
            quantity.to_string()
        };

        rows.push_str(&format!(
            r#"<tr>
    <td>
        <strong>{tyre_name}</strong>
        <span class="table-sub">{tyre_id}</span>
    </td>
    <td>{movement_type}</td>
    <td><span class="stock-movement {quantity_class}">{quantity_text}</span></td>
    <td>{reference}</td>
    <td>{stock_before} &rarr; {stock_after}</td>
    <td>{notes}</td>
</tr>"#,
            tyre_name = escape_html(&tyre_name),
            tyre_id = escape_html(&tyre_id),
            movement_type = escape_html(&movement_type),
            quantity_class = quantity_class,
            quantity_text = quantity_text,
            reference = escape_html(&reference),
            stock_before = stock_before,
            stock_after = stock_after,
            notes = escape_html(&notes),
        ));
    }

    let empty = if movements.is_empty() {
        r#"<div class="empty-state"><p>No stock movements recorded yet.</p></div>"#
    } else {
        ""
    };

    let content = format!(
        r#"
<div class="page-head">
    <div>
        <div class="eyebrow">Inventory</div>
        <h1>Stock History</h1>
        <p>Track every stock addition, sale and adjustment.</p>
    </div>
    <a href="/admin/inventory" class="btn btn-light">Back to Inventory</a>
</div>

<section class="panel">
    <div class="panel-head">
        <div>
            <h2>Stock Movements</h2>
            <p>Complete inventory movement history.</p>
        </div>
    </div>
    <div class="table-wrap">
        <table class="admin-table">
            <thead>
                <tr>
                    <th>Tyre</th>
                    <th>Movement</th>
                    <th>Quantity</th>
                    <th>Reference</th>
                    <th>Stock</th>
                    <th>Notes</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>
    </div>
    {empty}
</section>
"#,
        rows = rows,
        empty = empty,
    );

    Html(admin_page("Stock History", &content))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
