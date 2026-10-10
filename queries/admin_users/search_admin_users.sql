SELECT
	p.id,
	p.username,
	p.created_at
FROM
	profiles p
INNER JOIN admin_users a ON a.profile_id = p.id
WHERE
	p.username ILIKE $1
ORDER BY p.id ASC;
