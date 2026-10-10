should commercial properties and residential properties should be one and the same in the schema so that querying it becomes easy? most of the times, the user will not need to query both properties at the same time.

how should the types of properties be classified? the current schema uses an enum and database fields like bedroom_count

how the logical classifications work:

residential properties will have types like bungalows, tenaments, row houses, duplex, penthouse, flats, plots, etc. and those will also have a bedroom_count (BHK)

commercial properties will not have types as residential ones, and will not have bedroom count; instead it should have properties like office space, showroom, land (like plots), warehouse, factory, shops, etc.
should these types be their separate tables instead of being an enum as it is implemented currently?

also, how can we fit farm houses, villas, etc. in this whole classification?

also there can be types of land such as agricultural land, building approved land, etc. in the future; how should all this be modeled?

both property types should have area (sq ft. or any other more appropriate unified, single unit)

how should all this be modeled, keeping in mind the query efficiency, maintainability, data correctness, and ease of adding new property types and flexibility?

the location schema is also undeveloped containing only a single field (area). It should be more comprehensive, should support search and queries, and have scope of google apis/osm/other api integration support in the future.

all properties can either be for rent, for lease or for sale, there is currently the property status enum for that, idk if should be an enum or something else, or it may be unavailable. for any of that (already sold, or on lease, etc.) while searching for properties, the user will most likely be interested only in the ones that are available. the pricing and currency is also to be implemented for all of these. It doesn't necessarily need to support i18n and l10n for currencies etc. for now, though it should be scalable to support such features in the near future.


for admin login+admin panel, most of the functionality is currently underdeveloped, and lacking functionality and good UX.

explaining the admin login, the admin should be seeded through a seeding script,which needs to be created, it should take the username and password from the env file. currently the admin has access to all resources - project owners, projects that will belong to owners, properties that will belong to projects, other superadmin user crud (rarely used). most of this is under developed/yet to be developed.


(for chatgpt: also include code perspective not just user perspective)
The superadmin user, once created, should be able to perform these actions:
login on path /auth/admin-login, which takes the user to the superadmin panel at /admin. this shows the dashboard with all project count, all project owners count, recently created projects with total/available properties filtered by time in a card, - all of these queries should be efficient and have appropriate indexing and other optimizations in place for scalable performance.
The admin panel page has the sidebar which has this dashboard listed.
The sidebar also has other listing links such as locations, projects, project owners, properties list. These things should not be in the sidebar, atleast not for the admin dashboard, hence should be removed (except project owners).
the location is probably not required, and should be removed from the shema as well, instead, the projects itself will have the location information such as country, state, city/province/region, etc. there can be tables for states, countries, city/province/region then (or should it use external service/libraries). (I will let you - chatgpt decide on this one).
The project owners should should have Name, info/bio, profile picture, and a profile so they can log in. the project owners will not have access to all resources, or other project owners/projects. the project owners should also have an active flag which determines if the project owner can log in - default false.

Creating a project owner should create a profile for that owner as well, and that should be linked. and that profile should not be deletable unless the linked project owner is deleted.

Currently if the session is created using admin user ids, this is easy on auth as it just checks the in-memory session store. this can be moved to redis, or direct database verification can be added so deleted users cannot login. allowing the project owners to login would require remodelling the session/auth architecture, move to token auth if required. I chose session auth as it was easy to implement for initial implementation, and so that admin can delete sessions, logging users out, to allow single/multiple logins as required, etc. This is a required functionality that superadmin has access to, so they can log project owners and other superadmins out, by deleting their sessions.

