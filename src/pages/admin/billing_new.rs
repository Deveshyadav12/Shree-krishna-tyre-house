use axum::{
    Form,
    extract::State,
    response::{Html, Redirect},
};
use serde::Deserialize;
use sqlx::Row;

use super::layout::admin_page;
use crate::AppState;

#[derive(Deserialize)]
pub struct NewBillForm {
    pub customer_name: String,
    pub customer_phone: String,
    pub billing_type: String,
    pub payment_method: String,
    pub discount: Option<i64>,
    pub notes: Option<String>,
    // tyre fields: tyre_id_N, tyre_qty_N (N = 0..9)
    pub tyre_id_0: Option<String>,
    pub tyre_qty_0: Option<i32>,
    pub tyre_id_1: Option<String>,
    pub tyre_qty_1: Option<i32>,
    pub tyre_id_2: Option<String>,
    pub tyre_qty_2: Option<i32>,
    pub tyre_id_3: Option<String>,
    pub tyre_qty_3: Option<i32>,
    pub tyre_id_4: Option<String>,
    pub tyre_qty_4: Option<i32>,
    // service fields: svc_id_N (N = 0..9)
    pub svc_id_0: Option<String>,
    pub svc_id_1: Option<String>,
    pub svc_id_2: Option<String>,
    pub svc_id_3: Option<String>,
    pub svc_id_4: Option<String>,
    pub svc_id_5: Option<String>,
    pub svc_id_6: Option<String>,
    pub svc_id_7: Option<String>,
    pub svc_id_8: Option<String>,
    pub svc_id_9: Option<String>,
}

