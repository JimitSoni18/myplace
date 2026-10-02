SELECT slug FROM properties WHERE id = $1 AND deleted_at IS NULL;
