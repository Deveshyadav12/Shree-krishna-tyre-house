use axum::{extract::State, response::Html};
use sqlx::Row;

use crate::pages::layout::page;
use crate::AppState;

pub async fn cart(State(state): State<AppState>) -> Html<String> {
    // Load all in-stock tyres and active services for the cart page
    let tyres = sqlx::query(
        "SELECT id, brand, name, size, tyre_type, price, stock FROM inventory WHERE stock > 0 ORDER BY brand, name",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let services = sqlx::query(
        "SELECT id, name, description, price FROM services WHERE active = TRUE ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut tyre_rows = String::new();
    for row in &tyres {
        let id: String = row.get("id");
        let brand: String = row.get("brand");
        let name: String = row.get("name");
        let size: String = row.get("size");
        let tyre_type: String = row.get("tyre_type");
        let price: i64 = row.get("price");
        tyre_rows.push_str(&format!(
            r#"<div class="cart-product-card" data-id="{id}" data-type="TYRE" data-price="{price}" data-name="{label}">
                <div class="cart-product-info">
                    <div class="cart-product-brand">{brand}</div>
                    <div class="cart-product-name">{name}</div>
                    <div class="cart-product-meta">{size} &bull; {tyre_type}</div>
                </div>
                <div class="cart-product-right">
                    <div class="cart-product-price">&#8377;{price}</div>
                    <button class="cart-add-btn" onclick="cartAdd('{id}','TYRE','{label}',{price})">Add to Cart</button>
                </div>
            </div>"#,
            id = esc(&id),
            brand = esc(&brand),
            name = esc(&name),
            size = esc(&size),
            tyre_type = esc(&tyre_type),
            price = price,
            label = esc(&format!("{} {} {}", brand, name, size)),
        ));
    }

    let mut svc_rows = String::new();
    for row in &services {
        let id: String = row.get("id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let price: i64 = row.get("price");
        svc_rows.push_str(&format!(
            r#"<div class="cart-product-card" data-id="{id}" data-type="SERVICE" data-price="{price}" data-name="{name}">
                <div class="cart-product-info">
                    <div class="cart-product-name">{name}</div>
                    <div class="cart-product-meta">{desc}</div>
                </div>
                <div class="cart-product-right">
                    <div class="cart-product-price">&#8377;{price}</div>
                    <button class="cart-add-btn" onclick="cartAdd('{id}','SERVICE','{name}',{price})">Add</button>
                </div>
            </div>"#,
            id = esc(&id),
            name = esc(&name),
            desc = esc(&description),
            price = price,
        ));
    }

    let content = format!(
        r#"
<section class="page-hero">
    <div class="container">
        <span class="eyebrow">SHOPPING CART</span>
        <h1>Your Cart</h1>
        <p>Add tyres and services, then proceed to checkout.</p>
    </div>
</section>

<section class="section">
    <div class="container">
        <div class="cart-layout">

            <!-- LEFT: Products to add -->
            <div class="cart-products-col">

                <div class="cart-section-head">
                    <span class="eyebrow">TYRES</span>
                    <h2>Available Tyres</h2>
                </div>
                <div class="cart-product-list">
                    {tyre_rows}
                </div>

                <div class="cart-section-head" style="margin-top:32px;">
                    <span class="eyebrow">SERVICES</span>
                    <h2>Available Services</h2>
                </div>
                <div class="cart-product-list">
                    {svc_rows}
                </div>

            </div>

            <!-- RIGHT: Cart summary -->
            <div class="cart-summary-col">
                <div class="cart-summary-card">
                    <div class="cart-summary-head">
                        <h2>Cart</h2>
                        <button class="cart-clear-btn" onclick="cartClear()">Clear</button>
                    </div>
                    <div id="cartItems" class="cart-items-list">
                        <div class="cart-empty-msg">Your cart is empty.</div>
                    </div>
                    <div class="cart-totals">
                        <div class="cart-total-row">
                            <span>Subtotal</span>
                            <strong id="cartSubtotal">&#8377;0</strong>
                        </div>
                        <div class="cart-total-row cart-grand">
                            <span>Total</span>
                            <strong id="cartTotal">&#8377;0</strong>
                        </div>
                    </div>
                    <a href="/checkout" class="cart-checkout-btn" id="cartCheckoutBtn">
                        Proceed to Checkout &rarr;
                    </a>
                </div>
            </div>

        </div>
    </div>
</section>

<script>
// Cart stored in localStorage as JSON array of {{id, type, name, price, qty}}
function cartGet() {{
    try {{ return JSON.parse(localStorage.getItem('sk_cart') || '[]'); }} catch(e) {{ return []; }}
}}
function cartSave(items) {{
    localStorage.setItem('sk_cart', JSON.stringify(items));
}}
function cartAdd(id, type, name, price) {{
    var items = cartGet();
    var existing = items.find(function(i) {{ return i.id === id; }});
    if (existing) {{
        if (type === 'TYRE') existing.qty = (existing.qty || 1) + 1;
    }} else {{
        items.push({{id: id, type: type, name: name, price: parseInt(price), qty: 1}});
    }}
    cartSave(items);
    cartRender();
}}
function cartRemove(id) {{
    var items = cartGet().filter(function(i) {{ return i.id !== id; }});
    cartSave(items);
    cartRender();
}}
function cartQty(id, delta) {{
    var items = cartGet();
    var item = items.find(function(i) {{ return i.id === id; }});
    if (item) {{
        item.qty = Math.max(1, (item.qty || 1) + delta);
        if (item.type === 'SERVICE') item.qty = 1;
    }}
    cartSave(items);
    cartRender();
}}
function cartClear() {{
    cartSave([]);
    cartRender();
}}
function cartRender() {{
    var items = cartGet();
    var el = document.getElementById('cartItems');
    if (!el) return;
    if (items.length === 0) {{
        el.innerHTML = '<div class="cart-empty-msg">Your cart is empty.</div>';
        document.getElementById('cartSubtotal').innerHTML = '&#8377;0';
        document.getElementById('cartTotal').innerHTML = '&#8377;0';
        return;
    }}
    var html = '';
    var total = 0;
    items.forEach(function(item) {{
        var line = item.price * (item.qty || 1);
        total += line;
        html += '<div class="cart-item">' +
            '<div class="cart-item-info"><strong>' + item.name + '</strong><small>' + item.type + '</small></div>' +
            '<div class="cart-item-controls">';
        if (item.type === 'TYRE') {{
            html += '<button onclick="cartQty(\'' + item.id + '\',-1)">-</button>' +
                '<span>' + (item.qty||1) + '</span>' +
                '<button onclick="cartQty(\'' + item.id + '\',1)">+</button>';
        }}
        html += '</div>' +
            '<div class="cart-item-price">&#8377;' + line + '</div>' +
            '<button class="cart-item-remove" onclick="cartRemove(\'' + item.id + '\')">&#10005;</button>' +
            '</div>';
    }});
    el.innerHTML = html;
    document.getElementById('cartSubtotal').innerHTML = '&#8377;' + total;
    document.getElementById('cartTotal').innerHTML = '&#8377;' + total;
}}
document.addEventListener('DOMContentLoaded', cartRender);
</script>
"#,
        tyre_rows = tyre_rows,
        svc_rows = svc_rows,
    );

    Html(page("Cart", &content))
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
