UPDATE residential_property_details
SET bedroom_count = ($1::float8)::numeric, bathroom_count = ($2::float8)::numeric, balcony_count = $3, is_duplex = $4, parking = $5
WHERE property_id = $6;
