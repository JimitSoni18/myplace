INSERT INTO amenities (name) VALUES ($1) ON CONFLICT (name) DO NOTHING;
