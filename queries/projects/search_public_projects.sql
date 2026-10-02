SELECT
	p.id, p.name, p.slug, p.category,
	loc.formatted_address as "location?",
	COUNT(DISTINCT prop.id) as "properties_count!: i64",
	p.possession_date,
	po.id as owner_id,
	po.name as owner_name,
	po.slug as owner_slug,
	(
		SELECT m.thumbnail_key
		FROM project_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.project_id = p.id
		ORDER BY pm.sequence ASC
		LIMIT 1
	) as hero_thumb_key
FROM projects p
	JOIN project_owners po ON po.id = p.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
	LEFT JOIN locations loc ON loc.id = p.location_id
	LEFT JOIN properties prop ON prop.project_id = p.id AND prop.deleted_at IS NULL
WHERE p.deleted_at IS NULL
  AND ($1::text IS NULL OR p.category = $1)
  AND ($2::text IS NULL OR p.name ILIKE $2 OR loc.formatted_address ILIKE $2)
GROUP BY p.id, loc.formatted_address, po.id, po.name, po.slug
ORDER BY p.id DESC;
