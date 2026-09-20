pub fn admin_page(title: &str, content: &str) -> String {
    let dashboard = if title == "Dashboard" { " active" } else { "" };
    let inventory =
        if title.contains("Inventory") || title.contains("Tyre") || title.contains("Stock") {
            " active"
        } else {
            ""
        };
    let services = if title == "Services" { " active" } else { "" };
    let brands = if title == "Brands" { " active" } else { "" };
    let customers = if title.contains("Customer") {
        " active"
    } else {
        ""
    };
    let jobs = if title.contains("Job") { " active" } else { "" };
    let billing = if title == "Billing" { " active" } else { "" };
    let invoices = if title == "Invoices" { " active" } else { "" };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">

<head>

    <meta charset="UTF-8">

    <meta
        name="viewport"
        content="width=device-width, initial-scale=1.0"
    >

    <title>{title} | Shri Krishna Tyre House</title>

    <link
        rel="stylesheet"
        href="/assets/css/admin.css"
    >

</head>


<body class="admin-body">


<div class="admin-layout">


    <!-- SIDEBAR -->

    <aside class="admin-sidebar" id="adminSidebar">


        <div class="admin-sidebar-brand">

            <div class="admin-brand-logo">
                SK
            </div>

            <div class="admin-brand-text">

                <strong>
                    SHRI KRISHNA
                </strong>

                <span>
                    TYRE HOUSE
                </span>

            </div>

        </div>


        <!-- MAIN NAVIGATION -->

        <div class="admin-sidebar-section">

            <div class="admin-sidebar-label">
                MAIN
            </div>


            <a
                href="/admin"
                class="admin-nav-link{dashboard}"
            >

                <span class="admin-nav-icon">
                    DB
                </span>

                <span>
                    Dashboard
                </span>

            </a>


            <a
                href="/admin/inventory"
                class="admin-nav-link{inventory}"
            >

                <span class="admin-nav-icon">
                    TY
                </span>

                <span>
                    Tyre Inventory
                </span>

            </a>


            <a
                href="/admin/services"
                class="admin-nav-link{services}"
            >

                <span class="admin-nav-icon">
                    SV
                </span>

                <span>
                    Services
                </span>

            </a>


            <a
                href="/admin/brands"
                class="admin-nav-link{brands}"
            >

                <span class="admin-nav-icon">
                    BR
                </span>

                <span>
                    Brands
                </span>

            </a>


            <a
                href="/admin/customers"
                class="admin-nav-link{customers}"
            >

                <span class="admin-nav-icon">
                    CU
                </span>

                <span>
                    Customers
                </span>

            </a>

        </div>


        <!-- OPERATIONS -->

        <div class="admin-sidebar-section">

            <div class="admin-sidebar-label">
                OPERATIONS
            </div>


            <a
                href="/admin/jobs"
                class="admin-nav-link{jobs}"
            >

                <span class="admin-nav-icon">
                    JB
                </span>

                <span>
                    Job Orders
                </span>

            </a>


            <a
                href="/admin/billing"
                class="admin-nav-link{billing}"
            >

                <span class="admin-nav-icon">
                    BL
                </span>

                <span>
                    Billing
                </span>

            </a>


            <a
                href="/admin/invoices"
                class="admin-nav-link{invoices}"
            >

                <span class="admin-nav-icon">
                    IN
                </span>

                <span>
                    Invoices
                </span>

            </a>

        </div>


        <!-- BOTTOM -->

        <div class="admin-sidebar-bottom">

            <a
                href="/"
                class="admin-nav-link"
            >

                <span class="admin-nav-icon">
                    WE
                </span>

                <span>
                    View Website
                </span>

            </a>


            <a
                href="/admin/login"
                class="admin-nav-link"
            >

                <span class="admin-nav-icon">
                    LO
                </span>

                <span>
                    Logout
                </span>

            </a>

        </div>


    </aside>


    <!-- MAIN AREA -->

    <div class="admin-main">


        <!-- TOPBAR -->

        <header class="admin-topbar">


            <div class="admin-topbar-left">


                <button
                    type="button"
                    class="admin-menu-button"
                    id="adminMenuButton"
                >
                    MENU
                </button>


                <div class="admin-breadcrumb">

                    <span>
                        ADMIN
                    </span>

                    <strong>
                        /
                    </strong>

                    <span>
                        {title}
                    </span>

                </div>


            </div>


            <div class="admin-topbar-right">


                <div class="admin-online">

                    <span class="admin-online-dot"></span>

                    Online

                </div>


                <div class="admin-user">

                    <div class="admin-user-avatar">
                        A
                    </div>


                    <div class="admin-user-info">

                        <strong>
                            Administrator
                        </strong>

                        <small>
                            Admin
                        </small>

                    </div>

                </div>


            </div>


        </header>


        <!-- PAGE CONTENT -->

        <main class="admin-content">

            {content}

        </main>


        <!-- FOOTER -->

        <footer class="admin-footer">

            <span>
                Shri Krishna Tyre House Admin Panel
            </span>

            <span>
                © 2026
            </span>

        </footer>


    </div>


</div>


<script src="/assets/js/admin.js"></script>


</body>

</html>"#,
        title = title,
        content = content,
        dashboard = dashboard,
        inventory = inventory,
        services = services,
        brands = brands,
        customers = customers,
        jobs = jobs,
        billing = billing,
        invoices = invoices
    )
}
