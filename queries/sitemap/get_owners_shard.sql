SELECT id, slug, updated_at
FROM project_owners
WHERE active = TRUE AND deleted_at IS NULL AND id >= $1 AND id <= $2
ORDER BY id ASC;
