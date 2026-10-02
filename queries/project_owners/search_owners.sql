SELECT
    po.id, po.name, po.active, po.profile_img_thumb_key,
    po.email,
    COUNT(p.id) as "projects_count!: i64"
FROM project_owners po
    LEFT JOIN projects p ON p.project_owner_id = po.id AND p.deleted_at IS NULL
WHERE po.name ILIKE $1 AND po.deleted_at IS NULL
GROUP BY po.id
ORDER BY po.id DESC;
