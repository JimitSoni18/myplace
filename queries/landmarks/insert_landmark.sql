INSERT INTO project_landmarks (
    project_id, category, name, distance, duration, latitude, longitude, source, sequence
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);
