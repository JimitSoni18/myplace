SELECT
	p.id, p.name, p.slug, p.category,
	po.name as owner_name,
	loc.formatted_address as "location?",
	(
		SELECT m.thumbnail_key
		FROM project_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.project_id = p.id
		ORDER BY pm.sequence ASC
		LIMIT 1
	) as hero_thumb_key,
	COUNT(DISTINCT prop.id) as "properties_count!: i64",
	COUNT(DISTINCT pm.media_id) as "media_count!: i64"
FROM projects p
	JOIN project_owners po ON po.id = p.project_owner_id
	LEFT JOIN locations loc ON loc.id = p.location_id
	LEFT JOIN properties prop ON prop.project_id = p.id AND prop.deleted_at IS NULL
	LEFT JOIN project_media pm ON pm.project_id = p.id
WHERE p.deleted_at IS NULL
	AND ($1::int IS NULL OR p.project_owner_id = $1)
	AND ($2::text IS NULL OR p.category = $2)
	AND ($3::text IS NULL OR p.name ILIKE $3)
GROUP BY p.id, po.name, loc.formatted_address
ORDER BY p.id DESC;
