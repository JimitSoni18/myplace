SELECT id, name, is_active, created_at
FROM project_types
WHERE is_active = TRUE
ORDER BY id ASC;
