SELECT a.name
FROM project_amenities pa
JOIN amenities a ON a.id = pa.amenity_id
WHERE pa.project_id = $1;
