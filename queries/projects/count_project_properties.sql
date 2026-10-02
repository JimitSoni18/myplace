SELECT COUNT(id) as "count!: i64" FROM properties WHERE project_id = $1 AND deleted_at IS NULL;
