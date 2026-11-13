SELECT DISTINCT city
FROM projects
WHERE city != '' AND deleted_at IS NULL
ORDER BY city ASC;
