UPDATE properties
SET property_type_id = $1, unit_number = $2, building = $3, floor_number = $4,
    total_floors = $5, built_up_area = ($6::float8)::numeric, usable_area = ($7::float8)::numeric, description = $8,
    updated_at = NOW()
WHERE id = $9 AND project_id = $10 AND deleted_at IS NULL;