pub async fn billing_new(State(state): State<AppState>) -> Html<String> {
    let tyres = sqlx::query(
        "SELECT id, brand, name, size, price, stock FROM inventory WHERE stock > 0 ORDER BY brand, name",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let services =
        sqlx::query("SELECT id, name, price FROM services WHERE active = TRUE ORDER BY name")
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

    let mut tyre_options = String::new();
    for row in &tyres {
        let id: String = row.get("id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let size: String = row.get("size");
        let price: i64 = row.get("price");
        let stock: i32 = row.get("stock");
        tyre_options.push_str(&format!(
            r#"<option value="{id}" data-price="{price}" data-name="{label}" data-stock="{stock}">{label} — ₹{price} (Stock: {stock})</option>"#,
            id = esc(&id),
            price = price,
            label = esc(&format!("{} {} {}", brand, name, size)),
            stock = stock,
        ));
    }

    let mut svc_cards = String::new();
    for (i, row) in services.iter().enumerate() {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let price: i64 = row.get("price");
        svc_cards.push_str(&format!(
            r#"<label class="nb-svc-card" id="svc-card-{i}">
                <input type="checkbox" class="nb-svc-check" name="svc_id_{i}" value="{id}"
                    data-price="{price}" data-name="{name}" onchange="nbRecalc()">
                <div class="nb-svc-info">
                    <strong>{name}</strong>
                    <span>&#8377;{price}</span>
                </div>
            </label>"#,
            i = i,
            id = esc(&id),
            price = price,
            name = esc(&name),
        ));
    }

    let content = format!(
        r#"
<div class="admin-page-header">
    <div>
        <div class="admin-eyebrow">BILLING</div>
        <h1>Create New Bill</h1>
        <p>Create an offline bill for a customer. Bill is saved as PENDING until cash is confirmed.</p>
    </div>
    <a href="/admin/billing" class="admin-secondary-button">Back to Billing</a>
</div>

<form method="post" action="/admin/billing/new" id="newBillForm">

<!-- BILLING TYPE -->
<div class="admin-panel" style="margin-bottom:20px;">
    <div class="admin-panel-header">
        <div><h2>Billing Type</h2><p>Select what this bill covers.</p></div>
    </div>
    <div class="nb-type-grid">
        <label class="nb-type-card" id="type-tyre">
            <input type="radio" name="billing_type" value="TYRE_ONLY" required onchange="nbTypeChange()">
            <div class="nb-type-icon">TY</div>
            <strong>Tyre Only</strong>
            <span>Tyre purchase only</span>
        </label>
        <label class="nb-type-card" id="type-service">
            <input type="radio" name="billing_type" value="SERVICE_ONLY" onchange="nbTypeChange()">
            <div class="nb-type-icon">SV</div>
            <strong>Service Only</strong>
            <span>Services only, no tyre</span>
        </label>
        <label class="nb-type-card" id="type-both">
            <input type="radio" name="billing_type" value="TYRE_AND_SERVICE" onchange="nbTypeChange()">
            <div class="nb-type-icon">TS</div>
            <strong>Tyre + Service</strong>
            <span>Tyres and services</span>
        </label>
    </div>
</div>

<!-- CUSTOMER -->
<div class="admin-panel" style="margin-bottom:20px;">
    <div class="admin-panel-header">
        <div><h2>Customer Details</h2><p>Enter customer name and phone.</p></div>
    </div>
    <div class="create-job-form-grid" style="padding:22px;">
        <div class="admin-form-group">
            <label>Customer Name *</label>
            <input type="text" name="customer_name" required placeholder="Rajesh Kumar">
        </div>
        <div class="admin-form-group">
            <label>Phone Number</label>
            <input type="tel" name="customer_phone" placeholder="9876543210">
        </div>
    </div>
</div>

<!-- TYRES SECTION -->
<div class="admin-panel" id="nb-tyres-section" style="margin-bottom:20px;display:none;">
    <div class="admin-panel-header">
        <div><h2>Select Tyres</h2><p>Choose tyres and quantities.</p></div>
    </div>
    <div style="padding:22px;">
        <div id="nb-tyre-rows">
            <div class="nb-tyre-row" id="nb-tyre-row-0">
                <div class="admin-form-group" style="flex:1;">
                    <label>Tyre</label>
                    <select name="tyre_id_0" class="nb-tyre-select" onchange="nbRecalc()">
                        <option value="">-- Select Tyre --</option>
                        {tyre_options}
                    </select>
                </div>
                <div class="admin-form-group" style="width:100px;">
                    <label>Qty</label>
                    <input type="number" name="tyre_qty_0" class="nb-tyre-qty" min="1" value="1" onchange="nbRecalc()">
                </div>
                <div class="nb-tyre-amount" id="nb-tyre-amt-0" style="align-self:flex-end;padding-bottom:8px;min-width:90px;text-align:right;font-weight:800;">&#8377;0</div>
            </div>
            <div class="nb-tyre-row" id="nb-tyre-row-1" style="display:none;">
                <div class="admin-form-group" style="flex:1;">
                    <label>Tyre</label>
                    <select name="tyre_id_1" class="nb-tyre-select" onchange="nbRecalc()">
                        <option value="">-- Select Tyre --</option>
                        {tyre_options}
                    </select>
                </div>
                <div class="admin-form-group" style="width:100px;">
                    <label>Qty</label>
                    <input type="number" name="tyre_qty_1" class="nb-tyre-qty" min="1" value="1" onchange="nbRecalc()">
                </div>
                <div class="nb-tyre-amount" id="nb-tyre-amt-1" style="align-self:flex-end;padding-bottom:8px;min-width:90px;text-align:right;font-weight:800;">&#8377;0</div>
            </div>
            <div class="nb-tyre-row" id="nb-tyre-row-2" style="display:none;">
                <div class="admin-form-group" style="flex:1;">
                    <label>Tyre</label>
                    <select name="tyre_id_2" class="nb-tyre-select" onchange="nbRecalc()">
                        <option value="">-- Select Tyre --</option>
                        {tyre_options}
                    </select>
                </div>
                <div class="admin-form-group" style="width:100px;">
                    <label>Qty</label>
                    <input type="number" name="tyre_qty_2" class="nb-tyre-qty" min="1" value="1" onchange="nbRecalc()">
                </div>
                <div class="nb-tyre-amount" id="nb-tyre-amt-2" style="align-self:flex-end;padding-bottom:8px;min-width:90px;text-align:right;font-weight:800;">&#8377;0</div>
            </div>
            <div class="nb-tyre-row" id="nb-tyre-row-3" style="display:none;">
                <div class="admin-form-group" style="flex:1;">
                    <label>Tyre</label>
                    <select name="tyre_id_3" class="nb-tyre-select" onchange="nbRecalc()">
                        <option value="">-- Select Tyre --</option>
                        {tyre_options}
                    </select>
                </div>
                <div class="admin-form-group" style="width:100px;">
                    <label>Qty</label>
                    <input type="number" name="tyre_qty_3" class="nb-tyre-qty" min="1" value="1" onchange="nbRecalc()">
                </div>
                <div class="nb-tyre-amount" id="nb-tyre-amt-3" style="align-self:flex-end;padding-bottom:8px;min-width:90px;text-align:right;font-weight:800;">&#8377;0</div>
            </div>
            <div class="nb-tyre-row" id="nb-tyre-row-4" style="display:none;">
                <div class="admin-form-group" style="flex:1;">
                    <label>Tyre</label>
                    <select name="tyre_id_4" class="nb-tyre-select" onchange="nbRecalc()">
                        <option value="">-- Select Tyre --</option>
                        {tyre_options}
                    </select>
                </div>
                <div class="admin-form-group" style="width:100px;">
                    <label>Qty</label>
                    <input type="number" name="tyre_qty_4" class="nb-tyre-qty" min="1" value="1" onchange="nbRecalc()">
                </div>
                <div class="nb-tyre-amount" id="nb-tyre-amt-4" style="align-self:flex-end;padding-bottom:8px;min-width:90px;text-align:right;font-weight:800;">&#8377;0</div>
            </div>
        </div>
        <button type="button" class="admin-secondary-button" style="margin-top:12px;" onclick="nbAddTyreRow()">+ Add Another Tyre</button>
    </div>
</div>

<!-- SERVICES SECTION -->
<div class="admin-panel" id="nb-services-section" style="margin-bottom:20px;display:none;">
    <div class="admin-panel-header">
        <div><h2>Select Services</h2><p>Choose services to include in this bill.</p></div>
    </div>
    <div class="nb-svc-grid" style="padding:22px;">
        {svc_cards}
    </div>
</div>

<!-- PAYMENT + TOTALS -->
<div class="admin-panel" style="margin-bottom:20px;">
    <div class="admin-panel-header">
        <div><h2>Payment &amp; Totals</h2></div>
    </div>
    <div style="padding:22px;">
        <div class="create-job-form-grid">
            <div class="admin-form-group">
                <label>Payment Method *</label>
                <select name="payment_method" required>
                    <option value="CASH">Cash</option>
                    <option value="UPI">UPI</option>
                    <option value="CARD">Card</option>
                </select>
            </div>
            <div class="admin-form-group">
                <label>Discount (&#8377;)</label>
                <input type="number" name="discount" min="0" value="0" id="nb-discount" onchange="nbRecalc()">
            </div>
            <div class="admin-form-group" style="grid-column:1/-1;">
                <label>Notes</label>
                <input type="text" name="notes" placeholder="Optional notes...">
            </div>
        </div>

        <div class="nb-totals-box">
            <div class="nb-total-row"><span>Subtotal</span><strong id="nb-subtotal">&#8377;0</strong></div>
            <div class="nb-total-row"><span>Discount</span><strong id="nb-discount-display">&#8377;0</strong></div>
            <div class="nb-total-row nb-grand"><span>Grand Total</span><strong id="nb-grand">&#8377;0</strong></div>
        </div>
    </div>
</div>

<div style="display:flex;gap:12px;justify-content:flex-end;margin-bottom:40px;">
    <a href="/admin/billing" class="admin-secondary-button">Cancel</a>
    <button type="submit" class="admin-primary-button" style="padding:12px 28px;font-size:13px;">
        Confirm Bill &amp; Print
    </button>
</div>

</form>

<script>
var nbTyreRowCount = 1;

function nbTypeChange() {{
    var val = document.querySelector('input[name="billing_type"]:checked');
    if (!val) return;
    var v = val.value;
    document.getElementById('nb-tyres-section').style.display =
        (v === 'TYRE_ONLY' || v === 'TYRE_AND_SERVICE') ? 'block' : 'none';
    document.getElementById('nb-services-section').style.display =
        (v === 'SERVICE_ONLY' || v === 'TYRE_AND_SERVICE') ? 'block' : 'none';
    nbRecalc();
}}

function nbAddTyreRow() {{
    if (nbTyreRowCount >= 5) return;
    document.getElementById('nb-tyre-row-' + nbTyreRowCount).style.display = 'flex';
    nbTyreRowCount++;
}}

function nbRecalc() {{
    var subtotal = 0;
    // tyres
    for (var i = 0; i < 5; i++) {{
        var sel = document.querySelector('[name="tyre_id_' + i + '"]');
        var qty = document.querySelector('[name="tyre_qty_' + i + '"]');
        var amt = document.getElementById('nb-tyre-amt-' + i);
        if (!sel || !qty || !amt) continue;
        var opt = sel.options[sel.selectedIndex];
        var price = opt ? parseInt(opt.getAttribute('data-price') || '0') : 0;
        var q = parseInt(qty.value || '1');
        var line = price * q;
        amt.innerHTML = '&#8377;' + line;
        subtotal += line;
    }}
    // services
    var svcs = document.querySelectorAll('.nb-svc-check:checked');
    svcs.forEach(function(c) {{
        subtotal += parseInt(c.getAttribute('data-price') || '0');
    }});
    var disc = parseInt(document.getElementById('nb-discount').value || '0');
    var grand = Math.max(0, subtotal - disc);
    document.getElementById('nb-subtotal').innerHTML = '&#8377;' + subtotal;
    document.getElementById('nb-discount-display').innerHTML = '&#8377;' + disc;
    document.getElementById('nb-grand').innerHTML = '&#8377;' + grand;
}}

// style type cards
document.querySelectorAll('input[name="billing_type"]').forEach(function(r) {{
    r.addEventListener('change', function() {{
        document.querySelectorAll('.nb-type-card').forEach(function(c) {{ c.classList.remove('nb-type-selected'); }});
        r.closest('.nb-type-card').classList.add('nb-type-selected');
    }});
}});
</script>
"#,
        tyre_options = tyre_options,
        svc_cards = svc_cards,
    );

    Html(admin_page("Billing", &content))
}

pub async fn create_bill(State(state): State<AppState>, Form(form): Form<NewBillForm>) -> Redirect {
    let billing_type = form.billing_type.trim().to_string();
    if billing_type.is_empty() {
        return Redirect::to("/admin/billing/new");
    }

    let customer_name = form.customer_name.trim().to_string();
    if customer_name.is_empty() {
        return Redirect::to("/admin/billing/new");
    }

    // Collect tyre selections from form
    let tyre_ids: Vec<Option<String>> = vec![
        form.tyre_id_0.clone(),
        form.tyre_id_1.clone(),
        form.tyre_id_2.clone(),
        form.tyre_id_3.clone(),
        form.tyre_id_4.clone(),
    ];
    let tyre_qtys: Vec<i32> = vec![
        form.tyre_qty_0.unwrap_or(1),
        form.tyre_qty_1.unwrap_or(1),
        form.tyre_qty_2.unwrap_or(1),
        form.tyre_qty_3.unwrap_or(1),
        form.tyre_qty_4.unwrap_or(1),
    ];
    let svc_ids: Vec<Option<String>> = vec![
        form.svc_id_0.clone(),
        form.svc_id_1.clone(),
        form.svc_id_2.clone(),
        form.svc_id_3.clone(),
        form.svc_id_4.clone(),
        form.svc_id_5.clone(),
        form.svc_id_6.clone(),
        form.svc_id_7.clone(),
        form.svc_id_8.clone(),
        form.svc_id_9.clone(),
    ];

    // Read tyre prices from DB (never trust browser)
    struct TyreItem {
        inventory_id: String,
        name: String,
        unit_price: i64,
        quantity: i32,
    }
    let mut tyre_items: Vec<TyreItem> = Vec::new();

    if billing_type == "TYRE_ONLY" || billing_type == "TYRE_AND_SERVICE" {
        for (i, maybe_id) in tyre_ids.iter().enumerate() {
            let Some(tid) = maybe_id else { continue };
            let tid = tid.trim();
            if tid.is_empty() {
                continue;
            }
            let qty = tyre_qtys[i].max(1);

            let row = sqlx::query(
                "SELECT id, brand, name, size, price, stock FROM inventory WHERE id = $1",
            )
            .bind(tid)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

            if let Some(row) = row {
                let price: i64 = row.get("price");
                let brand: String = row.get("brand");
                let name: String = row.get("name");
                let size: String = row.get("size");
                let inv_id: String = row.get("id");
                tyre_items.push(TyreItem {
                    inventory_id: inv_id,
                    name: format!("{} {} {}", brand, name, size),
                    unit_price: price,
                    quantity: qty,
                });
            }
        }
    }

    // Read service prices from DB
    struct SvcItem {
        service_id: String,
        name: String,
        unit_price: i64,
    }
    let mut svc_items: Vec<SvcItem> = Vec::new();

    if billing_type == "SERVICE_ONLY" || billing_type == "TYRE_AND_SERVICE" {
        for maybe_id in &svc_ids {
            let Some(sid) = maybe_id else { continue };
            let sid = sid.trim();
            if sid.is_empty() {
                continue;
            }

            let row =
                sqlx::query("SELECT id, name, price FROM services WHERE id = $1 AND active = TRUE")
                    .bind(sid)
                    .fetch_optional(&state.pool)
                    .await
                    .unwrap_or(None);

            if let Some(row) = row {
                let price: i64 = row.get("price");
                let name: String = row.get("name");
                let svc_id: String = row.get("id");
                svc_items.push(SvcItem {
                    service_id: svc_id,
                    name,
                    unit_price: price,
                });
            }
        }
    }

    if tyre_items.is_empty() && svc_items.is_empty() {
        return Redirect::to("/admin/billing/new");
    }

    // Calculate totals server-side
    let subtotal: i64 = tyre_items
        .iter()
        .map(|t| t.unit_price * t.quantity as i64)
        .sum::<i64>()
        + svc_items.iter().map(|s| s.unit_price).sum::<i64>();
    let discount = form.discount.unwrap_or(0).max(0);
    let total = (subtotal - discount).max(0);

    // Generate IDs
    let ts = chrono::Utc::now().timestamp_millis();
    let order_id = format!("ORD-{}", ts);
    let payment_id = format!("PAY-{}", ts);

    // Generate invoice number using sequence
    let seq_row = sqlx::query("SELECT nextval('invoice_seq') AS n")
        .fetch_one(&state.pool)
        .await;
    let seq_n: i64 = match seq_row {
        Ok(r) => r.get("n"),
        Err(e) => {
            eprintln!("Invoice seq error: {e}");
            ts % 100000
        }
    };
    let year = chrono::Utc::now().format("%Y");
    let invoice_number = format!("INV-{}-{:05}", year, seq_n);

    let payment_method = form.payment_method.trim().to_string();
    let notes = form.notes.clone().unwrap_or_default();

    // Begin transaction
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Transaction begin error: {e}");
            return Redirect::to("/admin/billing/new");
        }
    };

    // Insert order
    let r = sqlx::query(
        r#"INSERT INTO orders
           (id, invoice_number, customer_name, customer_phone, billing_type,
            subtotal, discount, tax, total, payment_status, order_status, source, notes)
           VALUES ($1,$2,$3,$4,$5,$6,$7,0,$8,'PENDING','PENDING','OFFLINE',$9)"#,
    )
    .bind(&order_id)
    .bind(&invoice_number)
    .bind(&customer_name)
    .bind(form.customer_phone.trim())
    .bind(&billing_type)
    .bind(subtotal)
    .bind(discount)
    .bind(total)
    .bind(&notes)
    .execute(&mut *tx)
    .await;

    if let Err(e) = r {
        eprintln!("Insert order error: {e}");
        return Redirect::to("/admin/billing/new");
    }

    // Insert tyre order_items
    for item in &tyre_items {
        let line_total = item.unit_price * item.quantity as i64;
        let _ = sqlx::query(
            r#"INSERT INTO order_items
               (order_id, item_type, inventory_id, item_name, quantity, unit_price, total)
               VALUES ($1,'TYRE',$2,$3,$4,$5,$6)"#,
        )
        .bind(&order_id)
        .bind(&item.inventory_id)
        .bind(&item.name)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(line_total)
        .execute(&mut *tx)
        .await;
    }

    // Insert service order_items
    for item in &svc_items {
        let _ = sqlx::query(
            r#"INSERT INTO order_items
               (order_id, item_type, service_id, item_name, quantity, unit_price, total)
               VALUES ($1,'SERVICE',$2,$3,1,$4,$4)"#,
        )
        .bind(&order_id)
        .bind(&item.service_id)
        .bind(&item.name)
        .bind(item.unit_price)
        .execute(&mut *tx)
        .await;
    }

    // Insert payment record as PENDING
    let _ = sqlx::query(
        r#"INSERT INTO payments
           (id, order_id, payment_method, payment_status, amount, currency)
           VALUES ($1,$2,$3,'PENDING',$4,'INR')"#,
    )
    .bind(&payment_id)
    .bind(&order_id)
    .bind(&payment_method)
    .bind(total)
    .execute(&mut *tx)
    .await;

    if let Err(e) = tx.commit().await {
        eprintln!("Bill commit error: {e}");
        return Redirect::to("/admin/billing/new");
    }

    // Redirect to the order detail/print page
    Redirect::to(&format!("/admin/billing/order/{}", order_id))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
