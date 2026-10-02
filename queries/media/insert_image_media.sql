INSERT INTO media (
	id, media_type, mime_type, s3_key, thumbnail_key,
	file_size, width, height
)
VALUES ($1, 'image', 'image/webp', $2, $3, $4, $5, $6);
