use askama::Template;

use crate::templates::admin::{ProjectDocumentItem, ProjectMediaItem};

pub struct OwnerDashboardProject {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location: String,
	pub properties_count: i64,
	pub hero_thumb_url: Option<String>,
}

#[derive(Template)]
#[template(path = "owner/dashboard.html")]
pub struct OwnerDashboardTemplate<'a> {
	pub owner_name: &'a str,
	pub username: &'a str,
	pub total_projects: i64,
	pub total_properties: i64,
	pub recent_projects: Vec<OwnerDashboardProject>,
}

#[derive(Template)]
#[template(path = "owner/project-list.html")]
pub struct OwnerProjectListTemplate<'a> {
	pub owner_name: &'a str,
	pub username: &'a str,
	pub projects: Vec<OwnerDashboardProject>,
}

pub struct OwnerProjectDetailData {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location_name: String,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
}

#[derive(Template)]
#[template(path = "owner/project-detail.html")]
pub struct OwnerProjectDetailTemplate<'a> {
	pub owner_name: &'a str,
	pub username: &'a str,
	pub project: OwnerProjectDetailData,
	pub description_html: String,
	pub properties_count: i64,
	pub media: Vec<ProjectMediaItem>,
	pub documents: Vec<ProjectDocumentItem>,
}
