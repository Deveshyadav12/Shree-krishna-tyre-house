use axum::{extract::State, response::Json};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use sqlx::Row;

use crate::AppState;

#[derive(Deserialize)]
pub struct CartItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub name: String,
    pub price: i64,
    pub qty: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub items: Vec<CartItem>,
}

#[derive(Deserialize)]
pub struct VerifyPaymentRequest {
    pub razorpay_order_id: String,
    pub razorpay_payment_id: String,
    pub razorpay_signature: String,
    pub order_id: String,
}

#[derive(Serialize)]
pub struct CreateOrderResponse {
    pub success: bool,
    pub order_id: String,
    pub invoice_number: String,
    pub razorpay_order_id: String,
    pub amount_paise: i64,
    pub billing_type: String,
    pub error: Option<String>,
}

pub async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Json<Value> {
    let customer_name = req.customer_name.trim().to_string();
    let customer_phone = req.customer_phone.trim().to_string();

    if customer_name.is_empty() || customer_phone.is_empty() {
        return Json(json!({"success": false, "error": "Name and phone are required."}));
    }
    if req.items.is_empty() {
        return Json(json!({"success": false, "error": "Cart is empty."}));
    }

    // Determine billing type from items
    let has_tyre = req.items.iter().any(|i| i.item_type == "TYRE");
    let has_service = req.items.iter().any(|i| i.item_type == "SERVICE");
    let billing_type = match (has_tyre, has_service) {
        (true, true) => "TYRE_AND_SERVICE",
        (true, false) => "TYRE_ONLY",
        (false, true) => "SERVICE_ONLY",
        _ => return Json(json!({"success": false, "error": "Invalid cart items."})),
    };

    // Read prices from DB — never trust client prices
    struct ResolvedItem {
        inventory_id: Option<String>,
        service_id: Option<String>,
        item_type: String,
        item_name: String,
        unit_price: i64,
        quantity: i32,
    }
    let mut resolved: Vec<ResolvedItem> = Vec::new();

    for item in &req.items {
        let qty = item.qty.unwrap_or(1).max(1);
        if item.item_type == "TYRE" {
            let row = sqlx::query(
                "SELECT id, brand, name, size, price, stock FROM inventory WHERE id = $1 AND stock > 0",
            )
            .bind(&item.id)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

            let Some(row) = row else {
                return Json(
                    json!({"success": false, "error": format!("Tyre {} is out of stock or not found.", item.id)}),
                );
            };
            let price: i64 = row.get("price");
            let brand: String = row.get("brand");
            let name: String = row.get("name");
            let size: String = row.get("size");
            let inv_id: String = row.get("id");
            resolved.push(ResolvedItem {
                inventory_id: Some(inv_id),
                service_id: None,
                item_type: "TYRE".to_string(),
                item_name: format!("{} {} {}", brand, name, size),
                unit_price: price,
                quantity: qty,
            });
        } else if item.item_type == "SERVICE" {
            let row =
                sqlx::query("SELECT id, name, price FROM services WHERE id = $1 AND active = TRUE")
                    .bind(&item.id)
                    .fetch_optional(&state.pool)
                    .await
                    .unwrap_or(None);

            let Some(row) = row else {
                return Json(
                    json!({"success": false, "error": format!("Service {} not found.", item.id)}),
                );
            };
            let price: i64 = row.get("price");
            let name: String = row.get("name");
            let svc_id: String = row.get("id");
            resolved.push(ResolvedItem {
                inventory_id: None,
                service_id: Some(svc_id),
                item_type: "SERVICE".to_string(),
                item_name: name,
                unit_price: price,
                quantity: 1,
            });
        }
    }

    if resolved.is_empty() {
        return Json(json!({"success": false, "error": "No valid items found."}));
    }

    let subtotal: i64 = resolved
        .iter()
        .map(|i| i.unit_price * i.quantity as i64)
        .sum();
    let total = subtotal;
    let amount_paise = total * 100; // Razorpay uses paise

    // Generate IDs
    let ts = chrono::Utc::now().timestamp_millis();
    let order_id = format!("ORD-{}", ts);
    let payment_id = format!("PAY-{}", ts);

    // Invoice number from sequence
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

    // Create Razorpay order via HTTP
    let rzp_key_id = std::env::var("RAZORPAY_KEY_ID").unwrap_or_default();
    let rzp_key_secret = std::env::var("RAZORPAY_KEY_SECRET").unwrap_or_default();

    if rzp_key_id.is_empty() || rzp_key_secret.is_empty() {
        return Json(
            json!({"success": false, "error": "Razorpay is not configured. Please contact the store."}),
        );
    }

    let rzp_order =
        create_razorpay_order(&rzp_key_id, &rzp_key_secret, amount_paise, &order_id).await;
    let rzp_order_id = match rzp_order {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Razorpay order creation error: {e}");
            return Json(
                json!({"success": false, "error": "Failed to create payment order. Please try again."}),
            );
        }
    };

    // Save to PostgreSQL as PENDING
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("TX begin error: {e}");
            return Json(json!({"success": false, "error": "Database error."}));
        }
    };

    let r = sqlx::query(
        r#"INSERT INTO orders
           (id, invoice_number, customer_name, customer_phone, billing_type,
            subtotal, discount, tax, total, payment_status, order_status, source)
           VALUES ($1,$2,$3,$4,$5,$6,0,0,$7,'PENDING','PENDING','ONLINE')"#,
    )
    .bind(&order_id)
    .bind(&invoice_number)
    .bind(&customer_name)
    .bind(&customer_phone)
    .bind(billing_type)
    .bind(subtotal)
    .bind(total)
    .execute(&mut *tx)
    .await;

    if let Err(e) = r {
        eprintln!("Insert order error: {e}");
        return Json(json!({"success": false, "error": "Failed to save order."}));
    }

    for item in &resolved {
        let line = item.unit_price * item.quantity as i64;
        let _ = sqlx::query(
            r#"INSERT INTO order_items
               (order_id, item_type, inventory_id, service_id, item_name, quantity, unit_price, total)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"#,
        )
        .bind(&order_id)
        .bind(&item.item_type)
        .bind(&item.inventory_id)
        .bind(&item.service_id)
        .bind(&item.item_name)
        .bind(item.quantity)
        .bind(item.unit_price)
        .bind(line)
        .execute(&mut *tx)
        .await;
    }

    let _ = sqlx::query(
        r#"INSERT INTO payments
           (id, order_id, payment_method, payment_status, amount, currency, razorpay_order_id)
           VALUES ($1,$2,'RAZORPAY','PENDING',$3,'INR',$4)"#,
    )
    .bind(&payment_id)
    .bind(&order_id)
    .bind(total)
    .bind(&rzp_order_id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = tx.commit().await {
        eprintln!("Order commit error: {e}");
        return Json(json!({"success": false, "error": "Failed to save order."}));
    }

    Json(json!({
        "success": true,
        "order_id": order_id,
        "invoice_number": invoice_number,
        "razorpay_order_id": rzp_order_id,
        "amount_paise": amount_paise,
        "billing_type": billing_type,
    }))
}

