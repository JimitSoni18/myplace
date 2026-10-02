UPDATE project_media
SET sequence = -sequence - 1000
WHERE project_id = $1;
