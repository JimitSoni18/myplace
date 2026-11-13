SELECT
    id,
    project_id,
    category,
    name,
    distance,
    duration,
    latitude::FLOAT8 AS "latitude?",
    longitude::FLOAT8 AS "longitude?",
    source,
    sequence
FROM project_landmarks
WHERE project_id = $1
ORDER BY sequence ASC, id ASC;