The project owners should also have other details like contact information, website, etc. this is public visible information, it will have its own page. The listing page for project owners has a search bar in the page layout template that gets rendered to all pages, the search for project owners is applied in the name. There is a link for going to the create page for project owners. The form tab should also have an active status checkbox - allowing project owner to login when enabled.
The list items has project owner name, total projects, edit link and delete button. On clicking edit, it takes user to edit page. the create page has name and description (info/bio) fields, other fields need to be added. The edit page has a pure html tab form, the details tab has the name, info/bio fields, other fields like website, contact info, etc. need to be added.It also has submit and cancel button for submitting and cancelling and going back to listing respectively. the image tab has an image input that handles upload for project owner profile image. the backend handles image upload separately for all kinds of uploads. for project owner profile image uploads. it verifies that the uploaded item is an image, converts images to webp and creates thumbnail in the backend. thumbnails should be shown whereever appropriate. it is stored under a common prefix with project owner id, so cleanup is easy.
The form is split in two tabs to keep form submission clean, with minimal JS, and due to better support of URL encoded form in back-end with type safety.
In the project owner listing, clicking on the project owner name should take the admin to the project owner view page, via a hyperlink. It should show the project owner profile picture, username, and all other info. the sidebar for this page should be different: it should have Projects listing in sidebar, clicking on that should take the user to the project listing for that particular project owner. the project owner id should be passed in the URL for this, as we are trying to be idiomatic and use very little javascript

For the project listing, the list should show the project name, description - markdown - with good markdown editor (maybe something lightweight like tiptap or something better) in the front-end, the city/state of project, project start date, launch date, possession date, price range, BHKs if applicable, primary project type - commercial or residential, rera/bu reports download if necessary, amenities - which may reference to a separate amenities table and may be used for filtering for end users, the project owner id that the project relates to, timestamps like created_at, deleted_at, etc., brochure pdf s3 key, the state/city of project for quick filtering by end user
the schema will also store the slug generated by the backend, whenever the project name, area/location, city are updated. This slug will be used in the dynamic project url for public facing front-end. the slug will be generated every time the project name is updated. It will be in this format: project/<project-name>/<project-id>. If the slug is incorrect, the back-end will redirect using Moved Permanently status code - this applies for all slugs that are used for all resources in dynamic URLs. It should be case insensitive. A reference implementation is already present in the codebase - a function called "slugify".
Project owners table also has a slug, and the same rules also apply to that. The slug for project owners will only have the name, the bio/info will be in the generated page, inside meta tags, or anywhere else where it makes sense for SEO.
Projects also have images, and one project may have multiple images, which is referenced in the project_images table in the schema. Project images stores a sequence number which will be the sequence in which the images are displayed, both to the project owner, and on the public url
The files can be stored in s3, we can add project files schema, and the edit form upload tab can add a documents section for uploading rera/bu reports, etc.

for the future, project may have ratings and reviews section, where user will be able to rate the project on different aspects like infra, amenities, maintenane, etc. with individual points rating, but this is for the future as normal user login is not planned yet.

For the project front-end, the listing should have the name of the project in the list item, edit link and delete button, total number of commercial and residential properties listed within.
the create form should have all the text fields that can be added in url encoded form. the amenities, etc. will also be a part of this.

for edit form, the there will again be 2 tabs, the first tab will have same fields as the add form, while the second tab will have an image upload section with up to 10 images + videos; it will be a responsive grid and user will be able to add more using a button.
if amenities are being added as a separate table, there should be a sidebar entry for it, and the superadmin user should be able to create/update/delete/list/search amenities. the back-end should have apis to do so.
For images, it already uses image crate and webp conversion, for videos, it should use ffmpeg with the ez-ffmpeg crate to convert videos to codec specified in the env (env should have h265 config set), using hwenc/hwdec if available, should keep the same resolution and retain audio and subtitles. if the uploaded video file is larger than 20MB (configured in env), it should reject the upload. it should add all files to a single multipart form, the form should still work if any existing image/video is not updated in the form (by uploading a new file), and the user just submits the form. The back-end api should update the images and sequence numbers as necessary, and remove stale images from s3. while uploading many items, if any item fails, it should display alert on the screen specifying what failed, with reason. this requires javascript in front-end which is fine, and the upload API will also be called using javascript. the upload form tab can use a little bit of javascript to handle multiple file input addition, and if the code gets ugly, little bit of petite-vue is allowed instead, the cdn link should be added. The project images should have key /project/{project-id}/{sequence-number}/image.webp (or video.mkv) and /project/{project-id}/{sequence-number}/thumbnail.webp. The thumbnail size for this will be slight larger than the project owner image thumbnails, as it will be displayed in a larger grid. the images table may also need to be changed, as we need to support both images and videos, so now there will be a media table, with media_type (chatgpt, help me here, is this correct? or what should be done?)., the media type should be image for project owner images and project images and all other images, and should be video for project videos. For project images and videos, there will be a separate project media table that maps projects to media.

