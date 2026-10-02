use axum::{
	body::Body,
	http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use myplace::{build_app_state, cache::InvalidationEvent, config::CONFIG, servers};
use tower::ServiceExt;

#[tokio::test]
async fn test_sitemap_index_and_shards() {
	let state = build_app_state().await;
	let app = servers::public_router(state);

	// 1. GET /sitemap.xml returns sitemap index
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemap.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	assert_eq!(
		response.headers().get("content-type").unwrap(),
		"application/xml; charset=utf-8"
	);

	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();

	// Must be a sitemapindex
	assert!(body.contains("<sitemapindex"));
	assert!(body.contains("</sitemapindex>"));

	// Must use canonical public site origin
	assert!(body.contains(&format!("{}/sitemaps/pages.xml", CONFIG.public_site_origin)));
	assert!(!body.contains("localhost:3000"));

	// Should contain project/owner/property shards
	assert!(body.contains(&format!(
		"{}/sitemaps/projects-1-",
		CONFIG.public_site_origin
	)));

	// 2. GET /sitemaps/pages.xml returns urlset with static pages
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/pages.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();
	assert!(body.contains("<urlset"));
	assert!(body.contains(&format!("{}/", CONFIG.public_site_origin)));
	assert!(body.contains(&format!("{}/projects", CONFIG.public_site_origin)));
	assert!(body.contains(&format!("{}/properties", CONFIG.public_site_origin)));

	// 3. GET /sitemaps/projects-1-1000.xml returns urlset
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/projects-1-1000.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();
	assert!(body.contains("<urlset"));
	assert!(body.contains(&format!("{}/projects/", CONFIG.public_site_origin)));

	// 4. Empty shard returns 404
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/projects-990001-991000.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// 5. Malformed shard param returns 404
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/invalid-param.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_sitemap_shard_creation_update_and_removal() {
	let state = build_app_state().await;
	let app = servers::public_router(state.clone());

	// Pick a test ID in range 95001-96000
	let test_id = 95005;

	// Clean up any stale record
	let _ = sqlx::query!("DELETE FROM projects WHERE id = $1", test_id)
		.execute(&state.model.db)
		.await;

	// Shard should initially be 404
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/projects-95001-96000.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// Fetch an existing owner and location to link
	let owner = sqlx::query!("SELECT id FROM project_owners LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.unwrap();
	let location = sqlx::query!("SELECT id FROM locations LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.unwrap();

	// 1. CREATE: Insert test project with test_id
	sqlx::query!(
		"INSERT INTO projects (id, project_owner_id, location_id, name, slug, category) \
		 OVERRIDING SYSTEM VALUE VALUES ($1, $2, $3, 'Shard Test Project', 'shard-test-project', 'Residential')",
		test_id,
		owner.id,
		location.id
	)
	.execute(&state.model.db)
	.await
	.unwrap();

	// Trigger invalidation / shard regeneration
	state
		.page_cache
		.invalidate(InvalidationEvent::ProjectUpdated(test_id))
		.await;

	// Shard should now exist (200 OK)
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/projects-95001-96000.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();
	assert!(body.contains("shard-test-project"));

	// Index should now include shard 95001-96000
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemap.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();
	assert!(body.contains("projects-95001-96000.xml"));

	// 2. UPDATE: Change slug and verify shard updates
	sqlx::query!(
		"UPDATE projects SET slug = 'shard-test-project-updated', updated_at = NOW() WHERE id = $1",
		test_id
	)
	.execute(&state.model.db)
	.await
	.unwrap();

	state
		.page_cache
		.invalidate(InvalidationEvent::ProjectUpdated(test_id))
		.await;

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/projects-95001-96000.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();
	assert!(body.contains("shard-test-project-updated"));

	// 3. DELETE: Soft-delete test project and verify shard removal
	sqlx::query!(
		"UPDATE projects SET deleted_at = NOW() WHERE id = $1",
		test_id
	)
	.execute(&state.model.db)
	.await
	.unwrap();

	state
		.page_cache
		.invalidate(InvalidationEvent::ProjectUpdated(test_id))
		.await;

	// Shard should now be removed / return 404
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemaps/projects-95001-96000.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// Index should no longer list projects-95001-96000.xml
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/sitemap.xml")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();
	assert!(!body.contains("projects-95001-96000.xml"));

	// Cleanup
	let _ = sqlx::query!("DELETE FROM projects WHERE id = $1", test_id)
		.execute(&state.model.db)
		.await;
}
