INSERT INTO property_listings (
	property_id, listing_type, status, currency_code, price, billing_period
)
VALUES ($1, $2, $3, 'INR', ($4::float8)::numeric, $5);
