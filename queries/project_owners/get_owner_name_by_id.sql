SELECT id, name FROM project_owners WHERE id = $1 AND deleted_at IS NULL;
