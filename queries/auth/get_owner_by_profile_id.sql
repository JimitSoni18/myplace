SELECT id, name, active
FROM project_owners
WHERE profile_id = $1 AND deleted_at IS NULL;
