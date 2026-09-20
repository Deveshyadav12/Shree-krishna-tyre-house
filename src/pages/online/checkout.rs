use axum::{extract::State, response::Html};
use sqlx::Row;

use crate::pages::layout::page;
use crate::AppState;

pub async fn checkout(State(state): State<AppState>) -> Html<String> {
    // Load Razorpay key (public key only — safe to expose)
    let razorpay_key = std::env::var("RAZORPAY_KEY_ID").unwrap_or_default();

    // Load tyres and services so JS can validate prices client-side display
    // (server always re-reads prices on order creation)
    let tyres = sqlx::query(
        "SELECT id, price FROM inventory WHERE stock > 0",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let services = sqlx::query(
        "SELECT id, price FROM services WHERE active = TRUE",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut price_map = String::new();
    for row in &tyres {
        let id: String = row.get("id");
        let price: i64 = row.get("price");
        price_map.push_str(&format!("'{id}':{price},", id = id, price = price));
    }
    for row in &services {
        let id: String = row.get("id");
        let price: i64 = row.get("price");
        price_map.push_str(&format!("'{id}':{price},", id = id, price = price));
    }

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">CHECKOUT</span>
        <h1>Complete Your Order</h1>
        <p>Review your cart and enter your details to pay.</p>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="checkout-layout">

            <!-- ORDER SUMMARY -->
            <div class="checkout-summary-col">
                <div class="checkout-panel">
                    <h2>Order Summary</h2>
                    <div id="checkoutItems" class="checkout-items"></div>
                    <div class="checkout-totals">
                        <div class="checkout-total-row">
                            <span>Total</span>
                            <strong id="checkoutTotal">&#8377;0</strong>
                        </div>
                    </div>
                </div>
            </div>

            <!-- CUSTOMER DETAILS -->
            <div class="checkout-form-col">
                <div class="checkout-panel">
                    <h2>Your Details</h2>
                    <div id="checkoutError" class="checkout-error" style="display:none;"></div>

                    <div class="checkout-form-group">
                        <label>Full Name *</label>
                        <input type="text" id="coName" placeholder="Rajesh Kumar" required>
                    </div>
                    <div class="checkout-form-group">
                        <label>Phone Number *</label>
                        <input type="tel" id="coPhone" placeholder="9876543210" required>
                    </div>
                    <div class="checkout-form-group">
                        <label>Email</label>
                        <input type="email" id="coEmail" placeholder="rajesh@example.com">
                    </div>

                    <button id="payBtn" class="checkout-pay-btn" onclick="startPayment()">
                        Pay with Razorpay
                    </button>
                    <p style="margin-top:12px;text-align:center;color:#64748b;font-size:12px;">
                        Secure payment powered by Razorpay
                    </p>
                </div>
            </div>

        </div>
    </div>
</section>

<script src="https://checkout.razorpay.com/v1/checkout.js"></script>
<script>
var RAZORPAY_KEY = '{razorpay_key}';
var SERVER_PRICES = {{{price_map}}};

function cartGet() {{
    try {{ return JSON.parse(localStorage.getItem('sk_cart') || '[]'); }} catch(e) {{ return []; }}
}}

function renderCheckout() {{
    var items = cartGet();
    var el = document.getElementById('checkoutItems');
    if (!el) return;
    if (items.length === 0) {{
        el.innerHTML = '<p style="color:#64748b;">Your cart is empty. <a href="/cart">Go back to cart</a>.</p>';
        return;
    }}
    var html = '';
    var total = 0;
    items.forEach(function(item) {{
        var serverPrice = SERVER_PRICES[item.id] || item.price;
        var line = serverPrice * (item.qty || 1);
        total += line;
        html += '<div class="checkout-item">' +
            '<div><strong>' + item.name + '</strong><small>' + item.type + (item.qty > 1 ? ' x' + item.qty : '') + '</small></div>' +
            '<div>&#8377;' + line + '</div></div>';
    }});
    el.innerHTML = html;
    document.getElementById('checkoutTotal').innerHTML = '&#8377;' + total;
}}

async function startPayment() {{
    var items = cartGet();
    if (items.length === 0) {{
        showError('Your cart is empty.');
        return;
    }}
    var name = document.getElementById('coName').value.trim();
    var phone = document.getElementById('coPhone').value.trim();
    var email = document.getElementById('coEmail').value.trim();
    if (!name || !phone) {{
        showError('Please enter your name and phone number.');
        return;
    }}

    document.getElementById('payBtn').disabled = true;
    document.getElementById('payBtn').textContent = 'Creating order...';

    try {{
        var resp = await fetch('/api/orders/create', {{
            method: 'POST',
            headers: {{'Content-Type': 'application/json'}},
            body: JSON.stringify({{
                customer_name: name,
                customer_phone: phone,
                customer_email: email,
                items: items
            }})
        }});
        var data = await resp.json();
        if (!data.success) {{
            showError(data.error || 'Failed to create order.');
            resetBtn();
            return;
        }}

        var options = {{
            key: RAZORPAY_KEY,
            amount: data.amount_paise,
            currency: 'INR',
            name: 'Shri Krishna Tyre House',
            description: data.billing_type,
            order_id: data.razorpay_order_id,
            prefill: {{ name: name, contact: phone, email: email }},
            theme: {{ color: '#111827' }},
            handler: async function(response) {{
                document.getElementById('payBtn').textContent = 'Verifying payment...';
                try {{
                    var vresp = await fetch('/api/payments/verify', {{
                        method: 'POST',
                        headers: {{'Content-Type': 'application/json'}},
                        body: JSON.stringify({{
                            razorpay_order_id: response.razorpay_order_id,
                            razorpay_payment_id: response.razorpay_payment_id,
                            razorpay_signature: response.razorpay_signature,
                            order_id: data.order_id
                        }})
                    }});
                    var vdata = await vresp.json();
                    if (vdata.success) {{
                        localStorage.removeItem('sk_cart');
                        window.location.href = '/order/success/' + data.order_id;
                    }} else {{
                        showError('Payment verification failed. Please contact us.');
                        resetBtn();
                    }}
                }} catch(e) {{
                    showError('Verification error. Please contact us.');
                    resetBtn();
                }}
            }},
            modal: {{
                ondismiss: function() {{ resetBtn(); }}
            }}
        }};
        var rzp = new Razorpay(options);
        rzp.open();
    }} catch(e) {{
        showError('Network error. Please try again.');
        resetBtn();
    }}
}}

function showError(msg) {{
    var el = document.getElementById('checkoutError');
    el.textContent = msg;
    el.style.display = 'block';
}}
function resetBtn() {{
    var btn = document.getElementById('payBtn');
    btn.disabled = false;
    btn.textContent = 'Pay with Razorpay';
}}

document.addEventListener('DOMContentLoaded', renderCheckout);
</script>
"#,
        razorpay_key = razorpay_key,
        price_map = price_map,
    );

    Html(page("Checkout", &content))
}
