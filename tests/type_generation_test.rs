#[test]
fn test_typescript_types_generation() {
	let res = myplace::api::types::export_all_types();
	assert!(res.is_ok(), "TypeScript types generation failed: {:?}", res.err());
}
