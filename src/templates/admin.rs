use std::fmt::{Debug, Display};

use askama::Template;
use serde::Deserialize;

use crate::api::admin::{locations::LocationItem, project_owners::ProjectList};

#[derive(PartialEq, Eq, Default, Debug)]
pub enum AdminPage {
	#[default]
	Dashboard,
	Locations,
	Owners,
	Projects,
	Properties,
	Users,
	AdminUsers,
	Amenities,
}

impl Display for AdminPage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(&self, f)
	}
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------
pub struct DashboardStat {
	pub label: String,
	pub value: i64,
}

pub struct DashboardRecentProject {
	pub id: i32,
	pub name: String,
	pub owner_name: String,
	pub location: String,
	pub category: String,
	pub total_properties: i64,
	pub available_properties: i64,
}

#[derive(Template, Default)]
#[template(path = "admin/dashboard.html")]
pub struct DashboardTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub total_owners: i64,
	pub total_projects: i64,
	pub total_properties: i64,
	pub total_residential: i64,
	pub total_commercial: i64,
	pub recent_projects: Vec<DashboardRecentProject>,
}

// ---------------------------------------------------------------------------
// Generic form helpers
// ---------------------------------------------------------------------------
pub struct EditFormValues<'a, T> {
	pub default_values: T,
	pub error: Option<&'a str>,
}

// ---------------------------------------------------------------------------
// Locations
// ---------------------------------------------------------------------------
pub type LocationEditFormValues<'a> = EditFormValues<'a, LocationItem>;

#[derive(Template, Default)]
#[template(path = "admin/location-list.html")]
pub struct LocationTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub locations: Vec<LocationItem>,
	pub q: Option<&'a str>,
}

#[derive(Template, Default)]
#[template(path = "admin/location-form.html")]
pub struct LocationForm<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub edit_values: Option<LocationEditFormValues<'a>>,
}

// ---------------------------------------------------------------------------
// Project Owners
// ---------------------------------------------------------------------------
#[derive(Template, Default)]
#[template(path = "admin/project-owner-list.html")]
pub struct ProjectOwnerTemplate<'a> {
	pub page: AdminPage,
	pub admin_name: &'a str,
	pub owners: Vec<ProjectList>,
	pub q: Option<&'a str>,
}

/// Edit-form data for an existing owner.
#[derive(Deserialize)]
pub struct ProjectOwnerEditItem {
	pub id: i32,
	pub name: String,
	pub bio: Option<String>,
	pub username: String,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub active: bool,
	pub profile_image_url: Option<String>,
	pub profile_thumb_url: Option<String>,
}

pub type ProjectOwnerEditValues<'a> = EditFormValues<'a, ProjectOwnerEditItem>;

#[derive(Template)]
#[template(path = "admin/project-owner-form.html")]
pub struct ProjectOwnerFormTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub edit_values: Option<ProjectOwnerEditValues<'a>>,
}

pub struct ProjectOwnerDetailItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub username: String,
	pub bio: Option<String>,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub active: bool,
	pub profile_image_url: Option<String>,
	pub profile_thumb_url: Option<String>,
	pub created_at: String,
	pub projects_count: i64,
}

pub struct OwnerProjectItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location: String,
	pub properties_count: i64,
}

#[derive(Template)]
#[template(path = "admin/project-owner-detail.html")]
pub struct ProjectOwnerDetailTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub owner: ProjectOwnerDetailItem,
	pub recent_projects: Vec<OwnerProjectItem>,
}

#[derive(Template)]
#[template(path = "admin/project-owner-projects.html")]
pub struct ProjectOwnerProjectsTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub owner_id: i32,
	pub owner_name: String,
	pub projects: Vec<OwnerProjectItem>,
}

// ---------------------------------------------------------------------------
// Amenities
// ---------------------------------------------------------------------------
pub struct AmenityItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub is_active: bool,
	pub projects_count: i64,
}

#[derive(Template)]
#[template(path = "admin/amenity-list.html")]
pub struct AmenityTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub amenities: Vec<AmenityItem>,
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------
pub struct ProjectListItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub owner_name: String,
	pub category: String,
	pub location: String,
	pub hero_thumb_url: Option<String>,
	pub properties_count: i64,
	pub media_count: i64,
}

pub struct OwnerOptionItem {
	pub id: i32,
	pub name: String,
}

pub struct LocationOptionItem {
	pub id: i32,
	pub address: String,
}

pub struct AmenityOptionItem {
	pub id: i32,
	pub name: String,
	pub selected: bool,
}

#[derive(Template)]
#[template(path = "admin/project-list.html")]
pub struct ProjectTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub projects: Vec<ProjectListItem>,
	pub owners: Vec<OwnerOptionItem>,
	pub selected_owner_id: Option<i32>,
	pub selected_category: Option<String>,
	pub q: Option<&'a str>,
}

