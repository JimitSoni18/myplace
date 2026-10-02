SELECT COUNT(p.id) as "c!: i64" FROM properties p JOIN property_types pt ON pt.id = p.property_type_id WHERE pt.category = 'LAND' AND p.deleted_at IS NULL;
