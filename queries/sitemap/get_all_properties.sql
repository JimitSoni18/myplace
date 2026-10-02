SELECT slug, updated_at FROM properties WHERE deleted_at IS NULL ORDER BY id DESC;
