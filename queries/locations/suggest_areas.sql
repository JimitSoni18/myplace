SELECT DISTINCT area
FROM projects
WHERE area != ''
  AND deleted_at IS NULL
  AND ($1::TEXT IS NULL OR city = $1)
ORDER BY area ASC;
