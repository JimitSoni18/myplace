SELECT
	p.id, p.name, p.slug, p.description,
	pt.name as project_type_name,
	p.formatted_address, p.city, p.area, p.state_or_province, p.postal_code,
	p.latitude::FLOAT8 AS "latitude?", p.longitude::FLOAT8 AS "longitude?",
	p.rera_number, p.rera_authority,
	p.total_land_area::FLOAT8 AS "total_land_area?", p.towers_count, p.floors_count, p.units_count,
	p.start_date, p.launch_date, p.possession_date
FROM projects p
	JOIN project_types pt ON pt.id = p.project_type_id
WHERE p.id = $1 AND p.project_owner_id = $2 AND p.deleted_at IS NULL;
