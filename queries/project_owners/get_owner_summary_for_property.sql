SELECT
	p.id, p.name, p.slug, p.category,
	loc.formatted_address as "location?",
	po.id as owner_id, po.name as owner_name, po.slug as owner_slug, po.profile_img_thumb_key as owner_image_key
FROM projects p
	JOIN project_owners po ON po.id = p.project_owner_id
	LEFT JOIN locations loc ON loc.id = p.location_id
WHERE p.id = $1;
