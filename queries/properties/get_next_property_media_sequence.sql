SELECT COALESCE(MAX(sequence), 0) + 1 as "seq!: i16" FROM property_media WHERE property_id = $1;
