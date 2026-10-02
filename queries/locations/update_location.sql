UPDATE locations SET formatted_address = $1, city = $2, state_or_province = $3, area = $4, updated_at = NOW() WHERE id = $5;
