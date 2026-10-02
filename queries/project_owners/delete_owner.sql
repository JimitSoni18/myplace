UPDATE project_owners SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL;
