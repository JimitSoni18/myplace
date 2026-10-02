INSERT INTO residential_property_details (
	property_id, bedroom_count, bathroom_count, balcony_count, is_duplex, parking
)
VALUES ($1, ($2::float8)::numeric, ($3::float8)::numeric, $4, $5, $6);
