SELECT COALESCE(MAX(sequence), -1) + 1 as "next_seq!: i16"
FROM project_media
WHERE project_id = $1;
