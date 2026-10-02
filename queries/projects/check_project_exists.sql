SELECT id, name FROM projects WHERE id = $1 AND deleted_at IS NULL;
