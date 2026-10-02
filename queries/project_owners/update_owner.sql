UPDATE project_owners
SET name = $1, bio = $2, email = $3, phone = $4, website = $5, active = $6, slug = $7, updated_at = NOW()
WHERE id = $8 AND deleted_at IS NULL;
