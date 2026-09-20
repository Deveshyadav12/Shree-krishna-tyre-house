use axum::{extract::State, response::Html};
use sqlx::Row;

use super::layout::page;
use crate::AppState;

pub async fn services(State(state): State<AppState>) -> Html<String> {
    let rows = sqlx::query(
        "SELECT id, name, description, price, active FROM services WHERE active = TRUE ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut service_cards = String::new();
    let icons = ["◉", "◎", "◌", "+", "●", "◈", "◆", "◇"];

    for (i, row) in rows.iter().enumerate() {
        let name: String = row.get("name");
        let description: String = row.get("description");
        let price: i64 = row.get("price");
        let icon = icons[i % icons.len()];

        let price_html = if price > 0 {
            format!(
                r#"<div style="margin-top:12px;color:#2563eb;font-size:14px;font-weight:800;">&#8377;{price}</div>"#,
                price = price
            )
        } else {
            r#"<div style="margin-top:12px;color:#64748b;font-size:13px;font-weight:700;">Contact for pricing</div>"#.to_string()
        };

        service_cards.push_str(&format!(
            r#"<article class="large-service-card">
                <div class="large-service-icon">{icon}</div>
                <span class="eyebrow">TYRE SERVICE</span>
                <h2>{name}</h2>
                <p>{description}</p>
                {price_html}
            </article>"#,
            icon = icon,
            name = esc(&name),
            description = esc(&description),
            price_html = price_html,
        ));
    }

    let empty_msg = if rows.is_empty() {
        r#"<div style="text-align:center;padding:60px 20px;color:#64748b;">
            <p style="font-size:18px;font-weight:700;">No services listed yet.</p>
            <p style="margin-top:8px;font-size:14px;">Contact us for service information.</p>
        </div>"#
    } else {
        ""
    };

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">OUR SERVICES</span>
        <h1>Professional Tyre Care</h1>
        <p>From tyre replacement to wheel alignment, we provide essential tyre and wheel services for your vehicle.</p>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="section-heading">
            <div>
                <span class="eyebrow">WHAT WE DO</span>
                <h2>Tyre &amp; Wheel Services</h2>
                <p>Choose the service your vehicle needs.</p>
            </div>
        </div>
        <div class="service-list-grid">
            {service_cards}
            {empty_msg}
        </div>
    </div>
</section>

<section class="contact-strip">
    <div class="container">
        <div class="contact-strip-content">
            <div>
                <span class="eyebrow">NEED TYRE SERVICE?</span>
                <h2>Talk To Shri Krishna Tyre House</h2>
                <p>Contact us for tyre information and service enquiries.</p>
            </div>
            <a href="/contact" class="btn btn-light">Contact Us &rarr;</a>
        </div>
    </div>
</section>
"#,
        service_cards = service_cards,
        empty_msg = empty_msg,
    );

    Html(page("Services", &content))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
