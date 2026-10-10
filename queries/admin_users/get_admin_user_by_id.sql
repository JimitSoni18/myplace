SELECT
	p.id,
	p.username,
	p.created_at
FROM
	profiles p
INNER JOIN admin_users a ON a.profile_id = p.id
WHERE
	p.id = $1;
