SELECT
    po.id, po.name, po.slug, pr.username, po.bio, po.email, po.phone, po.website,
    po.active, po.profile_img_key, po.profile_img_thumb_key, po.created_at,
    COUNT(p.id) as "projects_count!: i64"
FROM project_owners po
    LEFT JOIN profiles pr ON pr.id = po.profile_id
    LEFT JOIN projects p ON p.project_owner_id = po.id AND p.deleted_at IS NULL
WHERE po.id = $1 AND po.deleted_at IS NULL
GROUP BY po.id, pr.username;
