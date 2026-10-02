INSERT INTO land_property_details (
	property_id, parcel_number, zoning, approval_status, development_status
)
VALUES ($1, $2, $3, $4, $5);
