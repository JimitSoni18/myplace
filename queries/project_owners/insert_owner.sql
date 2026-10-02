INSERT INTO project_owners (profile_id, name, slug, bio, email, phone, website, active)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
RETURNING id;
