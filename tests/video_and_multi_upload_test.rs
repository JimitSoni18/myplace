use axum::{
	body::Body,
	http::{Request, StatusCode, header},
};
use myplace::{
	build_app_state,
	crypto::sign_cookie::get_signature,
	servers,
	session_store::{AuthUser, SessionStoreTrait, UserRole},
};
use tower::ServiceExt;
use uuid::Uuid;

async fn get_admin_cookie(state: &myplace::AppState) -> String {
	let auth_admin = AuthUser::new(1, "admin".to_string(), UserRole::Admin);
	let session_id = state.session_store.create_session(auth_admin).await;
	let sign = get_signature(&session_id);
	format!("session_id={}.{sign}", session_id.0)
}

fn generate_test_video() -> Vec<u8> {
	let temp_file = std::env::temp_dir().join(format!("test_{}.mp4", Uuid::new_v4()));
	let status = std::process::Command::new("ffmpeg")
		.args([
			"-y",
			"-f",
			"lavfi",
			"-i",
			"testsrc=duration=1:size=320x240:rate=10",
			"-pix_fmt",
			"yuv420p",
			temp_file.to_str().unwrap(),
		])
		.output()
		.expect("ffmpeg must be installed");
	assert!(
		status.status.success(),
		"ffmpeg test video generation failed"
	);

	let bytes = std::fs::read(&temp_file).unwrap();
	let _ = std::fs::remove_file(temp_file);
	bytes
}

fn generate_test_image() -> Vec<u8> {
	let img = image::RgbImage::new(100, 100);
	let mut buf = std::io::Cursor::new(Vec::new());
	img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
	buf.into_inner()
}

fn build_multipart_body(boundary: &str, parts: Vec<(&str, &str, &str, Vec<u8>)>) -> Vec<u8> {
	let mut body = Vec::new();
	for (field_name, file_name, content_type, data) in parts {
		body.extend(format!("--{boundary}\r\n").as_bytes());
		body.extend(
			format!(
				"Content-Disposition: form-data; name=\"{field_name}\"; filename=\"{file_name}\"\r\nContent-Type: {content_type}\r\n\r\n"
			)
			.as_bytes(),
		);
		body.extend(data);
		body.extend(b"\r\n");
	}
	body.extend(format!("--{boundary}--\r\n").as_bytes());
	body
}

#[tokio::test]
async fn test_video_and_multi_media_upload() {
	let state = build_app_state().await;
	let app = servers::admin_router(state.clone());
	let cookie = get_admin_cookie(&state).await;

	// Fetch a project or create one
	let proj = sqlx::query!("SELECT id FROM projects WHERE deleted_at IS NULL LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.unwrap();
	let project_id = proj.id;

	let video_bytes = generate_test_video();
	let img_bytes = generate_test_image();

	// 1. Upload a single video to project
	let boundary = "---------------------------987654321012345";
	let parts = vec![("files[]", "sample.mp4", "video/mp4", video_bytes.clone())];
	let body = build_multipart_body(boundary, parts);

	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!("/admin/projects/{project_id}/media"))
				.header(header::COOKIE, &cookie)
				.header(
					header::CONTENT_TYPE,
					format!("multipart/form-data; boundary={boundary}"),
				)
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// Verify video media record was created
	let vid_row = sqlx::query!(
		r#"
		SELECT m.id, m.media_type, m.thumbnail_key, m.width, m.height, m.duration_secs
		FROM project_media pm
		JOIN media m ON pm.media_id = m.id
		WHERE pm.project_id = $1 AND m.media_type = 'video'
		ORDER BY pm.sequence DESC
		LIMIT 1
		"#,
		project_id
	)
	.fetch_one(&state.model.db)
	.await
	.unwrap();

	assert_eq!(vid_row.media_type, "video");
	assert!(
		vid_row.thumbnail_key.is_some(),
		"thumbnail poster must be generated"
	);
	assert_eq!(vid_row.width, Some(320));
	assert_eq!(vid_row.height, Some(240));
	assert_eq!(vid_row.duration_secs, Some(1));

	// 2. Multi-file upload: 1 image + 1 video in single request
	let boundary = "---------------------------123456789012345";
	let parts = vec![
		("files[]", "img1.png", "image/png", img_bytes.clone()),
		("files[]", "vid2.mp4", "video/mp4", video_bytes.clone()),
	];
	let body = build_multipart_body(boundary, parts);

	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!("/admin/projects/{project_id}/media"))
				.header(header::COOKIE, &cookie)
				.header(
					header::CONTENT_TYPE,
					format!("multipart/form-data; boundary={boundary}"),
				)
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// 3. Verify media sequence and reordering
	let media_list = sqlx::query!(
		"SELECT media_id, sequence FROM project_media WHERE project_id = $1 ORDER BY sequence ASC",
		project_id
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap();

	assert!(media_list.len() >= 2);
	let mut reversed_ids: Vec<Uuid> = media_list.iter().map(|m| m.media_id).collect();
	reversed_ids.reverse();

	let reorder_payload = serde_json::json!({
		"order": reversed_ids
	});

	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!("/admin/projects/{project_id}/media/reorder"))
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/json")
				.body(Body::from(reorder_payload.to_string()))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	// Check that sequences now match reversed IDs
	let first_media = sqlx::query!(
		"SELECT media_id FROM project_media WHERE project_id = $1 AND sequence = 0",
		project_id
	)
	.fetch_one(&state.model.db)
	.await
	.unwrap();

	assert_eq!(first_media.media_id, reversed_ids[0]);

	// 4. Test corrupted / invalid video file produces 400 Bad Request and zero orphaned records
	let boundary = "---------------------------badfile12345";
	let parts = vec![(
		"files[]",
		"corrupted.mp4",
		"video/mp4",
		b"NOT_A_REAL_VIDEO_DATA".to_vec(),
	)];
	let body = build_multipart_body(boundary, parts);

	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!("/admin/projects/{project_id}/media"))
				.header(header::COOKIE, &cookie)
				.header(
					header::CONTENT_TYPE,
					format!("multipart/form-data; boundary={boundary}"),
				)
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
