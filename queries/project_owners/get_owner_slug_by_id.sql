SELECT slug FROM project_owners WHERE id = $1 AND active = TRUE AND deleted_at IS NULL;
