use std::{fs, path::Path};
use ts_rs::{Config, TS};

/// Exports all Rust API request and response data types to TypeScript in `frontend/src/shared/api/generated/`
pub fn export_all_types() -> Result<(), Box<dyn std::error::Error>> {
	let out_dir = if Path::new("frontend/src").exists() {
		"frontend/src/shared/api/generated"
	} else if Path::new("src/shared").exists() {
		"src/shared/api/generated"
	} else {
		"frontend/src/shared/api/generated"
	};

	let cfg = Config::from_env()
		.with_out_dir(out_dir)
		.with_large_int("number");

	// 1. Export Auth types
	crate::api::auth::LoginPayload::export(&cfg)?;
	crate::api::auth::UserInfo::export(&cfg)?;
	crate::api::auth::LoginResponse::export(&cfg)?;

	// 2. Export Admin types
	crate::api::admin::admin_users::AdminUserListQuery::export(&cfg)?;
	crate::api::admin::admin_users::AdminUserItem::export(&cfg)?;
	crate::api::admin::admin_users::CreateAdminUserPayload::export(&cfg)?;
	crate::api::admin::admin_users::UpdateAdminUserPayload::export(&cfg)?;

	crate::api::admin::amenities::AmenityItem::export(&cfg)?;
	crate::api::admin::amenities::CreateAmenityPayload::export(&cfg)?;

	crate::api::admin::dashboard::DashboardRecentProject::export(&cfg)?;
	crate::api::admin::dashboard::DashboardStats::export(&cfg)?;
	crate::api::admin::dashboard::DashboardResponse::export(&cfg)?;

	crate::api::admin::meta::LookupItem::export(&cfg)?;
	crate::api::admin::meta::MetaOptionsResponse::export(&cfg)?;

	crate::api::admin::project_owners::OwnerListQuery::export(&cfg)?;
	crate::api::admin::project_owners::OwnerListItem::export(&cfg)?;
	crate::api::admin::project_owners::OwnerProjectSummary::export(&cfg)?;
	crate::api::admin::project_owners::OwnerDetailResponse::export(&cfg)?;
	crate::api::admin::project_owners::ProjectOwnerCreatePayload::export(&cfg)?;
	crate::api::admin::project_owners::ProjectOwnerUpdatePayload::export(&cfg)?;

	crate::api::admin::projects::ProjectListQuery::export(&cfg)?;
	crate::api::admin::projects::ProjectListItem::export(&cfg)?;
	crate::api::admin::projects::ProjectMediaItem::export(&cfg)?;
	crate::api::admin::projects::ProjectDocumentItem::export(&cfg)?;
	crate::api::admin::projects::ProjectConfigurationItem::export(&cfg)?;
	crate::api::admin::projects::ProjectFloorPlanItem::export(&cfg)?;
	crate::api::admin::projects::ProjectSpecificationItem::export(&cfg)?;
	crate::api::admin::projects::ProjectLandmarkItem::export(&cfg)?;
	crate::api::admin::projects::ProjectDetailData::export(&cfg)?;
	crate::api::admin::projects::ProjectFormPayload::export(&cfg)?;
	crate::api::admin::projects::ConfigurationFormPayload::export(&cfg)?;
	crate::api::admin::projects::SaveSpecificationsPayload::export(&cfg)?;
	crate::api::admin::projects::SaveLandmarksPayload::export(&cfg)?;
	crate::api::admin::projects::ReorderMediaPayload::export(&cfg)?;

	crate::api::admin::search::SearchQuery::export(&cfg)?;
	crate::api::admin::search::SearchResultProject::export(&cfg)?;
	crate::api::admin::search::SearchResultOwner::export(&cfg)?;
	crate::api::admin::search::SearchResults::export(&cfg)?;

	// 3. Export Owner Portal types
	crate::api::owners::OwnerDashboardProject::export(&cfg)?;
	crate::api::owners::OwnerDashboardStats::export(&cfg)?;
	crate::api::owners::OwnerDashboardResponse::export(&cfg)?;
	crate::api::owners::OwnerProjectMediaItem::export(&cfg)?;
	crate::api::owners::OwnerProjectDocumentItem::export(&cfg)?;
	crate::api::owners::OwnerConfigurationItem::export(&cfg)?;
	crate::api::owners::OwnerProjectDetailResponse::export(&cfg)?;

	// 4. Export Notification types
	crate::EnquiryNotification::export(&cfg)?;
	crate::api::notifications::NotificationsListResponse::export(&cfg)?;

	// 5. Generate barrel index.ts files for clean modular imports
	generate_barrel_indexes(out_dir)?;

	// 6. Export OpenAPI JSON specification
	use utoipa::OpenApi;
	let openapi_json = crate::api::openapi::ApiDoc::openapi().to_pretty_json()?;
	let openapi_path = if Path::new("frontend").exists() {
		"frontend/openapi.json"
	} else {
		"openapi.json"
	};
	fs::write(openapi_path, openapi_json)?;

	Ok(())
}

fn generate_barrel_indexes(base_dir: &str) -> std::io::Result<()> {
	let base = Path::new(base_dir);
	if !base.exists() {
		fs::create_dir_all(base)?;
	}

	let subdirs = ["auth", "admin", "owner", "notifications"];
	for sub in &subdirs {
		let dir = base.join(sub);
		if dir.exists() && dir.is_dir() {
			let mut exports = Vec::new();
			for entry in fs::read_dir(&dir)? {
				let entry = entry?;
				let path = entry.path();
				if path.is_file() {
					if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
						if name != "index" && path.extension().and_then(|s| s.to_str()) == Some("ts") {
							exports.push(format!("export * from \"./{name}\";"));
						}
					}
				}
			}
			exports.sort();
			let content = format!("// Automatically generated barrel export\n{}\n", exports.join("\n"));
			fs::write(dir.join("index.ts"), content)?;
		}
	}

	// Root index.ts
	let root_content = r#"// Automatically generated by `cargo run --bin gen_types`
export * from "./auth";
export * from "./admin";
export * from "./owner";
export * from "./notifications";
"#;
	fs::write(base.join("index.ts"), root_content)?;

	Ok(())
}
