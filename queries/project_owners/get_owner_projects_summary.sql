SELECT
    p.id, p.name, p.slug,
    pt.name as project_type_name,
    p.city, p.area, p.formatted_address,
    COUNT(DISTINCT pc.id) as "configurations_count!: i64"
FROM projects p
    JOIN project_types pt ON pt.id = p.project_type_id
    LEFT JOIN project_configurations pc ON pc.project_id = p.id AND pc.is_active = TRUE
WHERE p.project_owner_id = $1 AND p.deleted_at IS NULL
GROUP BY p.id, pt.name
ORDER BY p.id DESC
LIMIT 10;
