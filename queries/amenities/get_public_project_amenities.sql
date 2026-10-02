SELECT a.name
FROM project_amenities pa
JOIN amenities a ON a.id = pa.amenity_id
WHERE pa.project_id = $1 AND a.is_active = TRUE
ORDER BY a.name ASC;
