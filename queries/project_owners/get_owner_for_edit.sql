SELECT
    po.id, po.name, po.bio, pr.username, po.email, po.phone, po.website, po.active,
    po.profile_img_key, po.profile_img_thumb_key
FROM project_owners po
    LEFT JOIN profiles pr ON pr.id = po.profile_id
WHERE po.id = $1 AND po.deleted_at IS NULL;
