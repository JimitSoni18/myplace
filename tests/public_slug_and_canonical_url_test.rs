use axum::{
	body::Body,
	http::{Request, StatusCode, header},
};
use myplace::{build_app_state, cache::InvalidationEvent, servers};
use tower::ServiceExt;

#[tokio::test]
async fn test_public_canonical_slugs_and_redirects() {
	let state = build_app_state().await;
	let app = servers::public_router(state.clone());

	// -----------------------------------------------------------------------
	// 1. Projects canonical URLs and redirects
	// -----------------------------------------------------------------------
	let proj = sqlx::query!("SELECT id, slug, name FROM projects WHERE deleted_at IS NULL LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.expect("At least one project in DB");
	let project_id = proj.id;
	let project_slug = proj.slug;

	// 1a. Canonical URL returns 200 OK
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/projects/{project_id}-{project_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);

	// 1b. Wrong slug with valid ID returns 301 to canonical URL
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/projects/{project_id}-completely-wrong-slug"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
	let loc = res
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert_eq!(loc, format!("/projects/{project_id}-{project_slug}"));

	// 1c. Legacy slug-only returns 301 to canonical URL
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/projects/{project_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
	let loc = res
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert_eq!(loc, format!("/projects/{project_id}-{project_slug}"));

	// 1d. Nonexistent numeric ID returns 404
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/projects/999999-nonexistent-project")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::NOT_FOUND);

	// 1e. Nonexistent slug-only returns 404
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/projects/completely-nonexistent-project-xyz-987654")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::NOT_FOUND);

	// -----------------------------------------------------------------------
	// 2. Project Owners canonical URLs and redirects
	// -----------------------------------------------------------------------
	let owner =
		sqlx::query!("SELECT id, slug, name FROM project_owners WHERE deleted_at IS NULL LIMIT 1")
			.fetch_one(&state.model.db)
			.await
			.expect("At least one project owner in DB");
	let owner_id = owner.id;
	let owner_slug = owner.slug;

	// 2a. Canonical URL returns 200 OK
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/owners/{owner_id}-{owner_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);

	// 2b. Wrong slug with valid ID returns 301 to canonical URL
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/owners/{owner_id}-stale-or-wrong-name"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
	let loc = res
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert_eq!(loc, format!("/owners/{owner_id}-{owner_slug}"));

	// 2c. Legacy slug-only returns 301 to canonical URL
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/owners/{owner_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
	let loc = res
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert_eq!(loc, format!("/owners/{owner_id}-{owner_slug}"));

	// 2d. Nonexistent ID returns 404
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/owners/999999-nobody")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::NOT_FOUND);

	// -----------------------------------------------------------------------
	// 3. Properties canonical URLs and redirects
	// -----------------------------------------------------------------------
	let prop = sqlx::query!("SELECT id, slug FROM properties WHERE deleted_at IS NULL LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.expect("At least one property in DB");
	let prop_id = prop.id;
	let prop_slug = prop.slug;

	// 3a. Canonical URL returns 200 OK
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/properties/{prop_id}-{prop_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);

	// 3b. Wrong slug returns 301 to canonical URL
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/properties/{prop_id}-outdated-title"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
	let loc = res
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert_eq!(loc, format!("/properties/{prop_id}-{prop_slug}"));

	// 3c. Legacy slug-only returns 301 to canonical URL
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/properties/{prop_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
	let loc = res
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert_eq!(loc, format!("/properties/{prop_id}-{prop_slug}"));

	// 3d. Nonexistent property returns 404
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties/999999-not-real")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::NOT_FOUND);

	// -----------------------------------------------------------------------
	// 4. Changing resource name immediately updates canonical slug and redirects old slug
	// -----------------------------------------------------------------------
	// Create a new dedicated project for rename testing
	let new_proj = sqlx::query!(
		r#"
		INSERT INTO projects (project_owner_id, location_id, name, slug, category)
		VALUES ($1, (SELECT id FROM locations LIMIT 1), 'Initial Test Project Name', 'initial-test-project-name', 'Residential')
		RETURNING id, slug
		"#,
		owner_id
	)
	.fetch_one(&state.model.db)
	.await
	.unwrap();

	let test_proj_id = new_proj.id;
	let old_slug = new_proj.slug;

	// Invalidate cache
	state
		.page_cache
		.invalidate(InvalidationEvent::ProjectUpdated(test_proj_id))
		.await;

	// Verify initial slug works
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/projects/{test_proj_id}-{old_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);

	// Now rename project (change slug)
	let new_slug = "updated-test-project-name-2026";
	sqlx::query!(
		"UPDATE projects SET name = 'Updated Test Project Name 2026', slug = $1 WHERE id = $2",
		new_slug,
		test_proj_id
	)
	.execute(&state.model.db)
	.await
	.unwrap();

	// Invalidate cache
	state
		.page_cache
		.invalidate(InvalidationEvent::ProjectUpdated(test_proj_id))
		.await;

	// Accessing old slug now permanently redirects (301) to new slug!
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/projects/{test_proj_id}-{old_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);
	let loc = res
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert_eq!(loc, format!("/projects/{test_proj_id}-{new_slug}"));

	// Accessing new slug returns 200 OK!
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/projects/{test_proj_id}-{new_slug}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);

	// Clean up test project
	sqlx::query!("DELETE FROM projects WHERE id = $1", test_proj_id)
		.execute(&state.model.db)
		.await
		.unwrap();
}
