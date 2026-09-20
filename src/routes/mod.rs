use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    AppState,
    pages::{about, admin, brands, contact, home, services, tyres},
};

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home::home))
        .route("/tyres", get(tyres::tyres))
        .route("/tyres/{id}", get(tyres::tyre_detail))
        .route("/services", get(services::services))
        .route("/brands", get(brands::brands))
        .route("/brands/{slug}", get(brands::brand_detail))
        .route("/about", get(about::about))
        .route("/contact", get(contact::contact))
        .route(
            "/admin/login",
            get(admin::login::login).post(admin::login::login_submit),
        )
        .route("/admin", get(admin::dashboard::dashboard))
        // inventory/new and inventory/history must come before inventory/{id}
        .route(
            "/admin/inventory/new",
            get(admin::inventory_new::inventory_new).post(admin::inventory_new::create_inventory),
        )
        .route(
            "/admin/inventory/history",
            get(admin::stock_history::stock_history),
        )
        .route("/admin/inventory", get(admin::inventory::inventory))
        .route(
            "/admin/inventory/{id}",
            get(admin::inventory::inventory_detail),
        )
        .route(
            "/admin/inventory/{id}/edit",
            get(admin::inventory::inventory_edit).post(admin::inventory::update_inventory),
        )
        .route(
            "/admin/inventory/{id}/adjust",
            post(admin::inventory::adjust_stock),
        )
        .route(
            "/admin/services",
            get(admin::services::services).post(admin::services::create_service),
        )
        .route(
            "/admin/services/{id}/delete",
            post(admin::services::delete_service),
        )
        .route(
            "/admin/brands",
            get(admin::brands::brands).post(admin::brands::create_brand),
        )
        .route(
            "/admin/brands/{id}/delete",
            post(admin::brands::delete_brand),
        )
        .route("/admin/customers", get(admin::customers::customers))
        .route(
            "/admin/customers/new",
            get(admin::customer_new::customer_new).post(admin::customer_new::create_customer),
        )
        .route(
            "/admin/customers/{id}",
            get(admin::customers::customer_detail),
        )
        .route(
            "/admin/jobs",
            get(admin::jobs::jobs).post(admin::job_new::create_job),
        )
        .route("/admin/jobs/new", get(admin::job_new::job_new))
        .route("/admin/jobs/{id}", get(admin::jobs::job_detail))
        .route("/admin/jobs/{id}/start", post(admin::jobs::start_job))
        .route("/admin/jobs/{id}/complete", post(admin::jobs::complete_job))
        .route("/admin/billing", get(admin::billing::billing))
        .route(
            "/admin/billing/new",
            get(admin::billing_new::billing_new).post(admin::billing_new::create_bill),
        )
        .route(
            "/admin/billing/order/{id}",
            get(admin::billing::order_detail),
        )
        .route(
            "/admin/billing/order/{id}/receive-cash",
            get(admin::billing::receive_cash_form).post(admin::billing::confirm_cash_received),
        )
        .route("/admin/billing/{id}", get(admin::billing::bill_detail))
        .route(
            "/admin/billing/{id}/pay",
            post(admin::billing::process_payment),
        )
        .route("/admin/invoices", get(admin::invoices::invoices))
        .route("/admin/invoices/{id}", get(admin::invoices::invoice_detail))
}
