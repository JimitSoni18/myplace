SELECT pd.id, pd.media_id, pd.display_name, pd.doc_type, m.s3_key, m.file_size
FROM project_documents pd
JOIN media m ON m.id = pd.media_id
WHERE pd.project_id = $1
ORDER BY pd.id DESC;
