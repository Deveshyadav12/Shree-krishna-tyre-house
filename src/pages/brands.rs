use axum::{
    extract::{Path, State},
    response::Html,
};
use sqlx::Row;

use super::layout::page;
use crate::AppState;

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub async fn brands(State(state): State<AppState>) -> Html<String> {
    let rows = sqlx::query(
        r#"SELECT b.id, b.name, b.description,
                  (SELECT COUNT(*) FROM inventory WHERE brand = b.name) AS tyre_count
           FROM brands b
           WHERE b.active = TRUE
           ORDER BY b.name"#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut brand_cards = String::new();

    for row in &rows {
        let name: String = row.get("name");
        let description: String = row.get("description");
        let tyre_count: i64 = row.get("tyre_count");
        let slug = slugify(&name);

        let count_label = if tyre_count == 1 {
            "1 tyre".to_string()
        } else {
            format!("{tyre_count} tyres")
        };

        brand_cards.push_str(&format!(
            r#"<article class="brand-showcase">
                <div class="brand-logo-large">{abbr}</div>
                <span class="eyebrow" style="margin-top:24px;">TYRE BRAND</span>
                <h2>{name}</h2>
                <p>{description}</p>
                <p style="margin-top:8px;color:#64748b;font-size:13px;">{count_label} available</p>
                <a href="/brands/{slug}">Explore {name} Tyres &rarr;</a>
            </article>"#,
            abbr = esc(&name.chars().take(2).collect::<String>().to_uppercase()),
            name = esc(&name),
            description = if description.is_empty() {
                format!("{} tyres available at Shri Krishna Tyre House.", esc(&name))
            } else {
                esc(&description)
            },
            count_label = count_label,
            slug = slug,
        ));
    }

    let empty_msg = if rows.is_empty() {
        r#"<div style="text-align:center;padding:60px 20px;color:#64748b;">
            <p style="font-size:18px;font-weight:700;">No brands listed yet.</p>
            <p style="margin-top:8px;font-size:14px;">Contact us for brand availability.</p>
        </div>"#
    } else {
        ""
    };

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">OUR BRANDS</span>
        <h1>Trusted Tyre Brands</h1>
        <p>Explore tyre options from leading tyre brands available through Shri Krishna Tyre House.</p>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="section-heading">
            <div>
                <span class="eyebrow">BRANDS</span>
                <h2>Explore Our Tyre Brands</h2>
                <p>Select a brand to explore its tyre options.</p>
            </div>
        </div>
        <div class="brand-showcase-grid">
            {brand_cards}
            {empty_msg}
        </div>
    </div>
</section>

<section class="contact-strip">
    <div class="container">
        <div class="contact-strip-content">
            <div>
                <span class="eyebrow">NEED HELP?</span>
                <h2>Need Help Choosing A Tyre?</h2>
                <p>Contact Shri Krishna Tyre House for tyre information and support.</p>
            </div>
            <a href="/contact" class="btn btn-light">Contact Us &rarr;</a>
        </div>
    </div>
</section>
"#,
        brand_cards = brand_cards,
        empty_msg = empty_msg,
    );

    Html(page("Brands", &content))
}

pub async fn brand_detail(State(state): State<AppState>, Path(slug): Path<String>) -> Html<String> {
    // Load all active brands and find the one whose slugified name matches
    let brand_rows =
        sqlx::query("SELECT id, name, description FROM brands WHERE active = TRUE ORDER BY name")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let matched = brand_rows
        .iter()
        .find(|r| slugify(&r.get::<String, _>("name")) == slug);

    let Some(brand_row) = matched else {
        let content = r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">BRAND NOT FOUND</span>
        <h1>Brand Not Found</h1>
        <p>The brand you are looking for does not exist or is no longer active.</p>
    </div>
</section>
<section class="section">
    <div class="container" style="text-align:center;padding:40px 20px;">
        <a href="/brands" class="btn btn-primary">Browse All Brands</a>
    </div>
</section>"#;
        return Html(page("Brand Not Found", content));
    };

    let brand_name: String = brand_row.get("name");
    let brand_desc: String = brand_row.get("description");

    // Load tyres for this brand
    let tyre_rows = sqlx::query(
        "SELECT id, name, size, tyre_type, price, stock FROM inventory WHERE brand = $1 ORDER BY name",
    )
    .bind(&brand_name)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut product_cards = String::new();

    for row in &tyre_rows {
        let id: String = row.get("id");
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
                    <div class="product-brand">{brand_name}</div>
                    <h3>{name}</h3>
                    <p>{tyre_type}</p>
                    <div class="product-meta">
                        <span>{size}</span>
                        <strong>&#8377;{price}</strong>
                    </div>
                    <div style="margin-top:10px;color:{avail_color};font-size:12px;font-weight:700;">
                        &#9679; {avail_label}
                    </div>
                    <a href="/tyres/{id}" class="product-button">View Details &rarr;</a>
                </div>
            </article>"#,
            brand_name = esc(&brand_name),
            name = esc(&name),
            tyre_type = esc(&tyre_type),
            size = esc(&size),
            price = price,
            avail_color = avail_color,
            avail_label = avail_label,
            id = esc(&id),
        ));
    }

    let empty_msg = if tyre_rows.is_empty() {
        format!(
            r#"<div style="grid-column:1/-1;text-align:center;padding:60px 20px;color:#64748b;">
                <p style="font-size:18px;font-weight:700;">No {brand} tyres listed yet.</p>
                <p style="margin-top:8px;font-size:14px;">Contact us for availability.</p>
            </div>"#,
            brand = esc(&brand_name)
        )
    } else {
        String::new()
    };

    let description = if brand_desc.is_empty() {
        format!(
            "{} tyres available at Shri Krishna Tyre House.",
            esc(&brand_name)
        )
    } else {
        esc(&brand_desc)
    };

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">TYRE BRAND</span>
        <h1>{brand_name}</h1>
        <p>{description}</p>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="catalog-header">
            <div>
                <span class="eyebrow">{brand_name} TYRES</span>
                <h2>Available Options</h2>
            </div>
            <a href="/tyres" class="btn btn-primary">View All Tyres</a>
        </div>
        <div class="product-grid">
            {product_cards}
            {empty_msg}
        </div>
    </div>
</section>

<section class="section section-dark">
    <div class="container">
        <div class="section-heading">
            <div>
                <span class="eyebrow">SHRI KRISHNA TYRE HOUSE</span>
                <h2>Need Help Selecting A Tyre?</h2>
                <p>Contact our store for tyre information, availability and service enquiries.</p>
            </div>
            <a href="/contact" class="btn btn-primary">Contact Store</a>
        </div>
    </div>
</section>
"#,
        brand_name = esc(&brand_name),
        description = description,
        product_cards = product_cards,
        empty_msg = empty_msg,
    );

    Html(page(&format!("{} Tyres", esc(&brand_name)), &content))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
