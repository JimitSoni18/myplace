SELECT
    proj.id,
    proj.name,
    po.name AS owner_name,
    proj.city,
    proj.area,
    pt.name AS project_type_name,
    COUNT(DISTINCT pc.id) AS "configurations_count: i64"
FROM projects proj
JOIN project_owners po ON po.id = proj.project_owner_id
JOIN project_types pt ON pt.id = proj.project_type_id
LEFT JOIN project_configurations pc ON pc.project_id = proj.id AND pc.is_active = TRUE
WHERE proj.deleted_at IS NULL
GROUP BY proj.id, po.name, pt.name
ORDER BY proj.created_at DESC
LIMIT 10;