pub struct ProjectEditData {
	pub id: i32,
	pub owner_id: i32,
	pub location_id: i32,
	pub name: String,
	pub description: Option<String>,
	pub category: String,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/project-form.html")]
pub struct ProjectFormTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub owners: Vec<OwnerOptionItem>,
	pub locations: Vec<LocationOptionItem>,
	pub amenities: Vec<AmenityOptionItem>,
	pub edit_project: Option<ProjectEditData>,
}

pub struct ProjectDetailData {
	pub id: i32,
	pub owner_id: i32,
	pub owner_name: String,
	pub location_id: i32,
	pub location_name: String,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
	pub created_at: String,
}

pub struct ProjectMediaItem {
	pub id: uuid::Uuid,
	pub url: String,
	pub thumbnail_url: String,
	pub sequence: i16,
}

pub struct ProjectDocumentItem {
	pub id: i32,
	pub media_id: uuid::Uuid,
	pub display_name: String,
	pub doc_type: String,
	pub url: String,
	pub file_size_formatted: String,
}

pub struct ProjectPropertySummary {
	pub total: i64,
	pub residential: i64,
	pub commercial: i64,
	pub land: i64,
}

#[derive(Template)]
#[template(path = "admin/project-detail.html")]
pub struct ProjectDetailTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub project: ProjectDetailData,
	pub description_html: String,
	pub amenities: Vec<String>,
	pub media: Vec<ProjectMediaItem>,
	pub documents: Vec<ProjectDocumentItem>,
	pub property_summary: ProjectPropertySummary,
}

#[derive(Template)]
#[template(path = "admin/project-media.html")]
pub struct ProjectMediaTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub project_id: i32,
	pub project_name: String,
	pub media: Vec<ProjectMediaItem>,
}

#[derive(Template)]
#[template(path = "admin/project-documents.html")]
pub struct ProjectDocumentsTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub project_id: i32,
	pub project_name: String,
	pub documents: Vec<ProjectDocumentItem>,
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

pub struct PropertyListItem {
	pub id: i32,
	pub unit_number: Option<String>,
	pub building: Option<String>,
	pub type_name: String,
	pub area_sqft: Option<f64>,
	pub listing_type: String,
	pub status: String,
	pub price_formatted: String,
	pub media_count: i64,
	pub details_summary: String,
}

#[derive(Template)]
#[template(path = "admin/property-list.html")]
pub struct PropertyListTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub project_id: i32,
	pub project_name: String,
	pub category: String,
	pub properties: Vec<PropertyListItem>,
}

pub struct PropertyGlobalItem {
	pub id: i32,
	pub project_id: i32,
	pub project_name: String,
	pub unit_title: String,
	pub category: String,
	pub type_name: String,
	pub price_formatted: String,
	pub status: String,
}

pub struct ProjectOptionItem {
	pub id: i32,
	pub name: String,
}

#[derive(Template)]
#[template(path = "admin/property-all-list.html")]
pub struct PropertyAllListTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub properties: Vec<PropertyGlobalItem>,
	pub projects: Vec<ProjectOptionItem>,
	pub selected_project_id: Option<i32>,
	pub selected_category: Option<String>,
	pub q: Option<&'a str>,
}

pub struct PropertyTypeOption {
	pub id: i32,
	pub name: String,
}

pub struct PropertyEditData {
	pub id: i32,
	pub property_type_id: i32,
	pub unit_number: Option<String>,
	pub building: Option<String>,
	pub floor_number: Option<i32>,
	pub total_floors: Option<i32>,
	pub built_up_area: Option<f64>,
	pub usable_area: Option<f64>,
	pub description: Option<String>,
	pub listing_type: String,
	pub status: String,
	pub price: Option<f64>,
	pub billing_period: Option<String>,
	// residential
	pub bedroom_count: Option<f64>,
	pub bathroom_count: Option<i16>,
	pub balcony_count: Option<i16>,
	pub is_duplex: bool,
	pub parking: Option<String>,
	// land
	pub parcel_number: Option<String>,
	pub zoning: Option<String>,
	pub approval_status: Option<String>,
	pub development_status: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/property-form.html")]
pub struct PropertyFormTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub project_id: i32,
	pub project_name: String,
	pub category: String,
	pub property_types: Vec<PropertyTypeOption>,
	pub amenities: Vec<AmenityOptionItem>,
	pub edit_property: Option<PropertyEditData>,
}

#[derive(Template)]
#[template(path = "admin/property-media.html")]
pub struct PropertyMediaTemplate<'a> {
	pub admin_name: &'a str,
	pub page: AdminPage,
	pub project_id: i32,
	pub project_name: String,
	pub property_id: i32,
	pub unit_title: String,
	pub media: Vec<ProjectMediaItem>,
}




