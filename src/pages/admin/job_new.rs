use axum::{
    extract::{Form, State},
    response::{Html, Redirect},
};
use serde::Deserialize;
use sqlx::Row;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateJobForm {
    pub customer: String,
    pub vehicle: String,
    pub vehicle_type: String,
    pub registration: String,
    pub items: String,
    pub notes: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct JobItem {
    pub name: String,
    pub price: i64,
    pub quantity: i32,
    pub item_type: String,
}

pub async fn job_new(State(state): State<AppState>) -> Html<String> {
    let customers =
        sqlx::query("SELECT id, name, vehicle, registration FROM customers ORDER BY name")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let tyres = sqlx::query(
        "SELECT id, name, size, price, stock FROM inventory WHERE stock > 0 ORDER BY brand, name",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let services =
        sqlx::query("SELECT id, name, price FROM services WHERE active = TRUE ORDER BY name")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let customer_options = customers
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let vehicle: String = row.get("vehicle");
            let registration: String = row.get("registration");
            format!(
                r#"<option value="{id}" data-vehicle="{vehicle}" data-registration="{registration}">{name}</option>"#,
                id = escape_html(&id),
                name = escape_html(&name),
                vehicle = escape_html(&vehicle),
                registration = escape_html(&registration),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let tyre_cards = tyres
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let name: String = row.get("name");
            let size: String = row.get("size");
            let price: i64 = row.get("price");
            let stock: i32 = row.get("stock");
            format!(
                r#"<label class="job-product-card">
                    <input type="checkbox" class="job-tyre"
                        data-id="{id}" data-name="{name}" data-price="{price}">
                    <div>
                        <strong>{name}</strong>
                        <span>{size} &middot; Stock: {stock}</span>
                    </div>
                    <div class="job-price">&#8377;{price}</div>
                    <div class="job-quantity">
                        <button type="button" class="qty-minus">&minus;</button>
                        <input type="number" class="qty-input" value="1" min="1" max="{stock}">
                        <button type="button" class="qty-plus">+</button>
                    </div>
                </label>"#,
                id = escape_html(&id),
                name = escape_html(&name),
                size = escape_html(&size),
                price = price,
                stock = stock,
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let service_cards = services
        .iter()
        .map(|row| {
            let name: String = row.get("name");
            let price: i64 = row.get("price");
            format!(
                r#"<label class="job-service-card">
                    <input type="checkbox" class="job-service"
                        data-name="{name}" data-price="{price}">
                    <span>
                        <strong>{name}</strong>
                        <small>&#8377;{price}</small>
                    </span>
                </label>"#,
                name = escape_html(&name),
                price = price,
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let content = format!(
        r#"
<section class="admin-page">
    <div class="admin-page-header">
        <div>
            <span class="admin-eyebrow">JOB MANAGEMENT</span>
            <h1>Create Job Order</h1>
            <p>Create a new tyre and service job for a customer.</p>
        </div>
        <div class="admin-page-actions">
            <a href="/admin/jobs" class="admin-secondary-button">Back to Jobs</a>
        </div>
    </div>

    <form method="post" action="/admin/jobs" id="createJobForm">
        <div class="create-job-layout">

            <div class="create-job-main">

                <div class="admin-panel create-job-panel">
                    <div class="create-job-panel-header">
                        <div class="create-job-step">1</div>
                        <div>
                            <h2>Customer &amp; Vehicle</h2>
                            <p>Select an existing customer.</p>
                        </div>
                    </div>
                    <div class="create-job-form-grid">
                        <div class="admin-form-group">
                            <label for="jobCustomer">Customer</label>
                            <select id="jobCustomer" name="customer" required>
                                <option value="">Select customer</option>
                                {customer_options}
                            </select>
                        </div>
                        <div class="admin-form-group">
                            <label for="jobVehicle">Vehicle</label>
                            <select id="jobVehicle" name="vehicle" required>
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
                            <label for="jobVehicleType">Vehicle Type</label>
                            <select id="jobVehicleType" name="vehicle_type" required>
                                <option value="">Select vehicle type</option>
                                <option>Passenger Car</option>
                                <option>SUV</option>
                                <option>Sedan</option>
                                <option>Hatchback</option>
                                <option>Two Wheeler</option>
                                <option>Commercial Vehicle</option>
                            </select>
                        </div>
                        <div class="admin-form-group">
                            <label for="jobRegistration">Registration Number</label>
                            <input id="jobRegistration" name="registration" type="text" placeholder="RJ-18-AB-1024" required>
                        </div>
                    </div>
                </div>

                <div class="admin-panel create-job-panel">
                    <div class="create-job-panel-header">
                        <div class="create-job-step">2</div>
                        <div>
                            <h2>Tyres</h2>
                            <p>Select tyres from current inventory.</p>
                        </div>
                        <a href="/admin/inventory/new" class="admin-secondary-button" style="margin-left:auto">+ Add Tyre</a>
                    </div>
                    <div class="job-product-grid">{tyre_cards}</div>
                    <div class="inventory-job-note">Only tyres with available stock are shown.</div>
                </div>

                <div class="admin-panel create-job-panel">
                    <div class="create-job-panel-header">
                        <div class="create-job-step">3</div>
                        <div>
                            <h2>Services</h2>
                            <p>Select required workshop services.</p>
                        </div>
                    </div>
                    <div class="job-service-grid">{service_cards}</div>
                </div>

                <div class="admin-panel create-job-panel">
                    <div class="create-job-panel-header">
                        <div class="create-job-step">4</div>
                        <div>
                            <h2>Job Notes</h2>
                            <p>Add workshop instructions or customer requirements.</p>
                        </div>
                    </div>
                    <div style="padding:0 24px 24px">
                        <textarea class="admin-form-textarea" name="notes" rows="5" placeholder="Example: Replace all four tyres and perform wheel alignment."></textarea>
                    </div>
                </div>

            </div>

            <div class="create-job-summary">
                <div class="job-summary-card">
                    <div class="job-summary-header">
                        <span>ORDER SUMMARY</span>
                        <strong>NEW</strong>
                    </div>
                    <div class="job-summary-customer" id="summaryCustomerBox" style="display:none">
                        <strong id="summaryCustomer">-</strong>
                        <small id="summaryVehicle">-</small>
                    </div>
                    <div class="job-summary-items" id="summaryItems">
                        <div class="job-summary-empty">No items selected yet.</div>
                    </div>
                    <div class="job-summary-totals">
                        <div>
                            <span>Tyres</span>
                            <strong id="tyreTotal">&#8377;0</strong>
                        </div>
                        <div>
                            <span>Services</span>
                            <strong id="serviceTotal">&#8377;0</strong>
                        </div>
                        <div class="job-grand-total">
                            <span>Grand Total</span>
                            <strong id="grandTotal">&#8377;0</strong>
                        </div>
                    </div>
                    <input type="hidden" name="items" id="jobItems" value="[]">
                    <button type="submit" class="job-create-button">Create Job Order</button>
                    <p class="job-summary-note">Items and totals update as you select above.</p>
                </div>
            </div>

        </div>
    </form>
</section>
"#,
        customer_options = customer_options,
        tyre_cards = tyre_cards,
        service_cards = service_cards,
    );

    Html(super::layout::admin_page("Create Job Order", &content))
}

pub async fn create_job(
    State(state): State<AppState>,
    Form(form): Form<CreateJobForm>,
) -> Redirect {
    let customer_name = sqlx::query("SELECT name FROM customers WHERE id = $1")
        .bind(&form.customer)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .map(|row| row.get::<String, _>("name"))
        .unwrap_or_else(|| form.customer.clone());

    let items: Vec<JobItem> = serde_json::from_str(&form.items).unwrap_or_default();

    let total: i64 = items
        .iter()
        .map(|item| item.price * item.quantity as i64)
        .sum();

    let job_id = format!("JOB-{}", chrono::Utc::now().timestamp_millis());

    let result = sqlx::query(
        r#"INSERT INTO jobs (id, customer, vehicle, vehicle_type, registration, notes, status, total, payment_status, payment_method, invoice_id)
           VALUES ($1, $2, $3, $4, $5, $6, 'NEW', $7, 'UNPAID', '-', '-')"#,
    )
    .bind(&job_id)
    .bind(&customer_name)
    .bind(&form.vehicle)
    .bind(&form.vehicle_type)
    .bind(&form.registration)
    .bind(&form.notes)
    .bind(total)
    .execute(&state.pool)
    .await;

    if let Err(e) = result {
        eprintln!("Create job error: {}", e);
        return Redirect::to("/admin/jobs");
    }

    for item in &items {
        let _ = sqlx::query(
            r#"INSERT INTO job_items (job_id, name, price, quantity, item_type)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(&job_id)
        .bind(&item.name)
        .bind(item.price)
        .bind(item.quantity)
        .bind(&item.item_type)
        .execute(&state.pool)
        .await;
    }

    Redirect::to(&format!("/admin/jobs/{}", job_id))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
