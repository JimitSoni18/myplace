UPDATE property_media
SET sequence = -sequence - 1000
WHERE property_id = $1;
