INSERT INTO media (
	id, media_type, mime_type, s3_key, thumbnail_key,
	file_size, width, height, duration_secs
)
VALUES ($1, 'video', $2, $3, $4, $5, $6, $7, $8);
