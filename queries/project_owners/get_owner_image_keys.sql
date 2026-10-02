SELECT profile_img_key, profile_img_thumb_key FROM project_owners WHERE id = $1 AND deleted_at IS NULL;
