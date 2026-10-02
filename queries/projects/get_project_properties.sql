SELECT
	p.id, p.slug, p.unit_number, p.building,
	p.built_up_area,
	pt.name as property_type_name,
	pt.category,
	pl.price,
	pl.status as "status?"
FROM properties p
JOIN property_types pt ON pt.id = p.property_type_id
LEFT JOIN property_listings pl ON pl.property_id = p.id
WHERE p.project_id = $1 AND p.deleted_at IS NULL
ORDER BY p.id DESC;