On the projects listing, of the admin clicks on the project name it should take the admin to the project view page, it should display the project images and videos, the project details, amenities, info/bio, and all other info (chatgpt: need to discuss the UI here). The sidebar for this page should have the residential properties and commercial properties link
Properties - residential and commercial should be present under a project, both should have their separate sidebar entry; clicking on it should take the admin to the properties, residential or commercial - whichever clicked, and the project link should be passed in the url so the back-end can return the project's properties. there should be a searchbar for searching properties, like other entities. properties should be listed with the details it has, for example, residential properties should be with their bhk count, listed price, area sq ft., house number, etc.(may have tags like duplex, row house, etc.? - chatgpt).
The property item should have an edit link, and delete button. There should also be an add button at the top. On delete, the admin should get a javascript confirmation prompt asking if they want to delete, and all the related images will be deleted. The back-end should handle deleting property media from the table as well as s3. (Chatgpt: Since the property schema is not yet decided for different types of properties, first I will like for you to help me solve all that before adding development instructions for this one.) The property can also have images and videos, up to 20, which should be implemented, in schema, in form with a tab, in back-end. Creating/updating properties should also create a slug with the name, bhk, property status (for sale, for rent), area/locality, city, maybe the sqft area, etc. When creating the property, the property form should automatically take project_id based on the project that is currently opened. similar stuff for commercial properties.
(Chatgpt: Should properties also have info like floor, configuration (bathroom, bedroom, balconies, storage room, etc., if yes, how can we store and use it?), etc. Similar stuff for commercial properties)

If we decide to keep state, city, country, currency, etc in their separate tables, then these should also be on the sidebar for the superadmin to see. (Chatgpt: should the admin be able to edit/delete these?)

Currently the s3 url generated has the aws domain https://myplace.s3.amazonaws.com but it should really be http://myplace.web.docker.localhost:3903/ as the bucket service is hosted locally - see if this localhost domain is present somewhere or can be configured

all text forms are url encoded, image uploads are multipart.

In the future, the front-end may be split and hosted across 3 subdomains, one for superadmin, one for project owners, and one for public access, which will show the projects, project owners, properties, etc. and query as required. First we are focusing on the super admin one, then move to the project owners login, then to public pages.

Do not change the current admin UI layout, and keep things responsive, just modernize the UI/UX

Most of these features are either under-developed or undeveloped.

