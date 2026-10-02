SELECT
	p.id, p.project_id, pr.name as project_name,
	p.unit_number, p.building,
	pt.category, pt.name as type_name,
	pl.price::float8 as "price?",
	pl.status as "status?"
FROM properties p
	JOIN projects pr ON pr.id = p.project_id
	JOIN property_types pt ON pt.id = p.property_type_id
	LEFT JOIN property_listings pl ON pl.property_id = p.id
WHERE p.deleted_at IS NULL
	AND ($1::int IS NULL OR p.project_id = $1)
	AND ($2::text IS NULL OR pt.category = $2)
	AND ($3::text IS NULL OR (p.unit_number ILIKE $3 OR p.building ILIKE $3))
ORDER BY p.id DESC
LIMIT 50;
