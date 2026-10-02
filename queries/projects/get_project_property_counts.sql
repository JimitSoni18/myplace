SELECT pt.category, COUNT(p.id) as count
FROM properties p
JOIN property_types pt ON pt.id = p.property_type_id
WHERE p.project_id = $1 AND p.deleted_at IS NULL
GROUP BY pt.category;
