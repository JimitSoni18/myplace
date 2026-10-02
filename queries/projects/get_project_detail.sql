SELECT
	p.id, p.project_owner_id, po.name as owner_name,
	p.location_id, loc.formatted_address as "location_name?",
	p.name, p.slug, p.description, p.category,
	p.start_date, p.launch_date, p.possession_date, p.created_at
FROM projects p
	JOIN project_owners po ON po.id = p.project_owner_id
	LEFT JOIN locations loc ON loc.id = p.location_id
WHERE p.id = $1 AND p.deleted_at IS NULL;
