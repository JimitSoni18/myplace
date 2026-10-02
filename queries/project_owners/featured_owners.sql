SELECT
	po.id, po.name, po.slug, po.profile_img_thumb_key,
	COUNT(p.id) as "project_count!: i64"
FROM project_owners po
	LEFT JOIN projects p ON p.project_owner_id = po.id AND p.deleted_at IS NULL
WHERE po.active = TRUE AND po.deleted_at IS NULL
GROUP BY po.id
ORDER BY COUNT(p.id) DESC, po.id DESC
LIMIT 4;
