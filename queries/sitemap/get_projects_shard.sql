SELECT id, slug, updated_at
FROM projects
WHERE deleted_at IS NULL AND id >= $1 AND id <= $2
ORDER BY id ASC;
