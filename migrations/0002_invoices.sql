CREATE TABLE IF NOT EXISTS invoices (
    id VARCHAR(50) PRIMARY KEY,
    job_id VARCHAR(50) NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    customer VARCHAR(150) NOT NULL,
    vehicle VARCHAR(150) NOT NULL,
    vehicle_type VARCHAR(100) NOT NULL,
    registration VARCHAR(50) NOT NULL,
    total BIGINT NOT NULL DEFAULT 0,
    payment_method VARCHAR(50) NOT NULL DEFAULT 'Cash',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_invoices_job_id ON invoices(job_id);
CREATE INDEX IF NOT EXISTS idx_invoices_created_at ON invoices(created_at);

ALTER TABLE inventory ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
