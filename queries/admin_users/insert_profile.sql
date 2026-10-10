INSERT INTO profiles (username, password) VALUES ($1, $2) RETURNING id;
