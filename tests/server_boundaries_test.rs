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
async fn test_public_server_does_not_expose_admin_or_owner_routes() {
	let state = build_app_state().await;
	let app = servers::public_router(state);

	// Public routes exist
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

	// Health check endpoint works
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

	// Admin routes must not be exposed (404)
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/admin")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/admin/projects")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// Owner routes must not be exposed (404)
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/owner")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/owner/projects")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// Auth management routes must not be exposed on public server
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/auth/admin-login")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/auth/owner-login")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_admin_server_boundaries_and_auth() {
	let state = build_app_state().await;
	let app = servers::admin_router(state.clone());

	// Unauthenticated admin access rejected with 401
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/admin")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

	// Admin login route is accessible
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/auth/admin-login")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	// Health check endpoint is accessible unauthenticated
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/healthz")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	// Public routes must not be exposed on admin server
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/projects")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// Owner routes must not be exposed on admin server
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/owner")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// Authenticated admin user can access /admin
	let auth_admin = AuthUser::new(1, "admin".to_string(), UserRole::Admin);
	let session_id = state.session_store.create_session(auth_admin).await;
	let sign = get_signature(&session_id);
	let cookie_val = format!("session_id={}.{sign}", session_id.0);

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/admin")
				.header(header::COOKIE, cookie_val)
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_owner_server_boundaries_and_auth() {
	let state = build_app_state().await;
	let app = servers::owner_router(state.clone());

	// Unauthenticated owner access rejected with 401
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/owner")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

	// Owner login route is accessible
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/auth/owner-login")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	// Health check endpoint is accessible unauthenticated
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/healthz")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	// Admin routes must not be exposed on owner server
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/admin")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// Public routes must not be exposed on owner server
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/projects")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
