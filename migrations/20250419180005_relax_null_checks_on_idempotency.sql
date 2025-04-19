ALTER TABLE idempotency ALTER column response_status_code DROP NOT NULL;
ALTER TABLE idempotency ALTER column response_body DROP NOT NULL;
ALTER TABLE idempotency ALTER column response_headers DROP NOT NULL;
