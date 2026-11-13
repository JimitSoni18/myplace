use askama::Template;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct PublicProjectCard {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub project_type_name: String,
	pub city: String,
	pub area: String,
	pub formatted_address: String,
	pub hero_thumb_url: Option<String>,
	pub configurations_count: i64,
	pub possession_date: Option<String>,
	pub owner_id: i32,
	pub owner_name: String,
	pub owner_slug: String,
	pub min_price_formatted: Option<String>,
	pub bhk_summary: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PublicOwnerCard {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub image_url: Option<String>,
	pub project_count: i64,
}

#[derive(Clone, Debug)]
pub struct CategoryStat {
	pub name: &'static str,
	pub count: i64,
	pub icon: &'static str,
}

#[derive(Clone, Debug)]
pub struct PublicMediaItem {
	pub media_type: String,
	pub url: String,
	pub thumbnail_url: String,
	pub sequence: i16,
}

#[derive(Clone, Debug)]
pub struct PublicDocumentItem {
	pub display_name: String,
	pub doc_type: String,
	pub url: String,
	pub file_size_formatted: String,
}

#[derive(Clone, Debug)]
pub struct PublicOwnerSummary {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub image_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PublicOwnerDetail {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub phone: Option<String>,
	pub email: Option<String>,
	pub website: Option<String>,
	pub image_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PublicProjectDetail {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub project_type_name: String,
	pub formatted_address: String,
	pub city: String,
	pub area: String,
	pub state_or_province: String,
	pub postal_code: Option<String>,
	pub latitude: Option<f64>,
	pub longitude: Option<f64>,
	pub rera_number: Option<String>,
	pub rera_authority: Option<String>,
	pub total_land_area: Option<f64>,
	pub towers_count: Option<i32>,
	pub floors_count: Option<i32>,
	pub units_count: Option<i32>,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PublicConfigurationItem {
	pub id: i32,
	pub name: String,
	pub bhk: Option<f64>,
	pub carpet_area: Option<f64>,
	pub built_up_area: Option<f64>,
	pub super_built_up_area: Option<f64>,
	pub price: Option<f64>,
	pub price_formatted: String,
	pub price_on_request: bool,
	pub floor_plan_url: Option<String>,
	pub floor_plan_thumb_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PublicFloorPlanItem {
	pub id: Uuid,
	pub title: String,
	pub url: String,
	pub thumbnail_url: String,
}

#[derive(Clone, Debug)]
pub struct PublicSpecificationItem {
	pub category: String,
	pub items: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PublicLandmarkItem {
	pub category: String,
	pub name: String,
	pub distance: Option<String>,
	pub duration: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PublicTypeOption {
	pub id: i32,
	pub name: String,
}

#[derive(Clone, Debug)]
pub struct SitemapUrl {
	pub loc: String,
	pub lastmod: Option<String>,
	pub changefreq: &'static str,
	pub priority: &'static str,
}

#[derive(Template)]
#[template(path = "public/home.html")]
pub struct HomeTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<&'a str>,
	pub featured_projects: Vec<PublicProjectCard>,
	pub featured_owners: Vec<PublicOwnerCard>,
	pub categories_stats: Vec<CategoryStat>,
	pub available_cities: Vec<String>,
	pub project_types: Vec<PublicTypeOption>,
}

#[derive(Template)]
#[template(path = "public/projects.html")]
pub struct ProjectsListTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<&'a str>,
	pub projects: Vec<PublicProjectCard>,
	pub project_types: Vec<PublicTypeOption>,
	pub selected_type_id: Option<i32>,
	pub selected_city: Option<&'a str>,
	pub selected_area: Option<&'a str>,
	pub selected_bhk: Option<f64>,
	pub search_query: Option<&'a str>,
	pub available_cities: Vec<String>,
	pub available_areas: Vec<String>,
}

impl<'a> ProjectsListTemplate<'a> {
	pub fn is_type_selected(&self, id: &i32) -> bool {
		self.selected_type_id.as_ref() == Some(id)
	}

	pub fn is_city_selected(&self, city: &str) -> bool {
		self.selected_city == Some(city)
	}
}

#[derive(Template)]
#[template(path = "public/project-detail.html")]
pub struct ProjectDetailTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<String>,
	pub project: PublicProjectDetail,
	pub owner: PublicOwnerSummary,
	pub description_html: String,
	pub gallery: Vec<PublicMediaItem>,
	pub documents: Vec<PublicDocumentItem>,
	pub configurations: Vec<PublicConfigurationItem>,
	pub floor_plans: Vec<PublicFloorPlanItem>,
	pub specifications: Vec<PublicSpecificationItem>,
	pub landmarks: Vec<PublicLandmarkItem>,
	pub amenities: Vec<String>,
	pub enquiry_success: bool,
	pub enquiry_error: Option<String>,
}

#[derive(Template)]
#[template(path = "public/owner-detail.html")]
pub struct OwnerDetailTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<&'a str>,
	pub owner: PublicOwnerDetail,
	pub bio_html: String,
	pub projects: Vec<PublicProjectCard>,
}

#[derive(Template)]
#[template(path = "public/sitemap.xml")]
pub struct SitemapTemplate {
	pub urls: Vec<SitemapUrl>,
}

#[derive(Clone, Debug)]
pub struct SitemapIndexItem {
	pub loc: String,
	pub lastmod: Option<String>,
}

#[derive(Template)]
#[template(path = "public/sitemap_index.xml")]
pub struct SitemapIndexTemplate {
	pub sitemaps: Vec<SitemapIndexItem>,
}
