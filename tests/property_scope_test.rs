use axum::{
	body::Body,
	http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
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
async fn test_properties_are_project_scoped_not_global() {
	let state = build_app_state().await;
	let app = servers::admin_router(state.clone());
	let cookie = get_admin_cookie(&state).await;

	// 1. GET /admin/properties returns 200
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/admin/properties")
				.header(header::COOKIE, &cookie)
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);

	let bytes = response.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8(bytes.to_vec()).unwrap();

	// Must NOT contain link to /admin/properties/new
	assert!(!body.contains("/admin/properties/new"));

	// 2. Global /admin/properties/new must return 404
	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/admin/properties/new")
				.header(header::COOKIE, &cookie)
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::NOT_FOUND);

	// 3. Project-scoped property creation route must be accessible (200 OK)
	let project = sqlx::query!("SELECT id FROM projects WHERE deleted_at IS NULL LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.unwrap();

	let response = app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!(
					"/admin/projects/{}/properties/residential/new",
					project.id
				))
				.header(header::COOKIE, &cookie)
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(response.status(), StatusCode::OK);
}
