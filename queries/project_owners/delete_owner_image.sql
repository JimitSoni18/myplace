UPDATE project_owners SET profile_img_key = NULL, profile_img_thumb_key = NULL, updated_at = NOW() WHERE id = $1;
