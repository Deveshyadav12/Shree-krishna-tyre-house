pub fn page(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">

    <title>{} | Shri Krishna Tyre House</title>

    <meta
        name="description"
        content="Shri Krishna Tyre House - Quality tyres and professional tyre services."
    >

    <link rel="stylesheet" href="/assets/css/site.css">
</head>

<body>

<header class="site-header">

    <div class="container nav-container">

        <a href="/" class="logo">

            <span class="logo-mark">SK</span>

            <span class="logo-text">
                <strong>SHRI KRISHNA</strong>
                <small>TYRE HOUSE</small>
            </span>

        </a>

        <button
            class="mobile-menu-button"
            id="mobileMenuButton"
            aria-label="Open navigation"
        >
            ☰
        </button>

        <nav class="site-nav" id="siteNav">

            <a href="/">Home</a>
            <a href="/tyres">Tyres</a>
            <a href="/services">Services</a>
            <a href="/brands">Brands</a>
            <a href="/about">About</a>
            <a href="/contact">Contact</a>

            <a href="/tyres" class="nav-cta">
                Shop Tyres
            </a>

        </nav>

    </div>

</header>

<main>
{}
</main>

<footer class="site-footer">

    <div class="container footer-grid">

        <div class="footer-brand">

            <div class="logo">

                <span class="logo-mark">SK</span>

                <span class="logo-text">
                    <strong>SHRI KRISHNA</strong>
                    <small>TYRE HOUSE</small>
                </span>

            </div>

            <p>
                Quality tyres. Professional service.
                Confident driving.
            </p>

        </div>

        <div>

            <h4>Quick Links</h4>

            <a href="/">Home</a>
            <a href="/tyres">Tyres</a>
            <a href="/services">Services</a>
            <a href="/brands">Brands</a>

        </div>

        <div>

            <h4>Company</h4>

            <a href="/about">About</a>
            <a href="/contact">Contact</a>

        </div>

        <div>

            <h4>Visit Us</h4>

            <p>
                Singhana Road,<br>
                Buhana, Jhunjhunu,<br>
                Rajasthan 333502
            </p>

        </div>

    </div>

    <div class="footer-bottom">

        <div class="container">
            © 2026 Shri Krishna Tyre House. All rights reserved.
        </div>

    </div>

</footer>

<script src="/assets/js/site.js"></script>

</body>
</html>"#,
        title, content
    )
}
