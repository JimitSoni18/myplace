use askama::Template;

#[derive(Clone, Debug)]
pub struct PublicProjectCard {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location_name: String,
	pub hero_thumb_url: Option<String>,
	pub property_count: i64,
	pub possession_date: Option<String>,
	pub owner_id: i32,
	pub owner_name: String,
	pub owner_slug: String,
}

#[derive(Clone, Debug)]
pub struct PublicPropertyCard {
	pub id: i32,
	pub slug: String,
	pub title: String,
	pub category: String,
	pub property_type_name: String,
	pub price_formatted: String,
	pub listing_type: String,
	pub built_up_area: Option<f64>,
	pub bedroom_count: Option<f64>,
	pub bathroom_count: Option<f64>,
	pub hero_thumb_url: Option<String>,
	pub project_id: i32,
	pub project_name: String,
	pub project_slug: String,
	pub location_name: String,
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
	pub slug: &'static str,
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
pub struct PublicProjectSummary {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location_name: String,
}

#[derive(Clone, Debug)]
pub struct PublicProjectDetail {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location_name: String,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PublicPropertyDetail {
	pub id: i32,
	pub slug: String,
	pub unit_number: Option<String>,
	pub building: Option<String>,
	pub floor_number: Option<i32>,
	pub total_floors: Option<i32>,
	pub built_up_area: Option<f64>,
	pub usable_area: Option<f64>,
	pub category: String,
	pub property_type_name: String,
	pub price_formatted: String,
	pub listing_type: String,
	pub billing_period: Option<String>,
	// Subtype specific:
	pub bedroom_count: Option<f64>,
	pub bathroom_count: Option<f64>,
	pub balcony_count: Option<i16>,
	pub is_duplex: bool,
	pub parking: Option<String>,
	pub parcel_number: Option<String>,
	pub zoning: Option<String>,
	pub approval_status: Option<String>,
	pub development_status: Option<String>,
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
	pub featured_properties: Vec<PublicPropertyCard>,
	pub featured_owners: Vec<PublicOwnerCard>,
	pub categories_stats: Vec<CategoryStat>,
}

#[derive(Template)]
#[template(path = "public/projects.html")]
pub struct ProjectsListTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<&'a str>,
	pub category_filter: Option<&'a str>,
	pub search_query: Option<&'a str>,
	pub projects: Vec<PublicProjectCard>,
}

#[derive(Template)]
#[template(path = "public/project-detail.html")]
pub struct ProjectDetailTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<&'a str>,
	pub project: PublicProjectDetail,
	pub owner: PublicOwnerSummary,
	pub description_html: String,
	pub gallery: Vec<PublicMediaItem>,
	pub documents: Vec<PublicDocumentItem>,
	pub properties: Vec<PublicPropertyCard>,
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
#[template(path = "public/properties.html")]
pub struct PropertiesListTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<&'a str>,
	pub category_filter: Option<&'a str>,
	pub listing_type_filter: Option<&'a str>,
	pub search_query: Option<&'a str>,
	pub properties: Vec<PublicPropertyCard>,
}

#[derive(Template)]
#[template(path = "public/property-detail.html")]
pub struct PropertyDetailTemplate<'a> {
	pub meta_title: &'a str,
	pub meta_description: &'a str,
	pub canonical_url: &'a str,
	pub og_image: Option<&'a str>,
	pub property: PublicPropertyDetail,
	pub project: PublicProjectSummary,
	pub owner: PublicOwnerSummary,
	pub description_html: String,
	pub gallery: Vec<PublicMediaItem>,
	pub amenities: Vec<String>,
	pub enquiry_success: bool,
	pub enquiry_error: Option<String>,
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

#[derive(Template)]
#[template(path = "public/sitemap.xml")]
pub struct SitemapTemplate {
	pub urls: Vec<SitemapUrl>,
}
