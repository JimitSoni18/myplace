SELECT id, unit_number, building FROM properties WHERE id = $1 AND project_id = $2 AND deleted_at IS NULL;
