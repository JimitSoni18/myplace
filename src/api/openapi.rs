use utoipa::OpenApi;

use crate::{
	EnquiryNotification,
	api::{
		admin::{
			admin_users::{AdminUserItem, AdminUserListQuery, CreateAdminUserPayload, UpdateAdminUserPayload},
			amenities::{AmenityItem, CreateAmenityPayload},
			dashboard::{DashboardRecentProject, DashboardResponse, DashboardStats},
			locations::{LocationSuggestionQuery, LocationSuggestionsResponse},
			meta::{LookupItem, MetaOptionsResponse},
			project_owners::{
				OwnerDetailResponse, OwnerListItem, OwnerListQuery, OwnerProjectSummary,
				ProjectOwnerCreatePayload, ProjectOwnerUpdatePayload,
			},
			projects::{
				ConfigurationFormPayload, ProjectConfigurationItem, ProjectDetailData,
				ProjectDocumentItem, ProjectFloorPlanItem, ProjectFormPayload, ProjectLandmarkItem,
				ProjectListItem, ProjectListQuery, ProjectMediaItem, ProjectSpecificationItem,
				ReorderMediaPayload, SaveLandmarksPayload, SaveSpecificationsPayload,
			},
			search::{SearchQuery, SearchResultOwner, SearchResultProject, SearchResults},
		},
		auth::{LoginPayload, LoginResponse, UserInfo},
		notifications::NotificationsListResponse,
		owners::{
			OwnerConfigurationItem, OwnerDashboardProject, OwnerDashboardResponse,
			OwnerDashboardStats, OwnerProjectDocumentItem, OwnerProjectMediaItem,
			OwnerProjectDetailResponse,
		},
	},
};

// ---------------------------------------------------------------------------
// Path definitions for OpenAPI 3.1
// ---------------------------------------------------------------------------

// --- AUTH ---

#[utoipa::path(
	post,
	path = "/api/auth/admin/login",
	request_body = LoginPayload,
	responses(
		(status = 200, description = "Admin login successful", body = LoginResponse),
		(status = 401, description = "Invalid credentials")
	),
	tag = "Auth"
)]
pub async fn admin_login() {}

#[utoipa::path(
	post,
	path = "/api/auth/owner/login",
	request_body = LoginPayload,
	responses(
		(status = 200, description = "Owner login successful", body = LoginResponse),
		(status = 401, description = "Invalid credentials")
	),
	tag = "Auth"
)]
pub async fn owner_login() {}

#[utoipa::path(
	post,
	path = "/api/auth/logout",
	responses(
		(status = 204, description = "Logged out successfully")
	),
	tag = "Auth"
)]
pub async fn logout() {}

#[utoipa::path(
	get,
	path = "/api/auth/me",
	responses(
		(status = 200, description = "Current authenticated user", body = UserInfo),
		(status = 401, description = "Unauthorized")
	),
	tag = "Auth"
)]
pub async fn get_current_user() {}

// --- ADMIN DASHBOARD, META, SEARCH ---

#[utoipa::path(
	get,
	path = "/api/admin/dashboard",
	responses(
		(status = 200, description = "Admin dashboard statistics", body = DashboardResponse)
	),
	tag = "Admin Dashboard"
)]
pub async fn get_dashboard() {}

#[utoipa::path(
	get,
	path = "/api/admin/meta/options",
	responses(
		(status = 200, description = "Lookup options for forms", body = MetaOptionsResponse)
	),
	tag = "Admin Meta"
)]
pub async fn get_meta_options() {}

#[utoipa::path(
	get,
	path = "/api/admin/search",
	params(SearchQuery),
	responses(
		(status = 200, description = "Global search results", body = SearchResults)
	),
	tag = "Admin Search"
)]
pub async fn global_search() {}

// --- ADMIN USERS ---

#[utoipa::path(
	get,
	path = "/api/admin/users",
	params(AdminUserListQuery),
	responses(
		(status = 200, description = "List admin users", body = Vec<AdminUserItem>)
	),
	tag = "Admin Users"
)]
pub async fn list_admin_users() {}

#[utoipa::path(
	post,
	path = "/api/admin/users",
	request_body = CreateAdminUserPayload,
	responses(
		(status = 201, description = "Admin user created")
	),
	tag = "Admin Users"
)]
pub async fn create_admin_user() {}

