UPDATE property_listings
SET listing_type = $1, status = $2, price = ($3::float8)::numeric, billing_period = $4, updated_at = NOW()
WHERE property_id = $5;
