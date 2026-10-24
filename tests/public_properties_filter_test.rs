use axum::{
	body::Body,
	http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use myplace::{build_app_state, servers};
use tower::ServiceExt;

#[tokio::test]
async fn test_public_properties_filters() {
	let state = build_app_state().await;
	let app = servers::public_router(state.clone());

	// 1. Unfiltered GET /properties returns 200 OK
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8_lossy(&body_bytes);
	assert!(body.contains("Verified Properties Directory"));
	assert!(body.contains("All Categories"));
	assert!(body.contains("Budget"));
	assert!(body.contains("Property Type"));

	// 2. Filter by BHK (multiple values via repeated query params)
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties?bhk=2&bhk=3")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8_lossy(&body_bytes);
	assert!(body.contains("2 BHK"));
	assert!(body.contains("3 BHK"));
	assert!(body.contains("Reset All Filters"));

	// 3. Filter by BHK (comma-separated values)
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties?bhk=2,3")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8_lossy(&body_bytes);
	assert!(body.contains("2 BHK"));
	assert!(body.contains("3 BHK"));

	// 4. Filter by Property Types (multiple)
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties?prop_type=flat&prop_type=villa")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8_lossy(&body_bytes);
	assert!(body.contains("Flat"));
	assert!(body.contains("Reset All Filters"));

	// 5. Filter by Budget Min & Max
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties?min_price=5000000&max_price=10000000")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8_lossy(&body_bytes);
	assert!(body.contains("Budget:"));
	assert!(body.contains("Reset All Filters"));

	// 6. Filter by Budget Preset Range (sent as min_price & max_price)
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties?min_price=5000000&max_price=10000000")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8_lossy(&body_bytes);
	assert!(body.contains("Budget: ₹50 L – ₹1 Cr"));
	assert!(body.contains("Reset All Filters"));

	// 7. Combined / Mixed Filters: Category + Listing Type + BHK + Type + Budget
	let res = app
		.clone()
		.oneshot(
			Request::builder()
				.uri("/properties?category=RESIDENTIAL&listing_type=sale&bhk=2&prop_type=flat&min_price=2000000&max_price=20000000")
				.body(Body::empty())
				.unwrap(),
		)
		.await
		.unwrap();
	assert_eq!(res.status(), StatusCode::OK);
	let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
	let body = String::from_utf8_lossy(&body_bytes);
	assert!(body.contains("Residential Homes"));
	assert!(body.contains("For Sale"));
	assert!(body.contains("2 BHK"));
	assert!(body.contains("Flat"));
	assert!(body.contains("Reset All Filters"));
}