#[utoipa::path(
	get,
	path = "/api/admin/users/{id}",
	params(("id" = i32, Path, description = "Admin user ID")),
	responses(
		(status = 200, description = "Admin user detail", body = AdminUserItem)
	),
	tag = "Admin Users"
)]
pub async fn get_admin_user() {}

#[utoipa::path(
	put,
	path = "/api/admin/users/{id}",
	params(("id" = i32, Path, description = "Admin user ID")),
	request_body = UpdateAdminUserPayload,
	responses(
		(status = 200, description = "Admin user updated")
	),
	tag = "Admin Users"
)]
pub async fn update_admin_user() {}

#[utoipa::path(
	delete,
	path = "/api/admin/users/{id}",
	params(("id" = i32, Path, description = "Admin user ID")),
	responses(
		(status = 204, description = "Admin user deleted")
	),
	tag = "Admin Users"
)]
pub async fn delete_admin_user() {}

// --- ADMIN AMENITIES ---

#[utoipa::path(
	get,
	path = "/api/admin/amenities",
	responses(
		(status = 200, description = "List amenities", body = Vec<AmenityItem>)
	),
	tag = "Admin Amenities"
)]
pub async fn list_amenities() {}

#[utoipa::path(
	post,
	path = "/api/admin/amenities",
	request_body = CreateAmenityPayload,
	responses(
		(status = 201, description = "Amenity created")
	),
	tag = "Admin Amenities"
)]
pub async fn create_amenity() {}

#[utoipa::path(
	post,
	path = "/api/admin/amenities/{id}/toggle",
	params(("id" = i32, Path, description = "Amenity ID")),
	responses(
		(status = 200, description = "Amenity active state toggled")
	),
	tag = "Admin Amenities"
)]
pub async fn toggle_amenity() {}

#[utoipa::path(
	delete,
	path = "/api/admin/amenities/{id}",
	params(("id" = i32, Path, description = "Amenity ID")),
	responses(
		(status = 204, description = "Amenity deleted")
	),
	tag = "Admin Amenities"
)]
pub async fn delete_amenity() {}

// --- ADMIN LOCATIONS ---

#[utoipa::path(
	get,
	path = "/api/admin/locations/suggestions",
	params(LocationSuggestionQuery),
	responses(
		(status = 200, description = "Location suggestions", body = LocationSuggestionsResponse)
	),
	tag = "Admin Locations"
)]
pub async fn get_suggestions() {}

// --- ADMIN PROJECT OWNERS ---

#[utoipa::path(
	get,
	path = "/api/admin/project_owners",
	params(OwnerListQuery),
	responses(
		(status = 200, description = "List project owners", body = Vec<OwnerListItem>)
	),
	tag = "Admin Project Owners"
)]
pub async fn project_owner_list() {}

#[utoipa::path(
	post,
	path = "/api/admin/project_owners",
	request_body = ProjectOwnerCreatePayload,
	responses(
		(status = 201, description = "Project owner created")
	),
	tag = "Admin Project Owners"
)]
pub async fn create_project_owner() {}

#[utoipa::path(
	get,
	path = "/api/admin/project_owners/{id}",
	params(("id" = i32, Path, description = "Project owner ID")),
	responses(
		(status = 200, description = "Project owner detail", body = OwnerDetailResponse)
	),
	tag = "Admin Project Owners"
)]
pub async fn project_owner_detail() {}

#[utoipa::path(
	put,
	path = "/api/admin/project_owners/{id}",
	params(("id" = i32, Path, description = "Project owner ID")),
	request_body = ProjectOwnerUpdatePayload,
	responses(
		(status = 200, description = "Project owner updated")
	),
	tag = "Admin Project Owners"
)]
pub async fn update_project_owner() {}

#[utoipa::path(
	delete,
	path = "/api/admin/project_owners/{id}",
	params(("id" = i32, Path, description = "Project owner ID")),
	responses(
		(status = 204, description = "Project owner deleted")
	),
	tag = "Admin Project Owners"
)]
pub async fn delete_project_owner() {}

#[utoipa::path(
	post,
	path = "/api/admin/project_owners/{id}/toggle-active",
	params(("id" = i32, Path, description = "Project owner ID")),
	responses(
		(status = 200, description = "Project owner active toggled")
	),
	tag = "Admin Project Owners"
)]
pub async fn toggle_active_project_owner() {}

#[utoipa::path(
	post,
	path = "/api/admin/project_owners/{id}/image",
	params(("id" = i32, Path, description = "Project owner ID")),
	responses(
		(status = 200, description = "Profile image uploaded")
	),
	tag = "Admin Project Owners"
)]
pub async fn add_project_owner_image() {}

