SELECT
    a.id, a.name, a.slug, a.is_active,
    COUNT(pa.project_id) as "projects_count!: i64"
FROM amenities a
    LEFT JOIN project_amenities pa ON pa.amenity_id = a.id
GROUP BY a.id
ORDER BY a.name ASC;
