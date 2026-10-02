SELECT id, name FROM property_types WHERE category = $1 AND is_active = TRUE ORDER BY name ASC;
