INSERT INTO project_amenities (project_id, amenity_id) VALUES ($1, $2) ON CONFLICT DO NOTHING;
