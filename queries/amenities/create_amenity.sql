INSERT INTO amenities (name, slug) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING;
