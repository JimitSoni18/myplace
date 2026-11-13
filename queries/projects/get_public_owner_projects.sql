SELECT
	p.id, p.name, p.slug,
	pt.name as project_type_name,
	p.formatted_address, p.city, p.area,
	COUNT(DISTINCT pc.id) as "configurations_count!: i64",
	p.possession_date,
	(
		SELECT m.thumbnail_key
		FROM project_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.project_id = p.id
		ORDER BY pm.sequence ASC
		LIMIT 1
	) as hero_thumb_key,
	(
		SELECT MIN(pc2.price)::FLOAT8
		FROM project_configurations pc2
		WHERE pc2.project_id = p.id AND pc2.price > 0 AND pc2.is_active = TRUE
	) as "min_price?: f64"
FROM projects p
	JOIN project_types pt ON pt.id = p.project_type_id
	LEFT JOIN project_configurations pc ON pc.project_id = p.id AND pc.is_active = TRUE
WHERE p.project_owner_id = $1 AND p.deleted_at IS NULL
GROUP BY p.id, pt.name
ORDER BY p.id DESC;
