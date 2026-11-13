SELECT id, project_id, category, items, sequence
FROM project_specifications
WHERE project_id = $1
ORDER BY sequence ASC, id ASC;
