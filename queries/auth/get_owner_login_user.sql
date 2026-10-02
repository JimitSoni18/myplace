SELECT pr.id, pr.username, pr.password, po.active
FROM profiles pr
JOIN project_owners po ON po.profile_id = pr.id
WHERE pr.username = $1 AND po.deleted_at IS NULL;
