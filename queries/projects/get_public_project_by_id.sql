SELECT
	p.id, p.project_owner_id, p.name, p.slug, p.description, p.category,
	loc.formatted_address as "location_name?",
	p.start_date, p.launch_date, p.possession_date
FROM projects p
	LEFT JOIN locations loc ON loc.id = p.location_id
WHERE p.id = $1 AND p.deleted_at IS NULL;
