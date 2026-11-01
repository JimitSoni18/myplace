fn main() {
	println!("Generating TypeScript bindings from Rust types...");
	if let Err(e) = myplace::api::types::export_all_types() {
		eprintln!("Error exporting TypeScript types: {e}");
		std::process::exit(1);
	}
	println!("Successfully exported TypeScript types to frontend/src/shared/api/generated/");
}