explanation of session auth and clean up
(FOR CHATGPT):
```rs
use std::{collections::BTreeMap, ops::Deref, sync::Mutex};

use tokio::sync::RwLock;
use uuid::{ClockSequence, ContextV7, Timestamp, Uuid};

use crate::constants::misc::SIX_HOURS_SECONDS;

pub struct SharedContextV7(Mutex<ContextV7>);

pub fn get_shared_context() -> &'static SharedContextV7 {
	static CONTEXT_V7: SharedContextV7 = SharedContextV7(Mutex::new(ContextV7::new()));
	&CONTEXT_V7
}

impl ClockSequence for SharedContextV7 {
	type Output = u64;

	fn generate_sequence(&self, seconds: u64, subsec_nanos: u32) -> Self::Output {
		self.0.generate_sequence(seconds, subsec_nanos)
	}

	fn generate_timestamp_sequence(
		&self,
		seconds: u64,
		subsec_nanos: u32,
	) -> (Self::Output, u64, u32) {
		self.0.generate_timestamp_sequence(seconds, subsec_nanos)
	}

	fn usable_bits(&self) -> usize
	where
		Self::Output: Sized,
	{
		42
	}
}

#[derive(Clone)]
pub struct AuthUser {
	pub user_id: i32,
	pub started_at: Timestamp,
	pub expires_at: Timestamp,
    pub username: String,
	// TODO: client platform information: ip, browser, location, etc.
}

impl AuthUser {
	pub fn new(user_id: i32, username: String) -> Self {
		let shared_context = get_shared_context();
		let started_at = Timestamp::now(shared_context);
		let (seconds, subsec_nanos) = started_at.to_unix();
		let expires_at =
			Timestamp::from_unix(shared_context, seconds + SIX_HOURS_SECONDS, subsec_nanos);

		Self {
			user_id,
            username,
			started_at,
			expires_at,
		}
	}

	pub fn with_creation_timestamp(self, started_at: Timestamp) -> Self {
		Self { started_at, ..self }
	}
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct SessionId(pub Uuid);

impl Deref for SessionId {
	type Target = Uuid;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Default)]
pub struct SessionStore {
	pub sessions: RwLock<BTreeMap<SessionId, AuthUser>>,
}

pub enum SessionRenewError {
	SessionNotFound,
}

impl SessionStore {
	fn get_new_session_id() -> SessionId {
		SessionId(uuid::Uuid::new_v7(Timestamp::now(get_shared_context())))
	}

	pub async fn create_session(&self, auth_user: AuthUser) -> SessionId {
		let session_id = Self::get_new_session_id();
		self.sessions.write().await.insert(session_id, auth_user);
		session_id
	}

	pub async fn delete_session(&self, session_id: &SessionId) -> bool {
		self.sessions.write().await.remove(session_id).is_some()
	}

	pub async fn delet_all_user_sessions(&self, user_id: i32) {
		self.sessions
			.write()
			.await
			.retain(|_, v| v.user_id != user_id);
	}

	/// renews the session, and returns the new session id, keeping `started_at` as is
	pub async fn renew_session(
		&self,
		session_id: &SessionId,
	) -> Result<SessionId, SessionRenewError> {
		let mut guard = self.sessions.write().await;
		let AuthUser {
			started_at,
			user_id,
            username,
			..
		} = guard
			.remove(session_id)
			.ok_or(SessionRenewError::SessionNotFound)?;
		let new_auth_user = AuthUser::new(user_id, username).with_creation_timestamp(started_at);

		let new_session_id = Self::get_new_session_id();

		guard
			.insert(new_session_id, new_auth_user)
			.expect("should not have new uuid");

		Ok(new_session_id)
	}

	// TODO: implement
	pub async fn clean_up() {}
}
```

TODO: improve session storage abstraction without making it overly complicated, so we can move to something like redis or token auth without much difficulty, when required. periodic clean-up of dead session needs to be worked upon, under the abstraction itself if possible, else as a separate task.

TODO: for next steps, meta tags, dynamic sitemaps, etc. should be integrated in the project, properties, project owners for all the pages, this is required for SEO. this can be done using redis cache to cache pages and sitemaps, and regenerate when required on query.

Keep links/resource paths/apis semantic

chatgpt - the next paragraph is for you to help with:
I am thinking to have 3 separate front-ends, one for admin users, that have access to all resources, one for project owners, where they can manage their own resources, and one for visitors - the public facing UI. All that I descrived above was mostly related to admin login and dashboard front-end.
If we move ahead with this consideration, we may have to host 3 separate servers on 3 separate ports. For the project-owner front-end, we will need another dashboard/management system. this will mostly be similar to what we have for the admin, except, the project owner will only have access to their own projects. they will have a dashboard, which will show a card with all their recent projects, etc. with time, we can include better diagnostics in the dashboard, like property page visits, location wise property popularity, etc. with complex filters and all. This dashboard will also have a sidebar, but will not have project owners page, as they only have access to their own projects. it will have projects in sidebar, and project view page will then have residential and commercial properties in the sidebar.

(Chatgpt: is having this nested UX good? like properties are inside projects, which are inside project owners...?, if yes, that is fine then; if no, what do you suggest?)

Backend should implement access control for accessing these pages.

