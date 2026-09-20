/* =========================================================
   SHRI KRISHNA TYRE HOUSE
   PUBLIC WEBSITE JAVASCRIPT
   ========================================================= */

document.addEventListener("DOMContentLoaded", () => {

    /* -----------------------------------------------------
       MOBILE MENU
       ----------------------------------------------------- */

    const mobileMenuButton =
        document.getElementById("mobileMenuButton");

    const siteNav =
        document.getElementById("siteNav");

    if (mobileMenuButton && siteNav) {

        mobileMenuButton.addEventListener("click", () => {

            siteNav.classList.toggle("open");

        });

    }


    /* -----------------------------------------------------
       CLOSE MOBILE MENU AFTER LINK CLICK
       ----------------------------------------------------- */

    if (siteNav) {

        const navLinks =
            siteNav.querySelectorAll("a");

        navLinks.forEach((link) => {

            link.addEventListener("click", () => {

                siteNav.classList.remove("open");

            });

        });

    }


    /* -----------------------------------------------------
       TYRE SEARCH
       Uses /tyres instead of old .html files
       ----------------------------------------------------- */

    const searchButton =
        document.getElementById("searchTyres");

    if (searchButton) {

        searchButton.addEventListener("click", () => {

            const vehicle =
                document.getElementById("vehicleType")?.value || "";

            const size =
                document.getElementById("tyreSize")?.value.trim() || "";

            const brand =
                document.getElementById("brand")?.value || "";

            const params =
                new URLSearchParams();

            if (vehicle) {
                params.set("vehicle", vehicle);
            }

            if (size) {
                params.set("size", size);
            }

            if (brand) {
                params.set("brand", brand);
            }

            const query =
                params.toString();

            window.location.href =
                query
                    ? `/tyres?${query}`
                    : "/tyres";

        });

    }


    /* -----------------------------------------------------
       CONTACT FORM
       Frontend-only for now.
       Backend/database will be connected later.
       ----------------------------------------------------- */

    const contactForm =
        document.getElementById("contactForm");

    const contactStatus =
        document.getElementById("contactStatus");

    if (contactForm) {

        contactForm.addEventListener("submit", (event) => {

            event.preventDefault();

            if (contactStatus) {

                contactStatus.textContent =
                    "Thank you. Your message has been received.";

            }

            contactForm.reset();

        });

    }


    /* -----------------------------------------------------
       CURRENT YEAR
       ----------------------------------------------------- */

    const yearElements =
        document.querySelectorAll("[data-current-year]");

    yearElements.forEach((element) => {

        element.textContent =
            new Date().getFullYear();

    });

});
