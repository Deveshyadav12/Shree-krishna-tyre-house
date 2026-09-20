use axum::{
    extract::{Path, State},
    response::Html,
};
use sqlx::Row;

use super::layout::page;
use crate::AppState;

pub async fn tyres(State(state): State<AppState>) -> Html<String> {
    let rows = sqlx::query(
        "SELECT id, brand, name, size, tyre_type, price, stock FROM inventory ORDER BY brand, name",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let total = rows.len();
    let mut product_cards = String::new();

    for row in &rows {
        let id: String = row.get("id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let size: String = row.get("size");
        let tyre_type: String = row.get("tyre_type");
        let price: i64 = row.get("price");
        let stock: i32 = row.get("stock");

        let (avail_label, avail_color) = if stock > 0 {
            ("In Stock", "#16a34a")
        } else {
            ("Out of Stock", "#dc2626")
        };

        product_cards.push_str(&format!(
            r#"<article class="product-card">
                <div class="product-image">
                    <div class="tyre-placeholder">TYRE</div>
                </div>
                <div class="product-content">
                    <div class="product-brand">{brand}</div>
                    <h3>{name}</h3>
                    <p>{tyre_type}</p>
                    <div class="product-meta">
                        <span>Size: {size}</span>
                        <strong>&#8377;{price}</strong>
                    </div>
                    <div style="margin-top:10px;color:{avail_color};font-size:12px;font-weight:700;">
                        &#9679; {avail_label}
                    </div>
                    <a href="/tyres/{id}" class="product-button">View Tyre Details &rarr;</a>
                </div>
            </article>"#,
            brand = esc(&brand),
            name = esc(&name),
            tyre_type = esc(&tyre_type),
            size = esc(&size),
            price = price,
            avail_color = avail_color,
            avail_label = avail_label,
            id = esc(&id),
        ));
    }

    let empty_msg = if total == 0 {
        r#"<div style="grid-column:1/-1;text-align:center;padding:60px 20px;color:#64748b;">
            <p style="font-size:18px;font-weight:700;">No tyres available yet.</p>
            <p style="margin-top:8px;font-size:14px;">Check back soon or contact us for availability.</p>
        </div>"#
    } else {
        ""
    };

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">TYRE CATALOGUE</span>
        <h1>Find The Right Tyres</h1>
        <p>Explore tyre options for cars, SUVs and motorcycles.</p>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="catalog-header">
            <div>
                <span class="eyebrow">OUR PRODUCTS</span>
                <h2>Available Tyres</h2>
            </div>
            <span>{total} tyre products</span>
        </div>
        <div class="product-grid">
            {product_cards}
            {empty_msg}
        </div>
    </div>
</section>

<section class="contact-strip">
    <div class="container">
        <div class="contact-strip-content">
            <div>
                <span class="eyebrow">NEED HELP?</span>
                <h2>Not Sure Which Tyre To Choose?</h2>
                <p>Contact Shri Krishna Tyre House for tyre information and service support.</p>
            </div>
            <a href="/contact" class="btn btn-light">Contact Us &rarr;</a>
        </div>
    </div>
</section>
"#,
        total = total,
        product_cards = product_cards,
        empty_msg = empty_msg,
    );

    Html(page("Tyres", &content))
}

pub async fn tyre_detail(State(state): State<AppState>, Path(id): Path<String>) -> Html<String> {
    let row = sqlx::query(
        "SELECT id, brand, name, size, tyre_type, price, stock FROM inventory WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let Some(row) = row else {
        let content = r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">TYRE NOT FOUND</span>
        <h1>Tyre Not Found</h1>
        <p>The tyre you are looking for does not exist or has been removed.</p>
    </div>
</section>
<section class="section">
    <div class="container" style="text-align:center;padding:40px 20px;">
        <a href="/tyres" class="btn btn-primary">Browse All Tyres</a>
    </div>
</section>"#;
        return Html(page("Tyre Not Found", content));
    };

    let brand: String = row.get("brand");
    let name: String = row.get("name");
    let size: String = row.get("size");
    let tyre_type: String = row.get("tyre_type");
    let price: i64 = row.get("price");
    let stock: i32 = row.get("stock");

    let (avail_label, avail_color) = if stock > 0 {
        ("In Stock", "#16a34a")
    } else {
        ("Out of Stock", "#dc2626")
    };

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">TYRE DETAILS</span>
        <h1>{name}</h1>
        <p>{brand} &mdash; {tyre_type}</p>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="product-detail">
            <div class="product-detail-image">
                <div class="tyre-placeholder">TYRE</div>
            </div>
            <div class="product-detail-info">
                <span class="eyebrow">{brand}</span>
                <h1>{name}</h1>
                <p class="product-detail-description">
                    Quality {tyre_type} tyre from {brand}. Available at Shri Krishna Tyre House.
                </p>
                <div class="product-detail-price">&#8377;{price}</div>
                <div class="product-specs">
                    <div class="spec-item">
                        <small>Tyre Size</small>
                        <strong>{size}</strong>
                    </div>
                    <div class="spec-item">
                        <small>Vehicle Type</small>
                        <strong>{tyre_type}</strong>
                    </div>
                    <div class="spec-item">
                        <small>Brand</small>
                        <strong>{brand}</strong>
                    </div>
                    <div class="spec-item">
                        <small>Availability</small>
                        <strong style="color:{avail_color};">{avail_label}</strong>
                    </div>
                </div>
                <div class="hero-actions">
                    <a href="/contact" class="btn btn-primary">Contact Store</a>
                    <a href="/tyres" class="btn" style="border:1px solid #e2e8f0;background:#fff;">
                        Back To Tyres
                    </a>
                </div>
            </div>
        </div>
    </div>
</section>
"#,
        brand = esc(&brand),
        name = esc(&name),
        size = esc(&size),
        tyre_type = esc(&tyre_type),
        price = price,
        avail_label = avail_label,
        avail_color = avail_color,
    );

    Html(page(&format!("{} | Tyres", esc(&name)), &content))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
