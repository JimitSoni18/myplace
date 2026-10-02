INSERT INTO projects (
	project_owner_id, location_id, name, slug, description,
	category, start_date, launch_date, possession_date
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
RETURNING id;