#[utoipa::path(
	delete,
	path = "/api/admin/project_owners/{id}/image",
	params(("id" = i32, Path, description = "Project owner ID")),
	responses(
		(status = 204, description = "Profile image removed")
	),
	tag = "Admin Project Owners"
)]
pub async fn remove_project_owner_image() {}

// --- ADMIN PROJECTS ---

#[utoipa::path(
	get,
	path = "/api/admin/projects",
	params(ProjectListQuery),
	responses(
		(status = 200, description = "List projects", body = Vec<ProjectListItem>)
	),
	tag = "Admin Projects"
)]
pub async fn project_list() {}

#[utoipa::path(
	post,
	path = "/api/admin/projects",
	request_body = ProjectFormPayload,
	responses(
		(status = 201, description = "Project created")
	),
	tag = "Admin Projects"
)]
pub async fn create_project() {}

#[utoipa::path(
	get,
	path = "/api/admin/projects/{id}",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "Project detail", body = ProjectDetailData)
	),
	tag = "Admin Projects"
)]
pub async fn project_detail() {}

#[utoipa::path(
	put,
	path = "/api/admin/projects/{id}",
	params(("id" = i32, Path, description = "Project ID")),
	request_body = ProjectFormPayload,
	responses(
		(status = 200, description = "Project updated")
	),
	tag = "Admin Projects"
)]
pub async fn update_project() {}

#[utoipa::path(
	delete,
	path = "/api/admin/projects/{id}",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 204, description = "Project deleted")
	),
	tag = "Admin Projects"
)]
pub async fn delete_project() {}

// --- PROJECT CONFIGURATIONS ---

#[utoipa::path(
	get,
	path = "/api/admin/projects/{id}/configurations",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "List configurations", body = Vec<ProjectConfigurationItem>)
	),
	tag = "Admin Projects"
)]
pub async fn list_configurations() {}

#[utoipa::path(
	post,
	path = "/api/admin/projects/{id}/configurations",
	params(("id" = i32, Path, description = "Project ID")),
	request_body = ConfigurationFormPayload,
	responses(
		(status = 201, description = "Configuration created")
	),
	tag = "Admin Projects"
)]
pub async fn create_configuration() {}

#[utoipa::path(
	put,
	path = "/api/admin/projects/{id}/configurations/{cid}",
	params(
		("id" = i32, Path, description = "Project ID"),
		("cid" = i32, Path, description = "Configuration ID")
	),
	request_body = ConfigurationFormPayload,
	responses(
		(status = 200, description = "Configuration updated")
	),
	tag = "Admin Projects"
)]
pub async fn update_configuration() {}

#[utoipa::path(
	delete,
	path = "/api/admin/projects/{id}/configurations/{cid}",
	params(
		("id" = i32, Path, description = "Project ID"),
		("cid" = i32, Path, description = "Configuration ID")
	),
	responses(
		(status = 204, description = "Configuration deleted")
	),
	tag = "Admin Projects"
)]
pub async fn delete_configuration() {}

#[utoipa::path(
	post,
	path = "/api/admin/projects/{id}/configurations/{cid}/floor-plan",
	params(
		("id" = i32, Path, description = "Project ID"),
		("cid" = i32, Path, description = "Configuration ID")
	),
	responses(
		(status = 200, description = "Floor plan uploaded")
	),
	tag = "Admin Projects"
)]
pub async fn upload_configuration_floor_plan() {}

// --- MASTER FLOOR PLANS ---

#[utoipa::path(
	get,
	path = "/api/admin/projects/{id}/floor-plans",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "List master floor plans", body = Vec<ProjectFloorPlanItem>)
	),
	tag = "Admin Projects"
)]
pub async fn list_floor_plans() {}

#[utoipa::path(
	post,
	path = "/api/admin/projects/{id}/floor-plans",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 201, description = "Master floor plan uploaded")
	),
	tag = "Admin Projects"
)]
pub async fn upload_floor_plan() {}

#[utoipa::path(
	delete,
	path = "/api/admin/projects/{id}/floor-plans/{fpid}",
	params(
		("id" = i32, Path, description = "Project ID"),
		("fpid" = Uuid, Path, description = "Floor plan UUID")
	),
	responses(
		(status = 204, description = "Master floor plan deleted")
	),
	tag = "Admin Projects"
)]
pub async fn delete_floor_plan() {}

// --- SPECIFICATIONS & LANDMARKS ---

