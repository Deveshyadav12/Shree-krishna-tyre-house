use axum::{extract::State, response::Html};
use sqlx::Row;

use super::layout::page;
use crate::AppState;

pub async fn home(State(state): State<AppState>) -> Html<String> {
    // Featured tyres — latest 3 in stock
    let tyre_rows = sqlx::query(
        "SELECT id, brand, name, size, price FROM inventory WHERE stock > 0 ORDER BY created_at DESC LIMIT 3",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut featured_tyres = String::new();
    for row in &tyre_rows {
        let id: String = row.get("id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let size: String = row.get("size");
        let price: i64 = row.get("price");
        featured_tyres.push_str(&format!(
            r#"<article class="product-card">
                <div class="product-image">
                    <div class="tyre-placeholder">TYRE</div>
                </div>
                <div class="product-content">
                    <span class="product-brand">{brand}</span>
                    <h3>{name}</h3>
                    <p>In Stock</p>
                    <div class="product-meta">
                        <span>{size}</span>
                        <strong>&#8377;{price}</strong>
                    </div>
                    <a href="/tyres/{id}" class="product-button">View Details</a>
                </div>
            </article>"#,
            brand = esc(&brand),
            name = esc(&name),
            size = esc(&size),
            price = price,
            id = esc(&id),
        ));
    }

    // Fallback if no tyres yet
    if featured_tyres.is_empty() {
        featured_tyres =
            r#"<div style="grid-column:1/-1;text-align:center;padding:40px;color:#64748b;">
            <p>Tyre catalogue coming soon. <a href="/contact">Contact us</a> for availability.</p>
        </div>"#
                .to_string();
    }

    // Active brands — up to 6
    let brand_rows =
        sqlx::query("SELECT name FROM brands WHERE active = TRUE ORDER BY name LIMIT 6")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let mut brand_boxes = String::new();
    for row in &brand_rows {
        let name: String = row.get("name");
        brand_boxes.push_str(&format!(
            r#"<div class="brand-box">{}</div>"#,
            esc(&name.to_uppercase())
        ));
    }

    // Fallback brand boxes if none in DB
    if brand_boxes.is_empty() {
        brand_boxes = r#"<div class="brand-box">MRF</div>
            <div class="brand-box">CEAT</div>
            <div class="brand-box">APOLLO</div>
            <div class="brand-box">JK TYRE</div>"#
            .to_string();
    }

    // Active services — up to 4 for home page preview
    let service_rows = sqlx::query(
        "SELECT name, description FROM services WHERE active = TRUE ORDER BY name LIMIT 4",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let service_icons = ["◉", "◎", "◌", "+"];
    let mut service_cards = String::new();
    for (i, row) in service_rows.iter().enumerate() {
        let name: String = row.get("name");
        let description: String = row.get("description");
        let icon = service_icons[i % service_icons.len()];
        service_cards.push_str(&format!(
            r#"<article class="service-card">
                <div class="service-icon">{icon}</div>
                <h3>{name}</h3>
                <p>{description}</p>
                <a href="/services">Learn More &rarr;</a>
            </article>"#,
            icon = icon,
            name = esc(&name),
            description = if description.is_empty() {
                format!("Professional {} service.", esc(&name))
            } else {
                esc(&description)
            },
        ));
    }

    // Fallback services if none in DB
    if service_cards.is_empty() {
        service_cards = r#"
            <article class="service-card">
                <div class="service-icon">◉</div>
                <h3>Wheel Alignment</h3>
                <p>Accurate alignment for better handling and tyre life.</p>
                <a href="/services">Learn More &rarr;</a>
            </article>
            <article class="service-card">
                <div class="service-icon">◎</div>
                <h3>Wheel Balancing</h3>
                <p>Reduce vibration and improve driving comfort.</p>
                <a href="/services">Learn More &rarr;</a>
            </article>
            <article class="service-card">
                <div class="service-icon">◌</div>
                <h3>Tyre Replacement</h3>
                <p>Professional tyre replacement for your vehicle.</p>
                <a href="/services">Learn More &rarr;</a>
            </article>
            <article class="service-card">
                <div class="service-icon">+</div>
                <h3>Puncture Repair</h3>
                <p>Reliable puncture and tubeless service.</p>
                <a href="/services">Learn More &rarr;</a>
            </article>"#
            .to_string();
    }

    let content = format!(
        r#"
<section class="hero">
    <div class="hero-background"></div>
    <div class="container hero-content">
        <div class="hero-text">
            <span class="eyebrow">TRUSTED TYRE &amp; AUTO SERVICE</span>
            <h1>Drive With <span>Confidence.</span></h1>
            <p>Premium tyres and professional vehicle services for a safer, smoother and more confident drive.</p>
            <div class="hero-actions">
                <a href="/tyres" class="btn btn-primary">Explore Tyres</a>
                <a href="/services" class="btn btn-outline">View Services</a>
            </div>
        </div>
        <div class="hero-card">
            <div class="hero-card-icon">&#9678;</div>
            <h3>Professional Tyre Care</h3>
            <p>Quality products and reliable service for your vehicle.</p>
            <div class="hero-card-line"></div>
            <span>Shri Krishna Tyre House</span>
        </div>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="section-heading">
            <div>
                <span class="eyebrow">FEATURED PRODUCTS</span>
                <h2>Popular Tyres</h2>
            </div>
            <a href="/tyres" class="text-link">View All Tyres &rarr;</a>
        </div>
        <div class="product-grid">
            {featured_tyres}
        </div>
    </div>
</section>

<section class="section section-dark">
    <div class="container">
        <div class="section-heading center">
            <span class="eyebrow">OUR SERVICES</span>
            <h2>Complete Tyre Care</h2>
            <p>Professional services to keep your vehicle performing at its best.</p>
        </div>
        <div class="service-grid">
            {service_cards}
        </div>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="section-heading center">
            <span class="eyebrow">BRANDS</span>
            <h2>Trusted Tyre Brands</h2>
        </div>
        <div class="brand-grid">
            {brand_boxes}
        </div>
    </div>
</section>

<section class="about-preview">
    <div class="container about-grid">
        <div class="about-image">
            <div class="image-placeholder">SHRI KRISHNA<br>TYRE HOUSE</div>
        </div>
        <div class="about-content">
            <span class="eyebrow">ABOUT US</span>
            <h2>Your Trusted Tyre Partner</h2>
            <p>Shri Krishna Tyre House provides quality tyres and professional tyre care services for customers looking for reliable vehicle performance.</p>
            <p>From tyre selection to alignment, balancing, puncture repair and tyre care, our focus is on dependable service and customer satisfaction.</p>
            <a href="/about" class="btn btn-primary">Learn About Us</a>
        </div>
    </div>
</section>

<section class="contact-strip">
    <div class="container contact-strip-content">
        <div>
            <span class="eyebrow">VISIT OUR STORE</span>
            <h2>Ready For Your Next Drive?</h2>
            <p>Singhana Road, Buhana, Jhunjhunu, Rajasthan 333502</p>
        </div>
        <a href="/contact" class="btn btn-light">Contact Us</a>
    </div>
</section>
"#,
        featured_tyres = featured_tyres,
        service_cards = service_cards,
        brand_boxes = brand_boxes,
    );

    Html(page("Home", &content))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
