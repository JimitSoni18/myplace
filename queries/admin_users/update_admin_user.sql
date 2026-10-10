UPDATE profiles
SET username = $1, password = $2
WHERE id = $3;
