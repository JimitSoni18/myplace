SELECT
	p.id, p.property_type_id, pt.category,
	p.unit_number, p.building, p.floor_number, p.total_floors,
	p.built_up_area::float8 as "built_up_area?",
	p.usable_area::float8 as "usable_area?",
	p.description,
	pl.listing_type as "listing_type?",
	pl.status as "status?",
	pl.price::float8 as "price?",
	pl.billing_period,
	rpd.bedroom_count::float8 as "bedroom_count?",
	rpd.bathroom_count::float8 as "bathroom_count?",
	rpd.balcony_count as "balcony_count?",
	rpd.is_duplex as "is_duplex?",
	rpd.parking as "parking_res?",
	cpd.parking as "parking_com?",
	lpd.parcel_number as "parcel_number?",
	lpd.zoning as "zoning?",
	lpd.approval_status as "approval_status?",
	lpd.development_status as "development_status?"
FROM properties p
	JOIN property_types pt ON pt.id = p.property_type_id
	LEFT JOIN property_listings pl ON pl.property_id = p.id
	LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
	LEFT JOIN commercial_property_details cpd ON cpd.property_id = p.id
	LEFT JOIN land_property_details lpd ON lpd.property_id = p.id
WHERE p.id = $1 AND p.project_id = $2 AND p.deleted_at IS NULL;
