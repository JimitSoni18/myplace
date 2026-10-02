UPDATE project_owners
SET profile_img_key = $1, profile_img_thumb_key = $2, updated_at = NOW()
WHERE id = $3;
