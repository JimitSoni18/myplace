SELECT COUNT(pr.id)
FROM properties pr
JOIN property_types pt ON pt.id = pr.property_type_id
WHERE pr.deleted_at IS NULL AND pt.category = 'COMMERCIAL';
