SELECT id, name, slug, phone, email, website, profile_img_key, bio
FROM project_owners
WHERE id = $1 AND active = TRUE AND deleted_at IS NULL;
