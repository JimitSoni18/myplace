SELECT
	p.id, p.slug,
	pt.category, pt.name as property_type_name,
	p.built_up_area::float8 as "built_up_area?",
	rpd.bedroom_count::float8 as "bedroom_count?",
	rpd.bathroom_count::float8 as "bathroom_count?",
	pl.price::float8 as "price?",
	pl.currency_code as "currency_code?",
	pl.listing_type as "listing_type?",
	pl.billing_period as "billing_period?",
	proj.id as project_id,
	proj.name as project_name,
	proj.slug as project_slug,
	loc.formatted_address as "location?",
	(
		SELECT m.thumbnail_key
		FROM property_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.property_id = p.id
		ORDER BY pm.sequence ASC
		LIMIT 1
	) as hero_thumb_key
FROM properties p
	JOIN property_types pt ON pt.id = p.property_type_id
	JOIN projects proj ON proj.id = p.project_id AND proj.deleted_at IS NULL
	JOIN project_owners po ON po.id = proj.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
	LEFT JOIN locations loc ON loc.id = proj.location_id
	LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
	LEFT JOIN property_listings pl ON pl.property_id = p.id AND pl.status = 'active'
WHERE p.deleted_at IS NULL
  AND ($1::text IS NULL OR pt.category = $1)
  AND ($2::text IS NULL OR pl.listing_type = $2)
  AND ($3::text IS NULL OR proj.name ILIKE $3 OR loc.formatted_address ILIKE $3 OR p.unit_number ILIKE $3)
ORDER BY p.id DESC;
