INSERT INTO project_configurations (
    project_id, name, bhk, carpet_area, built_up_area, super_built_up_area,
    price, price_on_request, floor_plan_media_id, sequence
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
RETURNING id;
