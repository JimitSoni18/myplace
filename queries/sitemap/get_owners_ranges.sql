SELECT
    ((id - 1) / $1) * $1 + 1 AS "range_start!: i32",
    MAX(updated_at) AS "lastmod!: time::OffsetDateTime"
FROM project_owners
WHERE active = TRUE AND deleted_at IS NULL
GROUP BY 1
ORDER BY 1 ASC;
