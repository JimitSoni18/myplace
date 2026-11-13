SELECT
    pc.id,
    pc.project_id,
    pc.name,
    pc.bhk::FLOAT8 AS "bhk?",
    pc.carpet_area::FLOAT8 AS "carpet_area?",
    pc.built_up_area::FLOAT8 AS "built_up_area?",
    pc.super_built_up_area::FLOAT8 AS "super_built_up_area?",
    pc.price::FLOAT8 AS "price?",
    pc.price_on_request,
    pc.floor_plan_media_id,
    m.s3_key AS "floor_plan_key?",
    m.thumbnail_key AS "floor_plan_thumb_key?",
    pc.sequence,
    pc.is_active,
    pc.created_at
FROM project_configurations pc
LEFT JOIN media m ON m.id = pc.floor_plan_media_id
WHERE pc.project_id = $1
ORDER BY pc.sequence ASC, pc.id ASC;
