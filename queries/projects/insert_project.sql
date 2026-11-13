INSERT INTO projects (
	project_owner_id, project_type_id, name, slug, description,
	formatted_address, city, area, state_or_province, postal_code,
	latitude, longitude, rera_number, rera_authority,
	total_land_area, towers_count, floors_count, units_count,
	start_date, launch_date, possession_date
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
RETURNING id;