pub async fn verify_payment(
    State(state): State<AppState>,
    Json(req): Json<VerifyPaymentRequest>,
) -> Json<Value> {
    let rzp_key_secret = std::env::var("RAZORPAY_KEY_SECRET").unwrap_or_default();
    if rzp_key_secret.is_empty() {
        return Json(json!({"success": false, "error": "Razorpay not configured."}));
    }

    // Verify HMAC-SHA256 signature
    let payload = format!("{}|{}", req.razorpay_order_id, req.razorpay_payment_id);
    let valid = verify_razorpay_signature(&rzp_key_secret, &payload, &req.razorpay_signature);
    if !valid {
        eprintln!(
            "Razorpay signature verification FAILED for order {}",
            req.order_id
        );
        return Json(json!({"success": false, "error": "Payment verification failed."}));
    }

    // Begin transaction with row lock
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Verify TX error: {e}");
            return Json(json!({"success": false, "error": "Database error."}));
        }
    };

    // Lock and check order
    let order = sqlx::query(
        "SELECT id, total, payment_status, billing_type FROM orders WHERE id = $1 FOR UPDATE",
    )
    .bind(&req.order_id)
    .fetch_optional(&mut *tx)
    .await
    .unwrap_or(None);

    let Some(order) = order else {
        return Json(json!({"success": false, "error": "Order not found."}));
    };

    let pay_status: String = order.get("payment_status");

    // Idempotency: already paid
    if pay_status == "PAID" {
        return Json(json!({"success": true, "already_paid": true}));
    }

    // Check for duplicate razorpay_payment_id (idempotency)
    let dup = sqlx::query("SELECT id FROM payments WHERE razorpay_payment_id = $1")
        .bind(&req.razorpay_payment_id)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None);

    if dup.is_some() {
        return Json(json!({"success": true, "already_paid": true}));
    }

    // Mark order PAID
    let _ = sqlx::query(
        "UPDATE orders SET payment_status = 'PAID', order_status = 'CONFIRMED', updated_at = NOW() WHERE id = $1",
    )
    .bind(&req.order_id)
    .execute(&mut *tx)
    .await;

    // Update payment record
    let _ = sqlx::query(
        r#"UPDATE payments
           SET payment_status = 'PAID',
               razorpay_order_id = $1,
               razorpay_payment_id = $2,
               razorpay_signature = $3,
               paid_at = NOW()
           WHERE order_id = $4 AND payment_status = 'PENDING'"#,
    )
    .bind(&req.razorpay_order_id)
    .bind(&req.razorpay_payment_id)
    .bind(&req.razorpay_signature)
    .bind(&req.order_id)
    .execute(&mut *tx)
    .await;

    // Reduce tyre stock
    let tyre_items = sqlx::query(
        "SELECT inventory_id, item_name, quantity FROM order_items WHERE order_id = $1 AND item_type = 'TYRE'",
    )
    .bind(&req.order_id)
    .fetch_all(&mut *tx)
    .await
    .unwrap_or_default();

    let tyre_data: Vec<(String, String, i32)> = tyre_items
        .iter()
        .filter_map(|r| {
            let inv_id: Option<String> = r.try_get("inventory_id").ok().flatten();
            let name: String = r.get("item_name");
            let qty: i32 = r.get("quantity");
            inv_id.map(|id| (id, name, qty))
        })
        .collect();

    for (inv_id, item_name, qty) in &tyre_data {
        let tyre = sqlx::query("SELECT id, stock FROM inventory WHERE id = $1 FOR UPDATE")
            .bind(inv_id)
            .fetch_optional(&mut *tx)
            .await
            .unwrap_or(None);

        if let Some(tyre) = tyre {
            let stock_before: i32 = tyre.get("stock");
            let stock_after = (stock_before - qty).max(0);

            let _ =
                sqlx::query("UPDATE inventory SET stock = $1, updated_at = NOW() WHERE id = $2")
                    .bind(stock_after)
                    .bind(inv_id)
                    .execute(&mut *tx)
                    .await;

            let mv_id = format!("STK-{}-{}", chrono::Utc::now().timestamp_millis(), inv_id);
            let _ = sqlx::query(
                r#"INSERT INTO stock_movements
                   (id, tyre_id, tyre_name, movement_type, quantity, stock_before, stock_after, reference, notes)
                   VALUES ($1,$2,$3,'Stock Out',$4,$5,$6,$7,$8)"#,
            )
            .bind(&mv_id)
            .bind(inv_id)
            .bind(item_name)
            .bind(-qty)
            .bind(stock_before)
            .bind(stock_after)
            .bind(&req.order_id)
            .bind(format!("Online sale - Razorpay {}", req.razorpay_payment_id))
            .execute(&mut *tx)
            .await;
        }
    }

    if let Err(e) = tx.commit().await {
        eprintln!("Verify commit error: {e}");
        return Json(json!({"success": false, "error": "Failed to confirm payment."}));
    }

    eprintln!(
        "Payment verified and committed: order={} rzp_payment={}",
        req.order_id, req.razorpay_payment_id
    );

    Json(json!({"success": true}))
}

// Create a Razorpay order via their REST API
async fn create_razorpay_order(
    key_id: &str,
    key_secret: &str,
    amount_paise: i64,
    receipt: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "amount": amount_paise,
        "currency": "INR",
        "receipt": receipt,
    });

    let resp = client
        .post("https://api.razorpay.com/v1/orders")
        .basic_auth(key_id, Some(key_secret))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: Value = resp.json().await.map_err(|e| e.to_string())?;

    data["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No id in Razorpay response: {:?}", data))
}

fn verify_razorpay_signature(secret: &str, payload: &str, signature: &str) -> bool {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload.as_bytes());
    let result = mac.finalize().into_bytes();
    let computed = hex::encode(result);
    computed == signature
}
