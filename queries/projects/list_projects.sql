SELECT
	p.id, p.name, p.slug,
	pt.name as project_type_name,
	po.name as owner_name,
	p.city,
	p.area,
	p.formatted_address,
	p.rera_number,
	(
		SELECT m.thumbnail_key
		FROM project_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.project_id = p.id
		ORDER BY pm.sequence ASC
		LIMIT 1
	) as hero_thumb_key,
	COUNT(DISTINCT pc.id) as "configurations_count!: i64",
	COUNT(DISTINCT pm.media_id) as "media_count!: i64"
FROM projects p
	JOIN project_owners po ON po.id = p.project_owner_id
	JOIN project_types pt ON pt.id = p.project_type_id
	LEFT JOIN project_configurations pc ON pc.project_id = p.id AND pc.is_active = TRUE
	LEFT JOIN project_media pm ON pm.project_id = p.id
WHERE p.deleted_at IS NULL
	AND ($1::int IS NULL OR p.project_owner_id = $1)
	AND ($2::int IS NULL OR p.project_type_id = $2)
	AND ($3::text IS NULL OR p.name ILIKE $3 OR p.city ILIKE $3 OR p.area ILIKE $3)
GROUP BY p.id, pt.name, po.name
ORDER BY p.id DESC;
