UPDATE property_media
SET sequence = $3
WHERE property_id = $1 AND media_id = $2;
