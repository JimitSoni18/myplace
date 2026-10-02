SELECT
	id, project_owner_id, location_id, name, description,
	category, start_date, launch_date, possession_date
FROM projects
WHERE id = $1 AND deleted_at IS NULL;
