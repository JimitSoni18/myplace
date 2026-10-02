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

async fn get_admin_cookie(state: &myplace::AppState) -> String {
	let auth_admin = AuthUser::new(1, "admin".to_string(), UserRole::Admin);
	let session_id = state.session_store.create_session(auth_admin).await;
	let sign = get_signature(&session_id);
	format!("session_id={}.{sign}", session_id.0)
}

#[tokio::test]
async fn test_project_creation_form_amenities_and_validation() {
	let state = build_app_state().await;
	let app = servers::admin_router(state.clone());
	let cookie = get_admin_cookie(&state).await;

	// Fetch existing owner and location IDs from db
	let owner = sqlx::query!("SELECT id FROM project_owners WHERE deleted_at IS NULL LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.unwrap();
	let location = sqlx::query!("SELECT id FROM locations LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.unwrap();
	let amenities = sqlx::query!("SELECT id FROM amenities LIMIT 2")
		.fetch_all(&state.model.db)
		.await
		.unwrap();
	assert!(amenities.len() >= 2, "need at least 2 amenities for test");
	let a1 = amenities[0].id;
	let a2 = amenities[1].id;

	// 1. NO AMENITIES: should succeed and redirect (303)
	let body = format!(
		"name=Test+Proj+No+Amenities&category=Residential&project_owner_id={}&location_id={}",
		owner.id, location.id
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::SEE_OTHER);
	let loc = response
		.headers()
		.get(header::LOCATION)
		.unwrap()
		.to_str()
		.unwrap();
	assert!(loc.starts_with("/admin/projects/"));

	// 2. ONE AMENITY: should succeed and redirect (303) - previously failed with deserialization error!
	let body = format!(
		"name=Test+Proj+One+Amenity&category=Residential&project_owner_id={}&location_id={}&amenities={}",
		owner.id, location.id, a1
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::SEE_OTHER);

	// 3. MULTIPLE AMENITIES: should succeed and redirect (303)
	let body = format!(
		"name=Test+Proj+Multi+Amenities&category=Residential&project_owner_id={}&location_id={}&amenities={}&amenities={}",
		owner.id, location.id, a1, a2
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::SEE_OTHER);

	// 4. DUPLICATE AMENITIES: should succeed and deduplicate without error
	let body = format!(
		"name=Test+Proj+Dup+Amenities&category=Residential&project_owner_id={}&location_id={}&amenities={}&amenities={}",
		owner.id, location.id, a1, a1
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::SEE_OTHER);

	// 5. INVALID OWNER ID (non-existent foreign key): must return 400 Bad Request, NOT 500
	let body = format!(
		"name=Test+Proj+Bad+Owner&category=Residential&project_owner_id=999999&location_id={}",
		location.id
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	// 6. MISSING REQUIRED FIELD (missing name): must return 400 Bad Request, NOT 500
	let body = format!(
		"category=Residential&project_owner_id={}&location_id={}",
		owner.id, location.id
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	// 7. EMPTY NAME: must return 400 Bad Request, NOT 500
	let body = format!(
		"name=+&category=Residential&project_owner_id={}&location_id={}",
		owner.id, location.id
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	// 8. MALFORMED NUMERIC INPUT (owner ID = "abc"): must return 400 Bad Request, NOT 500
	let body = format!(
		"name=Test+Proj+Malformed+Num&category=Residential&project_owner_id=abc&location_id={}",
		location.id
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	// 9. MALFORMED NUMERIC AMENITY (amenities = "xyz"): must return 400 Bad Request, NOT 500
	let body = format!(
		"name=Test+Proj+Malformed+Amenity&category=Residential&project_owner_id={}&location_id={}&amenities=xyz",
		owner.id, location.id
	);
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri("/admin/projects/new")
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::BAD_REQUEST);

	// Clean up created test projects
	let _ = sqlx::query!("DELETE FROM projects WHERE name LIKE 'Test Proj %'")
		.execute(&state.model.db)
		.await;
}
