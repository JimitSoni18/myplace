SELECT
	id, project_owner_id, project_type_id, name, slug, description,
	formatted_address, city, area, state_or_province, postal_code,
	latitude::FLOAT8 AS "latitude?", longitude::FLOAT8 AS "longitude?",
	rera_number, rera_authority,
	total_land_area::FLOAT8 AS "total_land_area?", towers_count, floors_count, units_count,
	start_date, launch_date, possession_date
FROM projects
WHERE id = $1 AND deleted_at IS NULL;
