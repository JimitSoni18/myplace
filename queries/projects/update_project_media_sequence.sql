UPDATE project_media
SET sequence = $3
WHERE project_id = $1 AND media_id = $2;
