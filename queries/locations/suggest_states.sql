SELECT DISTINCT state_or_province
FROM projects
WHERE state_or_province != '' AND deleted_at IS NULL
ORDER BY state_or_province ASC;
