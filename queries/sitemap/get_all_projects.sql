SELECT slug, updated_at FROM projects WHERE deleted_at IS NULL ORDER BY id DESC;
