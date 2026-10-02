UPDATE projects
SET project_owner_id = $1, location_id = $2, name = $3, slug = $4,
	description = $5, category = $6, start_date = $7, launch_date = $8,
	possession_date = $9, updated_at = NOW()
WHERE id = $10 AND deleted_at IS NULL;
