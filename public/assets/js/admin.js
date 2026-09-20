document.addEventListener("DOMContentLoaded", () => {

    const menuButton =
        document.getElementById("adminMenuButton") ||
        document.querySelector(".admin-menu-button");
    const sidebar = document.querySelector(".admin-sidebar");

    if (menuButton && sidebar) {
        menuButton.addEventListener("click", () => {
            sidebar.classList.toggle("open");
        });
    }

    const customerSelect = document.getElementById("jobCustomer");
    const vehicleSelect = document.getElementById("jobVehicle");
    const registrationInput = document.getElementById("jobRegistration");

    const summaryCustomer = document.getElementById("summaryCustomer");
    const summaryVehicle = document.getElementById("summaryVehicle");

    function syncCustomerVehicle() {

        if (!customerSelect) {
            return;
        }

        const selected =
            customerSelect.options[customerSelect.selectedIndex];

        if (!selected || !selected.value) {
            return;
        }

        const vehicle = selected.dataset.vehicle || "";
        const registration = selected.dataset.registration || "";

        if (vehicleSelect && vehicle) {
            vehicleSelect.value = vehicle;
        }

        if (registrationInput && registration) {
            registrationInput.value = registration;
        }

        if (summaryCustomer) {
            summaryCustomer.textContent =
                selected.textContent.trim();
        }

        if (summaryVehicle && vehicle) {
            summaryVehicle.textContent = vehicle;
        }
    }

    if (customerSelect) {
        customerSelect.addEventListener(
            "change",
            syncCustomerVehicle
        );
    }

    const tyreInputs =
        document.querySelectorAll(".job-tyre");

    const serviceInputs =
        document.querySelectorAll(".job-service");

    const tyreTotalElement =
        document.getElementById("tyreTotal");

    const serviceTotalElement =
        document.getElementById("serviceTotal");

    const grandTotalElement =
        document.getElementById("grandTotal");

    const summaryItems =
        document.getElementById("jobItems");

    function money(value) {
        return "₹" + Number(value).toLocaleString("en-IN");
    }

    function updateCustomer() {

        if (!customerSelect || !summaryCustomer) {
            return;
        }

        const selected =
            customerSelect.options[customerSelect.selectedIndex];

        summaryCustomer.textContent =
            selected && selected.value
                ? selected.textContent.trim()
                : "-";
    }

    function updateVehicle() {

        if (!vehicleSelect || !summaryVehicle) {
            return;
        }

        summaryVehicle.textContent =
            vehicleSelect.value || "-";
    }

    function updateSummary() {

        if (
            !summaryItems ||
            !tyreTotalElement ||
            !serviceTotalElement ||
            !grandTotalElement
        ) {
            return;
        }

        let tyreTotal = 0;
        let serviceTotal = 0;

        const items = [];

        tyreInputs.forEach(input => {

            if (!input.checked) {
                return;
            }

            const card =
                input.closest(".job-product-card");

            const quantityInput =
                card?.querySelector(".qty-input");

            const quantity =
                Math.max(
                    1,
                    Number(quantityInput?.value || 1)
                );

            const price =
                Number(input.dataset.price || 0);

            const name =
                input.dataset.name || "";

            tyreTotal += price * quantity;

            items.push({
                name: name,
                price: price,
                quantity: quantity,
                item_type: "Tyre"
            });
        });

        serviceInputs.forEach(input => {

            if (!input.checked) {
                return;
            }

            const price =
                Number(input.dataset.price || 0);

            const name =
                input.dataset.name || "";

            serviceTotal += price;

            items.push({
                name: name,
                price: price,
                quantity: 1,
                item_type: "Service"
            });
        });

        const grandTotal =
            tyreTotal + serviceTotal;

        tyreTotalElement.textContent =
            money(tyreTotal);

        serviceTotalElement.textContent =
            money(serviceTotal);

        grandTotalElement.textContent =
            money(grandTotal);

        summaryItems.value =
            JSON.stringify(items);

        updateCustomer();
        updateVehicle();
    }

    tyreInputs.forEach(input => {

        input.addEventListener(
            "change",
            updateSummary
        );

        const card =
            input.closest(".job-product-card");

        const minus =
            card?.querySelector(".qty-minus");

        const plus =
            card?.querySelector(".qty-plus");

        const quantityInput =
            card?.querySelector(".qty-input");

        if (minus && quantityInput) {
            minus.addEventListener("click", event => {

                event.preventDefault();

                const current =
                    Number(quantityInput.value || 1);

                quantityInput.value =
                    Math.max(1, current - 1);

                updateSummary();
            });
        }

        if (plus && quantityInput) {
            plus.addEventListener("click", event => {

                event.preventDefault();

                const current =
                    Number(quantityInput.value || 1);

                quantityInput.value =
                    current + 1;

                updateSummary();
            });
        }

        if (quantityInput) {
            quantityInput.addEventListener(
                "input",
                updateSummary
            );
        }
    });

    serviceInputs.forEach(input => {
        input.addEventListener(
            "change",
            updateSummary
        );
    });

    if (vehicleSelect) {
        vehicleSelect.addEventListener(
            "change",
            updateVehicle
        );
    }

    syncCustomerVehicle();
    updateSummary();
});
