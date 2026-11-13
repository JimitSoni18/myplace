INSERT INTO project_floor_plans (project_id, media_id, title, sequence)
VALUES ($1, $2, $3, $4)
RETURNING id;
