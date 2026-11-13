SELECT
    pfp.id,
    pfp.project_id,
    pfp.media_id,
    pfp.title,
    pfp.sequence,
    m.s3_key,
    m.thumbnail_key
FROM project_floor_plans pfp
JOIN media m ON m.id = pfp.media_id
WHERE pfp.project_id = $1
ORDER BY pfp.sequence ASC, pfp.created_at ASC;
