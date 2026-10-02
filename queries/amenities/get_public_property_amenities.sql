SELECT a.name
FROM property_amenities pa
JOIN amenities a ON a.id = pa.amenity_id
WHERE pa.property_id = $1 AND a.is_active = TRUE
ORDER BY a.name ASC;
