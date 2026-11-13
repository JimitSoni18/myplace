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

#[tokio::test]
async fn test_unified_server_boundaries_and_api() {
	let state = build_app_state().await;
	let app = servers::main_router(state.clone());

	// 1. Public route works
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/robots.txt")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	use http_body_util::BodyExt;
	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();
	let expected = format!(
		"Sitemap: {}/sitemap.xml",
		myplace::config::CONFIG.public_site_origin
	);
	assert!(body.contains(&expected));

	// 2. Health check endpoint works
	let health_resp = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/healthz")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(health_resp.status(), StatusCode::OK);

	// 3. Unauthenticated admin API access rejected with 401 JSON
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/api/admin/dashboard")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

	// 4. Unauthenticated owner API access rejected with 401 JSON
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/api/owner/dashboard")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

	// 5. Authenticated admin user can access /api/admin/dashboard via Bearer token
	let auth_admin = AuthUser::new(1, "admin".to_string(), UserRole::Admin);
	let session_id = state.session_store.create_session(auth_admin).await;
	let sign = get_signature(&session_id);
	let signed_token = format!("{}.{sign}", session_id.0);

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/api/admin/dashboard")
				.header(header::AUTHORIZATION, format!("Bearer {signed_token}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	// 6. Authenticated admin user can access /api/admin/dashboard via session cookie
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/api/admin/dashboard")
				.header(header::COOKIE, format!("session_id={signed_token}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	// 7. Owner user accessing admin API is rejected
	let auth_owner = AuthUser::new(2, "partner".to_string(), UserRole::ProjectOwner);
	let owner_session_id = state.session_store.create_session(auth_owner).await;
	let owner_sign = get_signature(&owner_session_id);
	let owner_token = format!("{}.{owner_sign}", owner_session_id.0);

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/api/admin/dashboard")
				.header(header::AUTHORIZATION, format!("Bearer {owner_token}"))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_all_frontend_admin_and_owner_routes_exist() {
	let state = build_app_state().await;
	let app = servers::main_router(state);

	let required_admin_routes = [
		"/api/admin/dashboard",
		"/api/admin/meta/options",
		"/api/admin/search",
		"/api/admin/users",
		"/api/admin/amenities",
		"/api/admin/locations/suggestions",
		"/api/admin/owners",
		"/api/admin/project_owners",
		"/api/admin/projects",
	];

	for route in required_admin_routes {
		let response = app
			.clone()
			.oneshot(
				Request::builder()
					.uri(route)
					.body(Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();

		assert_ne!(
			response.status(),
			StatusCode::NOT_FOUND,
			"Route {route} was not found (returned 404)"
		);
		assert_eq!(
			response.status(),
			StatusCode::UNAUTHORIZED,
			"Expected route {route} to be protected by admin auth middleware"
		);
	}

	let required_owner_routes = [
		"/api/owner/dashboard",
		"/api/owner/projects",
	];

	for route in required_owner_routes {
		let response = app
			.clone()
			.oneshot(
				Request::builder()
					.uri(route)
					.body(Body::empty())
					.unwrap(),
			)
			.await
			.unwrap();

		assert_ne!(
			response.status(),
			StatusCode::NOT_FOUND,
			"Route {route} was not found (returned 404)"
		);
		assert_eq!(
			response.status(),
			StatusCode::UNAUTHORIZED,
			"Expected route {route} to be protected by owner auth middleware"
		);
	}
}
