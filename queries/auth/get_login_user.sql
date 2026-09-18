SELECT
	p.id,
	p.username,
	p.password,
	u.profile_id
FROM
	profiles p
INNER JOIN admin_users u ON u.profile_id = p.id
WHERE
	p.username = $1;
