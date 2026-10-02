SELECT
	p.id, p.unit_number, p.building,
	pt.name as type_name,
	p.built_up_area::float8 as "built_up_area?",
	pl.listing_type as "listing_type?",
	pl.status as "status?",
	pl.price::float8 as "price?",
	COUNT(DISTINCT pm.media_id) as "media_count!: i64",
	rpd.bedroom_count::float8 as "bedroom_count?",
	rpd.bathroom_count::float8 as "bathroom_count?",
	lpd.parcel_number as "parcel_number?",
	lpd.zoning as "zoning?"
FROM properties p
	JOIN property_types pt ON pt.id = p.property_type_id
	LEFT JOIN property_listings pl ON pl.property_id = p.id
	LEFT JOIN property_media pm ON pm.property_id = p.id
	LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
	LEFT JOIN land_property_details lpd ON lpd.property_id = p.id
WHERE p.project_id = $1 AND pt.category = $2 AND p.deleted_at IS NULL
GROUP BY p.id, pt.name, pl.listing_type, pl.status, pl.price,
         rpd.bedroom_count, rpd.bathroom_count, lpd.parcel_number, lpd.zoning
ORDER BY p.id DESC;
