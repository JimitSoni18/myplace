INSERT INTO properties (
	project_id, property_type_id, unit_number, building, floor_number,
	total_floors, built_up_area, usable_area, description, slug
)
VALUES ($1, $2, $3, $4, $5, $6, ($7::float8)::numeric, ($8::float8)::numeric, $9, $10)
RETURNING id;