#[utoipa::path(
	get,
	path = "/api/admin/projects/{id}/specifications",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "Get project specifications", body = Vec<ProjectSpecificationItem>)
	),
	tag = "Admin Projects"
)]
pub async fn get_specifications() {}

#[utoipa::path(
	put,
	path = "/api/admin/projects/{id}/specifications",
	params(("id" = i32, Path, description = "Project ID")),
	request_body = SaveSpecificationsPayload,
	responses(
		(status = 200, description = "Save project specifications")
	),
	tag = "Admin Projects"
)]
pub async fn save_specifications() {}

#[utoipa::path(
	get,
	path = "/api/admin/projects/{id}/landmarks",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "Get project landmarks", body = Vec<ProjectLandmarkItem>)
	),
	tag = "Admin Projects"
)]
pub async fn get_landmarks() {}

#[utoipa::path(
	put,
	path = "/api/admin/projects/{id}/landmarks",
	params(("id" = i32, Path, description = "Project ID")),
	request_body = SaveLandmarksPayload,
	responses(
		(status = 200, description = "Save project landmarks")
	),
	tag = "Admin Projects"
)]
pub async fn save_landmarks() {}

// --- DOCUMENTS & MEDIA ---

#[utoipa::path(
	get,
	path = "/api/admin/projects/{id}/documents",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "List project documents", body = Vec<ProjectDocumentItem>)
	),
	tag = "Admin Projects"
)]
pub async fn project_documents_list() {}

#[utoipa::path(
	post,
	path = "/api/admin/projects/{id}/documents",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 201, description = "Document uploaded")
	),
	tag = "Admin Projects"
)]
pub async fn upload_project_document() {}

#[utoipa::path(
	delete,
	path = "/api/admin/projects/{id}/documents/{did}",
	params(
		("id" = i32, Path, description = "Project ID"),
		("did" = Uuid, Path, description = "Document UUID")
	),
	responses(
		(status = 204, description = "Document deleted")
	),
	tag = "Admin Projects"
)]
pub async fn delete_project_document() {}

#[utoipa::path(
	get,
	path = "/api/admin/projects/{id}/media",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "List project media", body = Vec<ProjectMediaItem>)
	),
	tag = "Admin Projects"
)]
pub async fn project_media_list() {}

#[utoipa::path(
	post,
	path = "/api/admin/projects/{id}/media",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 201, description = "Media uploaded")
	),
	tag = "Admin Projects"
)]
pub async fn upload_project_media() {}

#[utoipa::path(
	post,
	path = "/api/admin/projects/{id}/media/reorder",
	params(("id" = i32, Path, description = "Project ID")),
	request_body = ReorderMediaPayload,
	responses(
		(status = 200, description = "Media reordered")
	),
	tag = "Admin Projects"
)]
pub async fn reorder_project_media() {}

#[utoipa::path(
	delete,
	path = "/api/admin/projects/{id}/media/{mid}",
	params(
		("id" = i32, Path, description = "Project ID"),
		("mid" = Uuid, Path, description = "Media UUID")
	),
	responses(
		(status = 204, description = "Media deleted")
	),
	tag = "Admin Projects"
)]
pub async fn delete_project_media() {}

// --- NOTIFICATIONS ---

#[utoipa::path(
	get,
	path = "/api/admin/notifications",
	responses(
		(status = 200, description = "List notifications", body = NotificationsListResponse)
	),
	tag = "Notifications"
)]
pub async fn admin_list_notifications() {}

#[utoipa::path(
	post,
	path = "/api/admin/notifications/{enquiry_type}/{id}/read",
	params(
		("enquiry_type" = String, Path, description = "Enquiry type: project or property"),
		("id" = i32, Path, description = "Notification ID")
	),
	responses(
		(status = 204, description = "Notification marked as read")
	),
	tag = "Notifications"
)]
pub async fn admin_mark_read() {}

#[utoipa::path(
	post,
	path = "/api/admin/notifications/read-all",
	responses(
		(status = 204, description = "All notifications marked as read")
	),
	tag = "Notifications"
)]
pub async fn admin_read_all() {}

#[utoipa::path(
	get,
	path = "/api/owner/notifications",
	responses(
		(status = 200, description = "List owner notifications", body = NotificationsListResponse)
	),
	tag = "Notifications"
)]
pub async fn owner_list_notifications() {}

