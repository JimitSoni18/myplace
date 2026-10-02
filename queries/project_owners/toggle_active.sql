UPDATE project_owners
SET active = NOT active, updated_at = NOW()
WHERE id = $1 AND deleted_at IS NULL;
