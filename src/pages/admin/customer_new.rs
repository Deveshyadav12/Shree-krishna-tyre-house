use axum::{
    extract::{Form, State},
    response::{Html, Redirect},
};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
pub struct CustomerForm {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub vehicle: String,
    pub registration: String,
}

pub async fn customer_new() -> Html<String> {
    let content = r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">CUSTOMER MANAGEMENT</span>
            <h1>Add Customer</h1>
            <p>Create a new customer profile and vehicle record.</p>
        </div>
        <div class="admin-page-actions">
            <a href="/admin/customers" class="admin-secondary-button">Back to Customers</a>
        </div>
    </div>

    <form method="post" action="/admin/customers/new" class="af-form">

        <div class="admin-panel" style="margin-bottom:20px">
            <div class="admin-panel-header af-panel-header">
                <div>
                    <h2>Customer Information</h2>
                    <p>Enter the customer's basic contact information.</p>
                </div>
            </div>
            <div class="af-grid af-body">
                <div class="admin-form-group">
                    <label for="name">Customer Name</label>
                    <input id="name" name="name" type="text" placeholder="Enter customer name" required>
                </div>
                <div class="admin-form-group">
                    <label for="phone">Phone Number</label>
                    <input id="phone" name="phone" type="tel" placeholder="9876543210" required>
                </div>
                <div class="admin-form-group">
                    <label for="email">Email</label>
                    <input id="email" name="email" type="email" placeholder="customer@example.com">
                </div>
                <div class="admin-form-group af-full">
                    <label for="address">Address</label>
                    <textarea id="address" name="address" rows="4" placeholder="Enter customer address"></textarea>
                </div>
            </div>
        </div>

        <div class="admin-panel" style="margin-bottom:20px">
            <div class="admin-panel-header af-panel-header">
                <div>
                    <h2>Vehicle Information</h2>
                    <p>Store the customer's primary vehicle.</p>
                </div>
            </div>
            <div class="af-grid af-body">
                <div class="admin-form-group">
                    <label for="vehicle">Vehicle</label>
                    <select id="vehicle" name="vehicle" required>
                        <option value="">Select vehicle</option>
                        <option>Maruti Swift</option>
                        <option>Hyundai Creta</option>
                        <option>Tata Nexon</option>
                        <option>Mahindra Scorpio</option>
                        <option>Honda Activa</option>
                        <option>Maruti Baleno</option>
                        <option>Hyundai i20</option>
                        <option>Tata Punch</option>
                        <option>Mahindra Thar</option>
                        <option>Honda City</option>
                        <option>Other</option>
                    </select>
                </div>
                <div class="admin-form-group">
                    <label for="registration">Registration Number</label>
                    <input id="registration" name="registration" type="text" placeholder="RJ-18-AB-1024" required>
                </div>
            </div>
        </div>

        <div class="af-actions">
            <a href="/admin/customers" class="admin-secondary-button">Cancel</a>
            <button type="submit" class="admin-primary-button">Save Customer</button>
        </div>

    </form>
</section>
"#;

    Html(super::layout::admin_page("Add Customer", content))
}

pub async fn create_customer(
    State(state): State<AppState>,
    Form(form): Form<CustomerForm>,
) -> Redirect {
    let id = format!("CUS-{}", chrono::Utc::now().timestamp_millis());

    let result = sqlx::query(
        r#"INSERT INTO customers (id, name, phone, email, address, vehicle, registration)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(&id)
    .bind(form.name.trim())
    .bind(form.phone.trim())
    .bind(form.email.trim())
    .bind(form.address.trim())
    .bind(form.vehicle.trim())
    .bind(form.registration.trim().to_uppercase())
    .execute(&state.pool)
    .await;

    if let Err(e) = result {
        eprintln!("Create customer error: {}", e);
    }

    Redirect::to("/admin/customers")
}
