UPDATE land_property_details
SET parcel_number = $1, zoning = $2, approval_status = $3, development_status = $4
WHERE property_id = $5;
