UPDATE projects SET deleted_at = NOW() WHERE id = $1;
