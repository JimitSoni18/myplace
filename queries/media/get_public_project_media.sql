SELECT m.media_type, m.s3_key, m.thumbnail_key, pm.sequence
FROM project_media pm
JOIN media m ON m.id = pm.media_id
WHERE pm.project_id = $1
ORDER BY pm.sequence ASC;
