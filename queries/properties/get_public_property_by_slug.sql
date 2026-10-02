SELECT
	p.id, p.project_id, p.slug, p.unit_number, p.building,
	p.floor_number, p.total_floors,
	p.built_up_area::float8 as "built_up_area?",
	p.usable_area::float8 as "usable_area?",
	p.description,
	pt.category, pt.name as property_type_name,
	pl.price::float8 as "price?",
	pl.currency_code as "currency_code?",
	pl.listing_type as "listing_type?",
	pl.billing_period as "billing_period?",
	rpd.bedroom_count::float8 as "bedroom_count?",
	rpd.bathroom_count::float8 as "bathroom_count?",
	rpd.balcony_count as "balcony_count?",
	rpd.is_duplex as "is_duplex?",
	rpd.parking as "res_parking?",
	cpd.parking as "com_parking?",
	lpd.parcel_number as "parcel_number?",
	lpd.zoning as "zoning?",
	lpd.approval_status as "approval_status?",
	lpd.development_status as "development_status?"
FROM properties p
	JOIN property_types pt ON pt.id = p.property_type_id
	JOIN projects proj ON proj.id = p.project_id AND proj.deleted_at IS NULL
	JOIN project_owners po ON po.id = proj.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
	LEFT JOIN property_listings pl ON pl.property_id = p.id AND pl.status = 'active'
	LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
	LEFT JOIN commercial_property_details cpd ON cpd.property_id = p.id
	LEFT JOIN land_property_details lpd ON lpd.property_id = p.id
WHERE LOWER(p.slug) = LOWER($1) AND p.deleted_at IS NULL;
