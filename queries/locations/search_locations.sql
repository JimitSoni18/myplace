SELECT id, formatted_address, city, state_or_province, area
FROM locations
WHERE formatted_address ILIKE $1 OR city ILIKE $1
ORDER BY id;
