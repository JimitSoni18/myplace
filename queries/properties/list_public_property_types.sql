SELECT id, name, slug, category
FROM property_types
WHERE is_active = TRUE
ORDER BY
  CASE category
    WHEN 'RESIDENTIAL' THEN 1
    WHEN 'COMMERCIAL' THEN 2
    WHEN 'LAND' THEN 3
    ELSE 4
  END,
  name ASC;