the admin should be able to log in into the project owner dashboard, and the project owner APIs should handle authentication and authorization for admins.
the session store should be able to handle both types of users.

now moving to the public pages/UI, chatgpt - the next paragraph is for you:
provide some good instructions and UI suggestions so that it can generate a good UI.
the UI should again be server rendered using the askama templates. but generating UI on every request is not ideal, since the UI may not change frequently.
so the html should be generated and stored, then added in an in-memory in an lfu cache with capacity take from env - default 50. This cache is okay for now but can and should be improved to support horizontal scalability in the future
The stored html should be in a directory that should be served by the server. the caching mentioned above can also be moved to the conent serving phase if more appropriate.
there should also be a sliding window rate limit per owner/admin to generate pages. This may not be ideal, since the server can still choke if multiple owners are trying this at the same time, so if you have any better ideas, consider those as well, otherwise we can improve this in the future.
Although there is rate limiting, the admin/project owner should not be blocked waiting for his page; the data should be updated and the page update job should be stored somewhere (maybe create a table in database), and every time the owner/admin updates the page content, the job should be updated rather than if it is not started yet.
the geneartion should run in background in the same app for now, picking from the db queue and running one by one.
the job should not have everything, it should fetch everything to build the page, from the database. can you suggest how to design build this. It needs to be simple, quick and fast
OR
if you want to skip the queue and background job, rate limit, etc. for the current implementation, you can skip that, and generate it within the entity update request.
For the SEO, the pages genearted should have appropriate optimizations, SEO optimizations, lighthouse optimizations, meta tags, the project text, sitemap and robots.txt. internal links and sites should not leak out.
the site map should be generated and updated based on the updates performed on the database. chatgpt - add good instructions based on the context you have gathered from this prompt to generate a good sitemap.
home page should have good CSS animations if required, which do not slow everything down, modern UI, swiper/carousel if required which can use a bit of javascript if required, if enteractivity is required - petite vue, pages should have good font, etc. the options that we have should be available, for example, download brochure, download rera/other docs, etc. and all other features. chatgpt - just add instructions to make a good modern UI for all pages - home, project, property, project owner, etc. that is mostly static, server generated, optimized with good SEO. I do not want to add excessive instructions for this one, as there are already definitions of what I want.
there should be appropriate filters and search for the entities, for example, for property:- location (city), property type, budget, bhk if residential property, etc. the back-end APIs should correctly implement these, and schema and queries should be optimized.
In the future, the home page needs to have items based on the gussed location of the user, recency, etc. which can be integrated later.

The public ui also needs to have a user form for properties and projects. There should be a form which asks for the user name(required), phone number(required), email(optional), free text(max 255 chars).
when the user fills out the form, the form should be stored in it's respective table, and the form should have the entity ID, of respective project or property. these forms should be visible to the admin/owners.
it should have a read flag, as it is supposed to be shown as in-app notifications. the admin-dashboard and owner dashboard should have a notification icon in the top bar.
new forms should be shown as notifications to admin and project owners. admin can see all forms and notifications, owners can see forms and notifications corresponding to their project and property. chatgpt: decide how the ui, schema and code be for this one.
we need to implement push notifications for this in the future. if possible and required, we can to make these dashboards progressive web apps.

chatgpt:
provide instructions for good the code structure, separation of concerns, 3 different ports to host 3 UIs, restful API, slug where required, PRG pattern for forms, endpoints for APIs and pages, queries in a separate folder, rust error handling without anyhow, errors implementing IntoResponse, each handler having it's own error type if required, each module having it's error submodule, meaningful errors, extractors for forms, normal forms be in url-encoded, uploads in multipart, opentelemetry with tracing integration, currently only logging using tracing-subscriber, authentication and authentication for admins and owners, access control among owners, middlewares or other patterns where required; the models like s3 client and sqlx pool/connection should be in axum state, not to put everything in one file, make it maintainable, etc. etc. etc.

TODO: future requirement of allowing project-specific users that can login that can work on genearted leads - will be it's own app with micro service architecture if required - does not need to be implemented right now
