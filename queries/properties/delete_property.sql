UPDATE properties SET deleted_at = NOW() WHERE id = $1 AND project_id = $2;
