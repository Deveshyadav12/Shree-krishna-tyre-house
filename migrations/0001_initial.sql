CREATE TABLE IF NOT EXISTS customers (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    phone VARCHAR(30) NOT NULL,
    email VARCHAR(150) NOT NULL DEFAULT '',
    address TEXT NOT NULL DEFAULT '',
    vehicle VARCHAR(150) NOT NULL DEFAULT '',
    registration VARCHAR(50) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS inventory (
    id VARCHAR(50) PRIMARY KEY,
    brand VARCHAR(100) NOT NULL,
    name VARCHAR(150) NOT NULL,
    size VARCHAR(80) NOT NULL,
    tyre_type VARCHAR(80) NOT NULL,
    price BIGINT NOT NULL DEFAULT 0,
    stock INTEGER NOT NULL DEFAULT 0,
    low_stock_limit INTEGER NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS services (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    price BIGINT NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS brands (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS jobs (
    id VARCHAR(50) PRIMARY KEY,
    customer VARCHAR(150) NOT NULL,
    vehicle VARCHAR(150) NOT NULL,
    vehicle_type VARCHAR(100) NOT NULL,
    registration VARCHAR(50) NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    status VARCHAR(50) NOT NULL DEFAULT 'NEW',
    total BIGINT NOT NULL DEFAULT 0,
    payment_status VARCHAR(50) NOT NULL DEFAULT 'UNPAID',
    payment_method VARCHAR(50) NOT NULL DEFAULT '-',
    invoice_id VARCHAR(50) NOT NULL DEFAULT '-',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS job_items (
    id BIGSERIAL PRIMARY KEY,
    job_id VARCHAR(50) NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    name VARCHAR(150) NOT NULL,
    price BIGINT NOT NULL DEFAULT 0,
    quantity INTEGER NOT NULL DEFAULT 1,
    item_type VARCHAR(50) NOT NULL
);

CREATE TABLE IF NOT EXISTS stock_movements (
    id VARCHAR(50) PRIMARY KEY,
    tyre_id VARCHAR(50) NOT NULL,
    tyre_name VARCHAR(150) NOT NULL,
    movement_type VARCHAR(80) NOT NULL,
    quantity INTEGER NOT NULL,
    stock_before INTEGER NOT NULL,
    stock_after INTEGER NOT NULL,
    reference VARCHAR(100) NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_jobs_customer
ON jobs(customer);

CREATE INDEX IF NOT EXISTS idx_jobs_status
ON jobs(status);

CREATE INDEX IF NOT EXISTS idx_jobs_payment_status
ON jobs(payment_status);

CREATE INDEX IF NOT EXISTS idx_job_items_job_id
ON job_items(job_id);

CREATE INDEX IF NOT EXISTS idx_stock_movements_tyre_id
ON stock_movements(tyre_id);

CREATE INDEX IF NOT EXISTS idx_stock_movements_created_at
ON stock_movements(created_at);
