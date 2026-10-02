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
async fn test_property_numeric_values_and_overflow_protection() {
	let state = build_app_state().await;
	let app = servers::admin_router(state.clone());
	let cookie = get_admin_cookie(&state).await;

	// Fetch a project and residential property type
	let proj = sqlx::query!("SELECT id FROM projects WHERE deleted_at IS NULL LIMIT 1")
		.fetch_one(&state.model.db)
		.await
		.expect("A project should exist");
	let project_id = proj.id;

	let prop_type =
		sqlx::query!("SELECT id FROM property_types WHERE category = 'RESIDENTIAL' LIMIT 1")
			.fetch_one(&state.model.db)
			.await
			.expect("A residential property type should exist");
	let type_id = prop_type.id;

	// -----------------------------------------------------------------------
	// 1. Normal values: 2 BHK, 2 Bathrooms
	// -----------------------------------------------------------------------
	let body = format!(
		"property_type_id={type_id}&unit_number=101&building=Tower+1&built_up_area=1200&usable_area=1000&listing_type=sale&status=active&price=5000000&bedroom_count=2&bathroom_count=2&balcony_count=1"
	);
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!(
					"/admin/projects/{project_id}/properties/residential/new"
				))
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// -----------------------------------------------------------------------
	// 2. Fractional values: 2.5 BHK, 1.5 Bathrooms
	// -----------------------------------------------------------------------
	let body = format!(
		"property_type_id={type_id}&unit_number=102&building=Tower+1&built_up_area=1350.5&usable_area=1120.25&listing_type=sale&status=active&price=6500000&bedroom_count=2.5&bathroom_count=1.5&balcony_count=2"
	);
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!(
					"/admin/projects/{project_id}/properties/residential/new"
				))
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// Verify fractional values were accurately saved in database
	let row = sqlx::query!(
		r#"
		SELECT rpd.bedroom_count::float8 as "bedroom_count?", rpd.bathroom_count::float8 as "bathroom_count?"
		FROM properties p
		JOIN residential_property_details rpd ON rpd.property_id = p.id
		WHERE p.project_id = $1 AND p.unit_number = '102'
		"#,
		project_id
	)
	.fetch_one(&state.model.db)
	.await
	.unwrap();
	assert_eq!(row.bedroom_count, Some(2.5));
	assert_eq!(row.bathroom_count, Some(1.5));

	// -----------------------------------------------------------------------
	// 3. Large valid values: 10 BHK, 12 Bathrooms (exceeds old NUMERIC(2,1) limit of 9.9)
	// -----------------------------------------------------------------------
	let body = format!(
		"property_type_id={type_id}&unit_number=Penthouse-10&building=Tower+1&built_up_area=8500&usable_area=7200&listing_type=sale&status=active&price=85000000&bedroom_count=10&bathroom_count=12&balcony_count=4"
	);
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!(
					"/admin/projects/{project_id}/properties/residential/new"
				))
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// Verify large values in database
	let row = sqlx::query!(
		r#"
		SELECT p.id, rpd.bedroom_count::float8 as "bedroom_count?", rpd.bathroom_count::float8 as "bathroom_count?"
		FROM properties p
		JOIN residential_property_details rpd ON rpd.property_id = p.id
		WHERE p.project_id = $1 AND p.unit_number = 'Penthouse-10'
		"#,
		project_id
	)
	.fetch_one(&state.model.db)
	.await
	.unwrap();
	let penthouse_id = row.id;
	assert_eq!(row.bedroom_count, Some(10.0));
	assert_eq!(row.bathroom_count, Some(12.0));

	// -----------------------------------------------------------------------
	// 4. Very large valid values: 99.5 BHK, 50 Bathrooms
	// -----------------------------------------------------------------------
	let body = format!(
		"property_type_id={type_id}&unit_number=Mansion-99&building=Estate&built_up_area=50000&listing_type=sale&status=active&price=500000000&bedroom_count=99.5&bathroom_count=50"
	);
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!(
					"/admin/projects/{project_id}/properties/residential/new"
				))
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// -----------------------------------------------------------------------
	// 5. Zero values: 0 BHK (Studio), 0 Balconies
	// -----------------------------------------------------------------------
	let body = format!(
		"property_type_id={type_id}&unit_number=Studio-01&building=Tower+2&built_up_area=450&listing_type=rent&status=active&price=25000&bedroom_count=0&bathroom_count=1&balcony_count=0"
	);
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!(
					"/admin/projects/{project_id}/properties/residential/new"
				))
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// -----------------------------------------------------------------------
	// 6. Negative values: must return 400 Bad Request (NEVER 500)
	// -----------------------------------------------------------------------
	let negative_cases = vec![
		(
			"negative bedroom",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&bedroom_count=-1"
			),
		),
		(
			"negative bathroom",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&bathroom_count=-2"
			),
		),
		(
			"negative balcony",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&balcony_count=-1"
			),
		),
		(
			"negative price",
			format!("property_type_id={type_id}&listing_type=sale&status=active&price=-500"),
		),
		(
			"negative area",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&built_up_area=-100"
			),
		),
	];

	for (desc, body) in negative_cases {
		let res = app
			.clone()
			.oneshot(
				Request::builder()
					.method("POST")
					.uri(format!(
						"/admin/projects/{project_id}/properties/residential/new"
					))
					.header(header::COOKIE, &cookie)
					.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
					.body(Body::from(body))
					.unwrap(),
			)
			.await
			.unwrap();
		assert_eq!(
			res.status(),
			StatusCode::BAD_REQUEST,
			"Failed on case '{desc}': expected 400 Bad Request, got {}",
			res.status()
		);
	}

	// -----------------------------------------------------------------------
	// 7. Excessive out-of-range values: must return 400 Bad Request (NEVER 500)
	// -----------------------------------------------------------------------
	let excessive_cases = vec![
		(
			"excessive bedrooms",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&bedroom_count=5000"
			),
		),
		(
			"excessive bathrooms",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&bathroom_count=2000"
			),
		),
		(
			"excessive balconies",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&balcony_count=1500"
			),
		),
	];

	for (desc, body) in excessive_cases {
		let res = app
			.clone()
			.oneshot(
				Request::builder()
					.method("POST")
					.uri(format!(
						"/admin/projects/{project_id}/properties/residential/new"
					))
					.header(header::COOKIE, &cookie)
					.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
					.body(Body::from(body))
					.unwrap(),
			)
			.await
			.unwrap();
		assert_eq!(
			res.status(),
			StatusCode::BAD_REQUEST,
			"Failed on case '{desc}': expected 400 Bad Request, got {}",
			res.status()
		);
	}

	// -----------------------------------------------------------------------
	// 8. Malformed numeric values: must return 400 Bad Request (NEVER 500)
	// -----------------------------------------------------------------------
	let malformed_cases = vec![
		(
			"non-numeric bedroom",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&bedroom_count=three"
			),
		),
		(
			"non-numeric bathroom",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=100000&bathroom_count=xyz"
			),
		),
		(
			"non-numeric price",
			format!(
				"property_type_id={type_id}&listing_type=sale&status=active&price=call_for_price"
			),
		),
	];

	for (desc, body) in malformed_cases {
		let res = app
			.clone()
			.oneshot(
				Request::builder()
					.method("POST")
					.uri(format!(
						"/admin/projects/{project_id}/properties/residential/new"
					))
					.header(header::COOKIE, &cookie)
					.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
					.body(Body::from(body))
					.unwrap(),
			)
			.await
			.unwrap();
		assert_eq!(
			res.status(),
			StatusCode::BAD_REQUEST,
			"Failed on malformed case '{desc}': expected 400 Bad Request, got {}",
			res.status()
		);
	}

	// -----------------------------------------------------------------------
	// 9. Update existing property with fractional & large values
	// -----------------------------------------------------------------------
	let update_body = format!(
		"property_type_id={type_id}&unit_number=Penthouse-10-Updated&building=Tower+1&built_up_area=9000&usable_area=7800&listing_type=sale&status=active&price=95000000&bedroom_count=16.5&bathroom_count=14.5&balcony_count=6"
	);
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.method("POST")
				.uri(format!(
					"/admin/projects/{project_id}/properties/{penthouse_id}/edit"
				))
				.header(header::COOKIE, &cookie)
				.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
				.body(Body::from(update_body))
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::SEE_OTHER);

	// Verify update in DB
	let updated_row = sqlx::query!(
		r#"
		SELECT rpd.bedroom_count::float8 as "bedroom_count?", rpd.bathroom_count::float8 as "bathroom_count?", rpd.balcony_count
		FROM properties p
		JOIN residential_property_details rpd ON rpd.property_id = p.id
		WHERE p.id = $1
		"#,
		penthouse_id
	)
	.fetch_one(&state.model.db)
	.await
	.unwrap();
	assert_eq!(updated_row.bedroom_count, Some(16.5));
	assert_eq!(updated_row.bathroom_count, Some(14.5));
	assert_eq!(updated_row.balcony_count, Some(6));

	// -----------------------------------------------------------------------
	// 10. Public view renders fractional and large specs correctly
	// -----------------------------------------------------------------------
	let public_app = servers::public_router(state.clone());
	let prop_record = sqlx::query!("SELECT slug FROM properties WHERE id = $1", penthouse_id)
		.fetch_one(&state.model.db)
		.await
		.unwrap();

	let res = public_app
		.clone()
		.oneshot(
			Request::builder()
				.uri(format!("/properties/{penthouse_id}-{}", prop_record.slug))
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let bytes = res.into_body().collect().await.unwrap().to_bytes();
	let html = String::from_utf8(bytes.to_vec()).unwrap();
	assert!(html.contains("16.5 BHK"));
	assert!(html.contains("14.5"));
}
