SELECT
    proj.id,
    proj.name,
    po.name   AS owner_name,
    loc.formatted_address AS location,
    proj.category,
    COUNT(DISTINCT pr.id)                                                   AS "total_properties: i64",
    COUNT(DISTINCT pl.id) FILTER (WHERE pl.status = 'active')               AS "available_properties: i64"
FROM projects proj
JOIN project_owners po ON po.id = proj.project_owner_id
JOIN locations     loc ON loc.id = proj.location_id
LEFT JOIN properties pr ON pr.project_id = proj.id AND pr.deleted_at IS NULL
LEFT JOIN property_listings pl ON pl.property_id = pr.id
WHERE proj.deleted_at IS NULL
  AND proj.created_at >= NOW() - INTERVAL '30 days'
GROUP BY proj.id, po.name, loc.formatted_address
ORDER BY proj.created_at DESC
LIMIT 20;
