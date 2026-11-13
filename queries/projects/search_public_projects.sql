SELECT
	p.id, p.name, p.slug,
	pt.name as project_type_name,
	p.formatted_address, p.city, p.area,
	COUNT(DISTINCT pc.id) as "configurations_count!: i64",
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
	) as hero_thumb_key,
	(
		SELECT MIN(pc2.price)
		FROM project_configurations pc2
		WHERE pc2.project_id = p.id AND pc2.price > 0 AND pc2.is_active = TRUE
	) as min_price,
	(
		SELECT string_agg(DISTINCT pc3.bhk::text || ' BHK', ', ' ORDER BY pc3.bhk::text || ' BHK')
		FROM project_configurations pc3
		WHERE pc3.project_id = p.id AND pc3.bhk IS NOT NULL AND pc3.is_active = TRUE
	) as bhk_summary
FROM projects p
	JOIN project_owners po ON po.id = p.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
	JOIN project_types pt ON pt.id = p.project_type_id
	LEFT JOIN project_configurations pc ON pc.project_id = p.id AND pc.is_active = TRUE
WHERE p.deleted_at IS NULL
  AND ($1::int IS NULL OR p.project_type_id = $1)
  AND ($2::text IS NULL OR p.city ILIKE $2)
  AND ($3::text IS NULL OR p.area ILIKE $3)
  AND ($4::text IS NULL OR p.name ILIKE $4 OR p.description ILIKE $4 OR p.formatted_address ILIKE $4)
  AND ($5::FLOAT8 IS NULL OR EXISTS (
      SELECT 1 FROM project_configurations pc4
      WHERE pc4.project_id = p.id AND pc4.bhk = $5 AND pc4.is_active = TRUE
  ))
GROUP BY p.id, pt.name, po.id, po.name, po.slug
ORDER BY p.id DESC;
