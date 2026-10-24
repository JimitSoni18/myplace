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

#[derive(Clone, Debug)]
pub struct PublicPropertyTypeOption {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
}

#[derive(Clone, Debug)]
pub struct ActiveFilterBadge {
	pub label: String,
	pub remove_url: String,
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
	pub property_types: Vec<PublicPropertyTypeOption>,
	pub selected_property_types: Vec<String>,
	pub selected_bhks: Vec<String>,
	pub min_price: Option<f64>,
	pub max_price: Option<f64>,
	pub active_badges: Vec<ActiveFilterBadge>,
	pub has_active_filters: bool,
}

pub fn build_properties_filter_url(
	category: Option<&str>,
	listing_type: Option<&str>,
	q: Option<&str>,
	bhks: &[String],
	prop_types: &[String],
	min_price: Option<f64>,
	max_price: Option<f64>,
) -> String {
	let mut pairs: Vec<(&str, String)> = Vec::new();
	if let Some(cat) = category.filter(|s| !s.trim().is_empty()) {
		pairs.push(("category", cat.to_string()));
	}
	if let Some(lt) = listing_type.filter(|s| !s.trim().is_empty()) {
		pairs.push(("listing_type", lt.to_string()));
	}
	if let Some(query) = q.filter(|s| !s.trim().is_empty()) {
		pairs.push(("q", query.to_string()));
	}
	for b in bhks {
		pairs.push(("bhk", b.to_string()));
	}
	for pt in prop_types {
		pairs.push(("prop_type", pt.to_string()));
	}
	if let Some(min) = min_price {
		pairs.push(("min_price", min.to_string()));
	}
	if let Some(max) = max_price {
		pairs.push(("max_price", max.to_string()));
	}

	if pairs.is_empty() {
		"/properties".to_string()
	} else {
		let query = serde_urlencoded::to_string(&pairs).unwrap_or_default();
		format!("/properties?{query}")
	}
}

impl<'a> PropertiesListTemplate<'a> {
	pub fn is_bhk_selected(&self, val: &str) -> bool {
		self.selected_bhks.iter().any(|b| b == val)
	}

	pub fn is_type_selected(&self, slug: &str) -> bool {
		self.selected_property_types.iter().any(|s| s == slug)
	}

	pub fn is_budget_preset(&self, min: Option<f64>, max: Option<f64>) -> bool {
		self.min_price == min && self.max_price == max
	}

	pub fn is_category(&self, cat: &str) -> bool {
		self.category_filter == Some(cat)
	}

	pub fn is_listing_type(&self, lt: &str) -> bool {
		self.listing_type_filter == Some(lt)
	}

	pub fn is_rental(&self) -> bool {
		self.listing_type_filter == Some("rent") || self.listing_type_filter == Some("lease")
	}

	pub fn show_bhk_filter(&self) -> bool {
		match self.category_filter {
			Some("COMMERCIAL") | Some("LAND") => false,
			_ => true,
		}
	}

	pub fn has_budget_filter(&self) -> bool {
		self.min_price.is_some() || self.max_price.is_some()
	}

	pub fn selected_types_count(&self) -> usize {
		self.selected_property_types.len()
	}

	pub fn is_type_in_active_category(&self, pt_category: &str) -> bool {
		match self.category_filter {
			Some(cat) => cat == pt_category,
			None => true,
		}
	}

	pub fn category_url(&self, new_cat: Option<&str>) -> String {
		let bhks = if new_cat == Some("COMMERCIAL") || new_cat == Some("LAND") {
			Vec::new()
		} else {
			self.selected_bhks.clone()
		};

		let prop_types: Vec<String> = match new_cat {
			Some(cat) => self
				.selected_property_types
				.iter()
				.filter(|slug| {
					self.property_types
						.iter()
						.any(|pt| &pt.slug == *slug && pt.category == cat)
				})
				.cloned()
				.collect(),
			None => self.selected_property_types.clone(),
		};

		build_properties_filter_url(
			new_cat,
			self.listing_type_filter,
			self.search_query,
			&bhks,
			&prop_types,
			self.min_price,
			self.max_price,
		)
	}
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