#[utoipa::path(
	post,
	path = "/api/owner/notifications/{enquiry_type}/{id}/read",
	params(
		("enquiry_type" = String, Path, description = "Enquiry type: project or property"),
		("id" = i32, Path, description = "Notification ID")
	),
	responses(
		(status = 204, description = "Notification marked as read")
	),
	tag = "Notifications"
)]
pub async fn owner_mark_read() {}

#[utoipa::path(
	post,
	path = "/api/owner/notifications/read-all",
	responses(
		(status = 204, description = "All owner notifications marked as read")
	),
	tag = "Notifications"
)]
pub async fn owner_read_all() {}

// --- OWNER PORTAL ---

#[utoipa::path(
	get,
	path = "/api/owner/dashboard",
	responses(
		(status = 200, description = "Owner portal dashboard", body = OwnerDashboardResponse)
	),
	tag = "Owner Portal"
)]
pub async fn owner_dashboard() {}

#[utoipa::path(
	get,
	path = "/api/owner/projects",
	responses(
		(status = 200, description = "Owner projects list", body = Vec<OwnerDashboardProject>)
	),
	tag = "Owner Portal"
)]
pub async fn owner_projects_list() {}

#[utoipa::path(
	get,
	path = "/api/owner/projects/{id}",
	params(("id" = i32, Path, description = "Project ID")),
	responses(
		(status = 200, description = "Owner project detail", body = OwnerProjectDetailResponse)
	),
	tag = "Owner Portal"
)]
pub async fn owner_project_detail() {}

// ---------------------------------------------------------------------------
// OpenApi Document
// ---------------------------------------------------------------------------

#[derive(OpenApi)]
#[openapi(
	paths(
		admin_login,
		owner_login,
		logout,
		get_current_user,
		get_dashboard,
		get_meta_options,
		global_search,
		list_admin_users,
		create_admin_user,
		get_admin_user,
		update_admin_user,
		delete_admin_user,
		list_amenities,
		create_amenity,
		toggle_amenity,
		delete_amenity,
		get_suggestions,
		project_owner_list,
		create_project_owner,
		project_owner_detail,
		update_project_owner,
		delete_project_owner,
		toggle_active_project_owner,
		add_project_owner_image,
		remove_project_owner_image,
		project_list,
		create_project,
		project_detail,
		update_project,
		delete_project,
		list_configurations,
		create_configuration,
		update_configuration,
		delete_configuration,
		upload_configuration_floor_plan,
		list_floor_plans,
		upload_floor_plan,
		delete_floor_plan,
		get_specifications,
		save_specifications,
		get_landmarks,
		save_landmarks,
		project_documents_list,
		upload_project_document,
		delete_project_document,
		project_media_list,
		upload_project_media,
		reorder_project_media,
		delete_project_media,
		admin_list_notifications,
		admin_mark_read,
		admin_read_all,
		owner_list_notifications,
		owner_mark_read,
		owner_read_all,
		owner_dashboard,
		owner_projects_list,
		owner_project_detail,
	),
	components(
		schemas(
			LoginPayload,
			UserInfo,
			LoginResponse,
			AdminUserItem,
			CreateAdminUserPayload,
			UpdateAdminUserPayload,
			AmenityItem,
			CreateAmenityPayload,
			DashboardRecentProject,
			DashboardStats,
			DashboardResponse,
			LookupItem,
			MetaOptionsResponse,
			LocationSuggestionsResponse,
			SearchResultProject,
			SearchResultOwner,
			SearchResults,
			OwnerListItem,
			OwnerProjectSummary,
			OwnerDetailResponse,
			ProjectOwnerCreatePayload,
			ProjectOwnerUpdatePayload,
			ProjectListItem,
			ProjectDetailData,
			ProjectFormPayload,
			ProjectMediaItem,
			ProjectDocumentItem,
			ProjectConfigurationItem,
			ConfigurationFormPayload,
			ProjectFloorPlanItem,
			ProjectSpecificationItem,
			SaveSpecificationsPayload,
			ProjectLandmarkItem,
			SaveLandmarksPayload,
			ReorderMediaPayload,
			EnquiryNotification,
			NotificationsListResponse,
			OwnerDashboardProject,
			OwnerDashboardStats,
			OwnerDashboardResponse,
			OwnerProjectMediaItem,
			OwnerProjectDocumentItem,
			OwnerConfigurationItem,
			OwnerProjectDetailResponse,
		)
	),
	info(
		title = "MyPlace API",
		version = "1.0.0",
		description = "Type-safe OpenAPI specification for MyPlace Real Estate Platform"
	)
)]
pub struct ApiDoc;
