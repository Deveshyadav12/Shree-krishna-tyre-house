use axum::{extract::State, response::Html};

use super::layout::page;
use crate::AppState;

pub async fn contact(State(_state): State<AppState>) -> Html<String> {
    let content = r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">CONTACT US</span>
        <h1>Let's Talk About Your Tyres</h1>
        <p>Contact Shri Krishna Tyre House for tyre information, products and service enquiries.</p>
    </div>
</section>

<section class="section">
    <div class="container contact-page-grid">

        <div>
            <span class="eyebrow">VISIT OUR STORE</span>
            <h2>Shri Krishna Tyre House</h2>
            <p class="contact-description">
                For tyre information, product enquiries and tyre services, contact our store.
            </p>

            <div class="contact-information">
                <div class="contact-item">
                    <strong>Address</strong>
                    <span>Singhana Road, Buhana, Jhunjhunu, Rajasthan 333502</span>
                </div>
                <div class="contact-item">
                    <strong>Phone</strong>
                    <span>Contact store for phone details</span>
                </div>
                <div class="contact-item">
                    <strong>Services</strong>
                    <span>Tyres, wheel alignment, wheel balancing, puncture repair, tyre replacement and tyre care.</span>
                </div>
                <div class="contact-item">
                    <strong>Store</strong>
                    <span>Shri Krishna Tyre House</span>
                </div>
            </div>
        </div>

        <div class="contact-form-card">
            <span class="eyebrow">SEND MESSAGE</span>
            <h2>Contact Us</h2>
            <p style="margin-top:8px;color:#64748b;font-size:13px;">
                Send your enquiry and our team can respond with relevant information.
            </p>

            <form id="contactForm">
                <div class="form-group">
                    <label for="name">Name</label>
                    <input id="name" name="name" type="text" required placeholder="Your name">
                </div>
                <div class="form-group">
                    <label for="phone">Phone</label>
                    <input id="phone" name="phone" type="tel" required placeholder="Your phone number">
                </div>
                <div class="form-group">
                    <label for="email">Email</label>
                    <input id="email" name="email" type="email" placeholder="Your email address">
                </div>
                <div class="form-group">
                    <label for="message">Message</label>
                    <textarea id="message" name="message" rows="6" required placeholder="Tell us what you need..."></textarea>
                </div>
                <button type="submit" class="btn btn-primary">Send Message</button>
                <div id="contactStatus"></div>
            </form>
        </div>

    </div>
</section>

<section class="section section-dark">
    <div class="container">
        <div class="section-heading">
            <div>
                <span class="eyebrow">FIND US</span>
                <h2>Shri Krishna Tyre House</h2>
                <p>Singhana Road, Buhana, Jhunjhunu, Rajasthan 333502</p>
            </div>
        </div>
        <div style="min-height:300px;display:grid;place-items:center;border:1px solid rgba(255,255,255,0.10);border-radius:18px;background:rgba(255,255,255,0.04);text-align:center;padding:30px;">
            <div>
                <div style="font-size:42px;margin-bottom:15px;">LOCATION</div>
                <strong>Singhana Road, Buhana</strong>
                <p style="margin-top:7px;color:#94a3b8;">Jhunjhunu, Rajasthan 333502</p>
            </div>
        </div>
    </div>
</section>
"#;

    Html(page("Contact", content))
}
