use axum::{extract::State, response::Html};
use sqlx::Row;

use super::layout::page;
use crate::AppState;

pub async fn about(State(state): State<AppState>) -> Html<String> {
    // Pull live counts to show on the about page
    let counts = sqlx::query(
        r#"SELECT
            (SELECT COUNT(*) FROM inventory WHERE stock > 0) AS in_stock,
            (SELECT COUNT(*) FROM brands WHERE active = TRUE) AS brand_count,
            (SELECT COUNT(*) FROM services WHERE active = TRUE) AS service_count
        "#,
    )
    .fetch_one(&state.pool)
    .await;

    let (in_stock, brand_count, service_count) = match counts {
        Ok(row) => (
            row.get::<i64, _>("in_stock"),
            row.get::<i64, _>("brand_count"),
            row.get::<i64, _>("service_count"),
        ),
        Err(e) => {
            eprintln!("About page stats error: {e}");
            (0, 0, 0)
        }
    };

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">ABOUT US</span>
        <h1>Built Around Better Driving</h1>
        <p>Shri Krishna Tyre House provides quality tyres and professional tyre and wheel care services for everyday drivers.</p>
    </div>
</section>

<section class="about-preview">
    <div class="container about-grid">
        <div class="about-image">
            <div class="image-placeholder">SHRI KRISHNA<br>TYRE HOUSE</div>
        </div>
        <div class="about-content">
            <span class="eyebrow">SHRI KRISHNA TYRE HOUSE</span>
            <h2>Quality Tyres. Professional Service.</h2>
            <p>We provide tyre products and essential tyre services for customers looking for dependable vehicle care.</p>
            <p>Our services include wheel alignment, wheel balancing, tyre replacement, puncture repair, tyre rotation, tubeless repair and other tyre care services.</p>
            <p>Our goal is to make tyre selection and vehicle tyre care simple and convenient for our customers.</p>
        </div>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="section-heading center">
            <span class="eyebrow">BY THE NUMBERS</span>
            <h2>What We Offer</h2>
        </div>
        <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:24px;max-width:700px;margin:0 auto;">
            <div style="text-align:center;padding:30px 20px;background:#f8fafc;border:1px solid #e2e8f0;border-radius:14px;">
                <div style="font-size:36px;font-weight:900;color:#111827;">{in_stock}</div>
                <div style="margin-top:8px;color:#64748b;font-size:13px;font-weight:700;">Tyres In Stock</div>
            </div>
            <div style="text-align:center;padding:30px 20px;background:#f8fafc;border:1px solid #e2e8f0;border-radius:14px;">
                <div style="font-size:36px;font-weight:900;color:#111827;">{brand_count}</div>
                <div style="margin-top:8px;color:#64748b;font-size:13px;font-weight:700;">Tyre Brands</div>
            </div>
            <div style="text-align:center;padding:30px 20px;background:#f8fafc;border:1px solid #e2e8f0;border-radius:14px;">
                <div style="font-size:36px;font-weight:900;color:#111827;">{service_count}</div>
                <div style="margin-top:8px;color:#64748b;font-size:13px;font-weight:700;">Services</div>
            </div>
        </div>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="section-heading">
            <div>
                <span class="eyebrow">WHAT WE FOCUS ON</span>
                <h2>Professional Tyre Care</h2>
                <p>Our website and store services are designed around your tyre needs.</p>
            </div>
        </div>
        <div class="service-grid">
            <article class="service-card" style="background:#f8fafc;border-color:#e2e8f0;">
                <div class="service-icon" style="background:#eff6ff;color:#2563eb;">01</div>
                <h3 style="color:#0f172a;">Quality Tyres</h3>
                <p style="color:#64748b;">Explore tyre options from multiple recognised brands.</p>
            </article>
            <article class="service-card" style="background:#f8fafc;border-color:#e2e8f0;">
                <div class="service-icon" style="background:#eff6ff;color:#2563eb;">02</div>
                <h3 style="color:#0f172a;">Professional Service</h3>
                <p style="color:#64748b;">Tyre and wheel services for everyday vehicle requirements.</p>
            </article>
            <article class="service-card" style="background:#f8fafc;border-color:#e2e8f0;">
                <div class="service-icon" style="background:#eff6ff;color:#2563eb;">03</div>
                <h3 style="color:#0f172a;">Customer Support</h3>
                <p style="color:#64748b;">Contact us for tyre information and service enquiries.</p>
            </article>
            <article class="service-card" style="background:#f8fafc;border-color:#e2e8f0;">
                <div class="service-icon" style="background:#eff6ff;color:#2563eb;">04</div>
                <h3 style="color:#0f172a;">Complete Tyre Care</h3>
                <p style="color:#64748b;">From inspection and repair to replacement and maintenance.</p>
            </article>
        </div>
    </div>
</section>

<section class="contact-strip">
    <div class="container">
        <div class="contact-strip-content">
            <div>
                <span class="eyebrow">VISIT SHRI KRISHNA TYRE HOUSE</span>
                <h2>Let's Talk About Your Tyres</h2>
                <p>Contact our store for tyre information and service enquiries.</p>
            </div>
            <a href="/contact" class="btn btn-light">Contact Us &rarr;</a>
        </div>
    </div>
</section>
"#,
        in_stock = in_stock,
        brand_count = brand_count,
        service_count = service_count,
    );

    Html(page("About", &content))
}
