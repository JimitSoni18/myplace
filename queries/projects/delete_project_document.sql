DELETE FROM project_documents WHERE id = $1 AND project_id = $2 RETURNING media_id;
