SELECT slug, updated_at FROM project_owners WHERE active = TRUE AND deleted_at IS NULL ORDER BY id DESC;
