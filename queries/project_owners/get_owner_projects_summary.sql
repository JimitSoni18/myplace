SELECT
    p.id, p.name, p.slug, p.category,
    loc.formatted_address as "location?",
    COUNT(prop.id) as "properties_count!: i64"
FROM projects p
    LEFT JOIN locations loc ON loc.id = p.location_id
    LEFT JOIN properties prop ON prop.project_id = p.id AND prop.deleted_at IS NULL
WHERE p.project_owner_id = $1 AND p.deleted_at IS NULL
GROUP BY p.id, loc.formatted_address
ORDER BY p.id DESC
LIMIT 10;
