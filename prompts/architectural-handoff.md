# EXISTING PROJECT: REAL-ESTATE PLATFORM — AUTONOMOUS DEVELOPMENT BRIEF

You are taking over an existing Rust/Axum/PostgreSQL real-estate application.

Your task is NOT to merely suggest code or build isolated CRUD pages. Your task is to understand the existing repository, establish a coherent architecture, evolve the schema where necessary, and implement the application in a maintainable sequence.

I want you to behave like a senior engineer taking ownership of the project.

You may make reasonable implementation decisions without repeatedly asking me for confirmation.

If something is genuinely ambiguous and making the wrong decision could cause significant rework, stop and ask. Otherwise, choose a sensible solution, document the decision, and continue.

Do not blindly preserve the current unfinished schema. The schema shown below is the starting point, not the final architecture.

---

# 1. PRODUCT OVERVIEW

This is a real-estate platform with three logical user-facing applications:

1. Super-admin application
2. Project-owner application
3. Public website

They may eventually be deployed independently, probably on separate ports/subdomains, but they share the same Rust backend/domain/database/storage infrastructure unless there is a strong reason otherwise.

The development priority is:

1. Super-admin application
2. Project-owner application
3. Public-facing application

Do not prematurely split the backend into microservices.

Keep one maintainable modular Rust backend for now.

Future deployment might look like:

```text
admin.example.com
owners.example.com
www.example.com
```

but local development can use separate ports.

The current implementation should be designed so this separation is possible later without rewriting the domain layer.

---

# 2. EXISTING TECHNOLOGY

The existing application uses:

* Rust
* Axum
* Askama
* PostgreSQL
* SQLx
* Garage for S3-compatible object storage
* `aws_sdk_s3::Client` as the S3 client
* Docker / docker-compose
* server-rendered HTML
* HTML forms
* minimal JavaScript
* `tracing` / `tracing-subscriber`

Do not introduce React/Vue/Next.js/etc.

The public/admin/owner UIs should remain server rendered with Askama.

Small amounts of JavaScript are allowed where HTML/CSS cannot reasonably provide the required UX.

Do not build a SPA.

---

# 3. FIRST ACTION: INSPECT THE REPOSITORY

Before making significant changes, inspect:

* complete directory structure
* Cargo.toml
* modules
* Axum routers
* application state
* database access
* migrations
* Askama templates
* existing CSS
* authentication
* session store
* Dockerfile
* docker-compose.yml
* Garage configuration
* environment configuration
* image-upload implementation
* error handling
* existing slugify implementation
* existing admin pages
* existing public routes/pages

Do not assume helper functions, configuration fields, storage abstractions, or types exist.

For example, the storage client is currently known to be:

```rust
aws_sdk_s3::Client
```

Do NOT invent APIs such as:

```rust
state.storage.public_url(...)
```

unless you actually create such an abstraction.

Prefer existing project conventions.

---

# 4. CODE ARCHITECTURE

Organize the backend into clear modules.

Do not put everything into one or two gigantic files.

A reasonable target structure is something similar to:

```text
src/
├── main.rs
├── app.rs
├── state.rs
├── config.rs
│
├── auth/
│   ├── mod.rs
│   ├── error.rs
│   ├── session.rs
│   └── middleware.rs
│
├── admin/
│   ├── mod.rs
│   ├── error.rs
│   ├── dashboard.rs
│   ├── owners.rs
│   ├── projects.rs
│   ├── properties.rs
│   └── users.rs
│
├── owners/
├── projects/
├── properties/
├── locations/
├── media/
├── public/
│
├── queries/
│
└── ...
```

This is only a guideline.

Use the existing project's style if it is already better.

Domain logic should not be mixed unnecessarily with HTTP rendering.

Database queries should be kept in a separate queries/repository layer where that fits the existing code.

Handlers should orchestrate:

```text
HTTP input
→ validation
→ domain/service operation
→ response
```

not contain giant blocks of business logic.

---

# 5. ERROR HANDLING

Do NOT use `anyhow` for request/domain errors.

Use meaningful application-specific errors.

Handlers should be able to do:

```rust
Result<Response, SomeHandlerError>
```

where the error implements:

```rust
IntoResponse
```

Each feature/module can have its own error type.

Do not expose internal SQL/storage errors directly to the browser.

Log internal failures appropriately with `tracing`.

User-facing errors should be concise and useful.

---

# 6. FORMS AND HTTP

Normal forms should use:

```http
application/x-www-form-urlencoded
```

and Axum's `Form<T>` extractor.

Uploads should use:

```http
multipart/form-data
```

Do not use JavaScript for normal CRUD form submission.

Use the PRG pattern:

```text
GET form
POST form
    ↓
success → redirect
failure → render same template with values + error
```

Use GET for filtering/search.

Use POST for ordinary form mutations.

For DELETE operations in admin tables, minimal JavaScript is acceptable:

```javascript
fetch(url, { method: "DELETE" })
```

Do not use GET for destructive operations.

After successful AJAX deletion, update the UI or navigate appropriately.

---

# 7. AUTHENTICATION — DECISION

The platform currently uses a server-side session store.

KEEP SESSION AUTHENTICATION.

Do not convert the entire application to JWT merely because project-owner authentication is being added.

Session authentication is appropriate for these server-rendered applications.

However, improve the abstraction so the in-memory implementation can later be replaced by Redis/database-backed sessions without rewriting the handlers.

Create an abstraction roughly equivalent to:

```text
SessionStore trait
    create
    get
    delete
    delete_all_for_user
    renew
```

The current backend can remain an in-memory implementation.

Keep the implementation simple.

The session store should eventually support:

* admin users
* project owners
* possibly impersonation/acting-as-owner context

The current session contains:

```text
user_id
username
started_at
expires_at
```

Extend it as necessary.

---

# 8. SESSION CLEANUP

Sessions currently expire after approximately six hours.

Implement proper expiration cleanup.

Prefer a background task that periodically removes expired sessions.

Do not let expired sessions accumulate forever.

Keep the cleanup logic inside the session abstraction where possible.

Do not make session persistence over-engineered at this stage.

Later, the same abstraction can be backed by Redis.

---

# 9. AUTHORIZATION MODEL

There are currently two important authenticated actors:

```text
Super Admin
Project Owner
```

Super admins can access everything.

Project owners may only access resources belonging to them.

Authorization MUST be enforced on the backend.

Never depend only on hiding links in the UI.

For example:

```text
/admin/projects/42/edit
```

must verify the requester's authorization.

Likewise:

```text
/owner/projects/42
```

must verify that project 42 belongs to the authenticated owner.

Admins may access the owner application as an administrator.

Do not treat a URL containing an owner ID as proof that the authenticated owner owns it.

---

# 10. ADMIN SEEDING

Create a database seeding mechanism for the initial super-admin.

Environment variables should provide:

```text
ADMIN_USERNAME
ADMIN_PASSWORD
```

The password MUST be hashed securely before storage.

Never store plaintext passwords.

Use Argon2id or an equivalent strong password-hashing implementation already compatible with the project.

The seed command should be safe to rerun.

Recommended behavior:

* create the admin profile/admin relationship if it does not exist
* if it already exists, do not unexpectedly overwrite the password on every deployment
* report clearly what happened

---

# 11. PROFILE MODEL

`profiles` represents login credentials.

Super admins are linked through:

```text
admin_users.profile_id
```

Project owners should ALSO have a profile.

A project owner should therefore look conceptually like:

```text
profiles
    ↓
project_owners
```

with a unique relationship.

Creating a project owner should create the associated profile within the same database transaction.

The profile should not be independently deletable while the owner exists.

Deleting an owner should explicitly remove its associated profile in a transaction after enforcing the required constraints.

Do not rely on users manually deleting arbitrary profile rows.

---

# 12. PROJECT OWNER MODEL

A project owner is both:

1. a public-facing entity
2. an authenticated actor

Project owners need:

```text
name
bio/info
profile image
username
active
website
contact information
```

Add reasonable contact fields rather than storing an arbitrary JSON blob.

For example:

```text
email
phone
website
```

Do not add dozens of speculative fields.

The owner has:

```text
active = false
```

by default.

If inactive:

* the owner cannot log in
* public profile can still exist if the product rules allow it
* existing projects are not automatically deleted

The super-admin can enable/disable owner login.

---

# 13. PROJECT OWNER LOGIN

Project owners need to be able to log in eventually.

Use the same session infrastructure as admins.

Do NOT duplicate authentication systems.

The authenticated principal should distinguish:

```text
Admin
ProjectOwner
```

The owner dashboard should only expose their own projects.

The public profile is separate from their private dashboard.

---

# 14. ADMIN UI

Do not change the overall admin layout that already exists.

The existing layout is intentional:

```text
+-----------------------------------------------+
| hamburger | Real Estate Admin       username  |
+----------------+------------------------------+
| Sidebar        |                              |
|                | Main content                 |
| Resources      |                              |
|                |                              |
|                |                              |
+----------------+------------------------------+
```

Desktop:

* sidebar on left
* top navbar on content side
* main content scrolls independently

Mobile:

* CSS-only drawer
* hamburger button
* close button
* no sidebar squashing into a horizontal nav
* main content should remain usable

Keep the existing visual language:

```css
--bg-main: #f7efe6;
--bg-panel: #fffaf5;
--accent: #6b3e26;
--accent-dark: #4b2a1a;
--border: #e6d5c3;
--text: #3e2b21;
```

UI should be:

* compact
* small readable text
* small spacing
* clear
* responsive
* not dense
* not enormous SaaS-card spacing
* semantic HTML
* no div soup

---

# 15. ADMIN NAVIGATION DECISION

The admin sidebar should NOT expose every domain concept as a flat list.

Use this general structure:

```text
Dashboard

Project Owners
Projects
Properties

----------------

Admin Users
Logout
```

Locations should NOT be a first-class admin sidebar resource.

Location information is part of project/location management rather than an independent CRUD resource.

Amenities may later become an independent resource because they are reusable filterable entities.

On contextual detail pages, the sidebar may become contextual.

For example, project owner view:

```text
Project Owner
    Overview
    Projects
```

Project view:

```text
Project
    Overview
    Residential Properties
    Commercial Properties
    Media
    Documents
```

This nested UX is intentional and should be retained.

The contextual view should still provide an obvious way back to the global resource list.

Do NOT hide the entire application navigation with no breadcrumb/back mechanism.

---

# 16. DASHBOARD

The admin dashboard should answer:

"What is happening in this system?"

Display:

* total project owners
* total projects
* total properties
* total residential properties
* total commercial properties
* recently created projects
* available properties associated with those projects

The recent-project widget should have a sensible default time window such as 30 days.

Each recent project can show:

```text
Project name
Created date
Owner
Location
Total properties
Available properties
```

Available properties means currently active/listed properties, not sold/closed ones.

The dashboard queries must be efficient.

Do not execute one SQL query per project (N+1).

Use grouped queries or joins.

Add appropriate indexes.

---

# 17. PROPERTY ARCHITECTURE — IMPORTANT DESIGN DECISION

Do NOT keep residential and commercial properties as two completely unrelated top-level tables.

The application needs a common property model because many concepts are common:

```text
project
description
floor_number
total_floors
unit_number
built_up_area
usable_area
parking
listing state
slug
media
created_at
```

Use:

```text
properties
```

as the common table.

Then use optional subtype detail tables where the fields genuinely differ.

Conceptually:

```text
properties
    |
    +---- residential_property_details
    |
    +---- commercial_property_details
    |
    +---- land_property_details
```

This gives us:

* easy global property counting
* easy common searching
* easy common media
* easy common listing handling
* clear subtype-specific fields
* strong relational integrity
* ability to add future categories

Do not put every possible property-specific field into one enormous nullable table.

---

# 18. PROPERTY CATEGORIES

Do NOT encode every property subtype in a PostgreSQL ENUM.

Property types will evolve.

Use database-driven classifications.

Use a table similar to:

```text
property_types

id
category
name
slug
parent_id
is_active
```

where category may be something like:

```text
RESIDENTIAL
COMMERCIAL
LAND
```

Examples:

Residential:

```text
Bungalow
Villa
Tenement
Row House
Penthouse
Flat
```

Commercial:

```text
Office
Showroom
Shop
Warehouse
Factory
```

Land:

```text
Residential Plot
Commercial Plot
Agricultural Land
Building-approved Land
Industrial Land
```

Important modeling decisions:

### Duplex

Do NOT make Duplex a property type.

A duplex is a configuration/property characteristic.

It belongs in residential details:

```text
is_duplex
```

### Farmhouse

Treat farmhouse as a residential property type.

### Villa

Treat villa as a residential property type.

### Plot

Treat plot as land, not automatically residential.

A residential plot and commercial plot can therefore use the same land-oriented model while having different `property_type` records.

This classification should be extensible without a database migration every time a new real-estate category is introduced.

---

# 19. RESIDENTIAL PROPERTY DETAILS

Residential details may contain:

```text
bedroom_count
bathroom_count
balcony_count
is_duplex
parking information where justified
```

Do not turn every feature into dozens of columns.

Use explicit columns for commonly queried/filterable attributes.

Use a separate amenities/features relationship for reusable features such as:

```text
Parking
Garden
Swimming Pool
Gym
Clubhouse
Security
```

Do not put these into comma-separated strings.

---

# 20. COMMERCIAL PROPERTY DETAILS

Commercial detail fields should be different from residential fields.

Property type distinguishes:

```text
Office
Shop
Showroom
Warehouse
Factory
```

Do not add bedroom count.

---

# 21. LAND PROPERTY DETAILS

Create a separate land detail model where needed.

Possible fields:

```text
parcel/reference number
zoning
approval status
development status
```

Only add fields that have a clear product use.

Do not turn this into a huge GIS system now.

---

# 22. PROPERTY AREA

All properties should use one canonical measurement internally.

Use:

```text
square feet
```

for the initial implementation.

Store as a numeric/decimal type appropriate for fractional values.

Example:

```text
area_sq_ft NUMERIC(...)
```

The UI may eventually display other units, but the database uses one canonical unit.

This keeps searching, filtering, sorting and comparison simple.

---

# 23. PROPERTY LISTING / PRICE MODEL

Do NOT represent price with:

```text
rent_or_lease_duration
status
one price column
```

inside the property itself.

The property may have different commercial states over time.

Use a listing/offer model:

```text
property_listings
```

Conceptually:

```text
id
property_id
listing_type       -- sale / rent / lease
status             -- active / closed / sold / withdrawn
currency_code
price
billing_period
created_at
updated_at
```

This gives us a better foundation for:

* sale
* rent
* lease
* future price history
* availability
* currency

Currency should initially be a three-letter ISO-style code such as:

```text
INR
USD
EUR
```

Store the numeric price separately.

Do not implement full multi-currency conversion now.

Make the schema compatible with it later.

---

# 24. AVAILABLE PROPERTIES

Do not make "available" another hardcoded property type.

Availability should be derived from listing state.

For example:

```text
active listing → available
sold/closed/withdrawn → not available
```

This makes dashboard queries and public filtering much cleaner.

---

# 25. PROPERTY STATUS

Do not use one property-level enum like:

```text
ForSale
ForLease
ForRent
Sold
```

as the entire model.

Use listings as described above.

This allows the product to evolve without constantly adding statuses.

---

# 26. LOCATIONS — DESIGN DECISION

Do NOT create a large hierarchy of manually maintained:

```text
countries
states
cities
areas
```

tables at this stage.

That creates significant administrative maintenance and becomes problematic when integrating Google Places/Maps or OpenStreetMap.

Instead create a reusable `locations` entity containing structured location information.

Conceptually:

```text
locations

id
country_code
country_name
state_or_province
city
locality
area
postal_code
formatted_address
latitude
longitude
provider
provider_place_id
created_at
updated_at
```

Not every field must be populated.

Projects reference a location.

This gives us:

* normalized reuse
* easy search
* future Google/OSM integration
* coordinates
* external provider identity
* good public filtering

Do NOT force external API integration now.

Design the schema so it can be added later.

Use the application's own locations table as the canonical stored snapshot.

Do not require a live Google API call just to render a project.

---

# 27. PROJECT MODEL

A project belongs to:

```text
project owner
location
```

A project should have:

```text
name
slug
description/info
owner
location
project category
start date
launch date
possession date
created_at
updated_at
deleted_at
```

Add fields that are clearly required by the product.

Project category should support at least:

```text
Residential
Commercial
Mixed
```

A mixed project is useful because a development can contain both residential and commercial properties.

Do not force every project into exactly one property category.

---

# 28. PROJECT AMENITIES

Amenities should be normalized.

Use:

```text
amenities
```

and a many-to-many relationship:

```text
project_amenities
```

Amenities should be reusable and searchable/filterable.

Superadmins should be able to:

* create
* update
* delete
* list
* search

amenities.

Deleting an amenity must not leave broken relationships.

Use appropriate foreign keys and cascade semantics.

Amenities should be available when editing projects.

---

# 29. PROJECT DOCUMENTS

Projects may have documents such as:

```text
RERA certificate
BU approval
brochure
other project documents
```

Do not store documents directly as arbitrary database blobs.

Create a media/document abstraction.

Store object keys in Garage and metadata in PostgreSQL.

Documents should have:

```text
display name
document type
media id
project id
created_at
```

Use appropriate MIME type and download behavior.

---

# 30. MEDIA ARCHITECTURE — IMPORTANT

Replace the current concept of `images` as a purely image-specific entity with a generic `media` concept.

The platform will eventually store:

```text
images
videos
documents
```

Do NOT create separate S3 logic for every feature.

Use a common `media` table containing metadata such as:

```text
id
media_type
mime_type
s3_key
thumbnail_key
file_size
width
height
duration
created_at
```

Only fields relevant to a particular type need to be populated.

Do not use `owner_type + owner_id` polymorphic foreign keys for domain relationships when explicit junction tables are cheap and provide proper database integrity.

Prefer:

```text
project_media
property_media
project_owner_profile_media
project_documents
```

etc.

For a project:

```text
project_media
    project_id
    media_id
    sequence
```

For a property:

```text
property_media
    property_id
    media_id
    sequence
```

For the owner profile image:

a one-to-one relation is sufficient.

---

# 31. MEDIA OBJECT KEYS

For owner profile images:

```text
owners/{owner_id}/original.webp
owners/{owner_id}/thumbnail.webp
```

This is intentionally deterministic because an owner has one profile image.

For project/property media, multiple items require unique media identities.

Prefer:

```text
projects/{project_id}/{media_id}/original.webp
projects/{project_id}/{media_id}/thumbnail.webp
```

and similarly:

```text
properties/{property_id}/{media_id}/original.webp
properties/{property_id}/{media_id}/thumbnail.webp
```

Do not use the display order as the unique object identity.

Sequence belongs in the database.

Do not use filenames supplied by users as S3 keys.

---

# 32. PROJECT MEDIA

A project can have up to 10 images/videos in the initial implementation.

The edit page should have an Image/Media tab.

Show a responsive grid.

Each item should show:

* thumbnail
* type
* sequence
* delete/remove control

The UI should allow adding more items with an Add button.

Do not force all ten uploads to be selected at once.

It is acceptable to use a small amount of JavaScript for this feature.

Petite-Vue may be used if the plain JavaScript becomes significantly more complicated, but do not introduce it unless necessary.

---

# 33. PROJECT MEDIA UPLOAD API

The upload endpoint may be called using JavaScript.

The backend must:

* validate file
* detect actual content type
* process image/video
* create media record
* upload objects
* associate media with project
* preserve ordering
* return structured success/error information

The admin UI should identify individual failures.

Example:

```text
✓ image1.jpg uploaded
✓ image2.jpg uploaded
✕ video1.mp4 failed: file too large
```

A single failed media item must not silently make all successful items appear failed.

---

# 34. IMAGE PROCESSING

Continue using the existing image crate/image-processing implementation where appropriate.

For images:

* validate/decode
* convert to WebP
* create thumbnail
* upload processed assets
* do not trust user-provided file extension
* enforce upload size limits

Owner thumbnails can be around 300x300.

Project/property thumbnails can be somewhat larger because they will be displayed in larger grids.

Make dimensions configurable if appropriate.

---

# 35. VIDEO PROCESSING

For videos, use FFmpeg through an appropriate Rust integration already intended by the project, such as `ez-ffmpeg`, but inspect the crate/API before implementation.

The video pipeline should:

* validate upload
* reject files larger than configured max size
* default max size to 20 MB
* make the size configurable through environment
* transcode using configurable codec settings
* use hardware decoding/encoding when available
* preserve resolution
* preserve audio
* preserve subtitles where the output format permits it
* generate a thumbnail/poster image

IMPORTANT:

Do not assume MKV is suitable for browser playback.

For browser-facing video, prefer a web-compatible container such as MP4 unless the repository has a justified reason otherwise.

The video codec should still be configurable.

The initial environment can specify H.265/HEVC, but the application should make browser compatibility considerations explicit.

---

# 36. PROPERTY MEDIA

Properties may have up to 20 images/videos.

Their edit forms should have the same general media-management model as projects.

When a property is deleted:

* delete property-media relationships
* remove associated Garage objects
* remove media records
* perform cleanup safely

Do not leave orphaned S3 objects indefinitely.

---

# 37. IMAGE PREVIEW IN FORMS

For simple single-image upload fields such as project-owner profile image:

Use:

* hidden native file input
* label/drop zone covering the input
* CSS styling
* minimal JS only for preview after selection

When editing an existing owner:

* show current thumbnail
* show replacement upload control
* provide Remove Image action

No JS is needed merely to make the browser accept dropped files if the input itself covers the drop area.

---

# 38. PROJECT OWNER UI

Global owner listing:

```text
Project Owners

[ Search.............. ] [Apply]       [New Owner]

--------------------------------------------------
Picture | Name | Projects | Actions
--------------------------------------------------
...
```

Owner name must be a hyperlink to the owner detail page.

Actions:

```text
Edit
Delete
```

Delete should confirm and use DELETE via minimal JS.

The list should show:

* thumbnail
* name
* project count
* edit
* delete

Do not show enormous cards.

---

# 39. PROJECT OWNER DETAIL PAGE

The owner detail page should show:

* profile image
* name
* username
* bio
* contact information
* website
* active/login status
* project count
* timestamps where useful

The contextual sidebar should contain:

```text
Owner Overview
Projects
```

Projects should navigate to:

```text
/admin/owners/{owner_id}/projects
```

This URL structure is intentional.

Do not rely on client-side state.

---

# 40. PROJECT LISTING

A project list should show approximately:

```text
Project
Owner
Location
Category
Residential properties
Commercial properties
Created
Actions
```

Project name links to project detail.

Search should support useful resource-specific queries.

Do not query every associated property independently for every row.

Use efficient grouped SQL.

---

# 41. PROJECT DETAIL PAGE

Project detail page should display:

* project title
* owner
* location
* description
* project category
* dates
* amenities
* images/videos
* documents
* property counts
* relevant metadata

The contextual sidebar should expose:

```text
Overview

Residential Properties
Commercial Properties

Media
Documents
```

Potentially:

```text
Amenities
```

if useful.

---

# 42. PROPERTIES UI

The property interface is contextual to projects.

When viewing a project:

```text
Residential Properties
Commercial Properties
```

are separate navigation entries.

Example:

```text
/admin/projects/{project_id}/properties/residential
/admin/projects/{project_id}/properties/commercial
```

This is intentionally project-scoped.

The project ID should come from the URL, and the backend should derive/filter all data from it.

Do NOT trust hidden form fields for authorization or ownership.

---

# 43. PROPERTY LIST

Residential list columns should include useful values such as:

```text
House
Building
BHK
Area
Type
Listing price
Status
Actions
```

Commercial:

```text
Unit
Building
Type
Area
Listing price
Status
Actions
```

Search should be resource specific.

Filters should include appropriate fields such as:

* property type
* price range
* listing type
* availability/status
* BHK for residential
* city/location where applicable

Use SQL indexes designed around actual filters.

---

# 44. PROPERTY FORM

Property creation automatically uses the currently opened project.

Example:

```text
/admin/projects/{project_id}/properties/residential/new
```

Do not require the administrator to select a project manually when the project context is already encoded in the URL.

Property forms should use normal URL-encoded forms.

Use tabs for:

```text
Details
Media
```

where appropriate.

---

# 45. PROPERTY FEATURES

Do not put every possible room/feature into a giant text field.

For commonly filtered numeric attributes, use dedicated columns:

```text
bedroom_count
bathroom_count
balcony_count
floor
```

Use normalized amenities/features for reusable categorical attributes.

This allows queries such as:

```text
3 BHK
2 bathrooms
balcony
parking
```

without parsing text.

---

# 46. SLUGS

There is an existing `slugify` implementation in the repository.

USE IT.

Do not replace it unnecessarily.

Slugs should be:

* generated server-side
* normalized
* case-insensitive for lookup
* canonical
* stable enough for SEO
* automatically regenerated when the fields defining the slug materially change

Project slug should include enough identity to remain unique.

A good canonical URL is:

```text
/projects/{slug}
```

where the slug can include the ID suffix.

Do NOT duplicate:

```text
project/
```

inside a slug field if the router already provides `/project/`.

Store the resource-specific slug in the database.

If a non-canonical slug is requested:

```text
/projects/old-name-123
```

and the current canonical slug is:

```text
/projects/new-name-123
```

respond with:

```http
301 Moved Permanently
Location: /projects/new-name-123
```

The same concept applies to project owners and other public entities using slugs.

---

# 47. PUBLIC URL DESIGN

Use semantic resource paths.

Examples:

```text
/projects/{slug}
/owners/{slug}
/properties/{slug}
```

Do not expose internal numeric IDs as the only public identifier.

IDs may be included in slugs when useful for uniqueness.

Internal admin routes may use IDs because they are not public SEO URLs.

---

# 48. PUBLIC WEBSITE

The public-facing application should be server rendered with Askama.

Do not build a SPA.

Pages include:

```text
Home
Projects listing
Project details
Project owner details
Property listing/search
Property details
```

The visual style should feel like a modern, polished real-estate website rather than an admin dashboard.

Use:

* strong typography
* good imagery
* clear property cards
* responsive layouts
* tasteful animation
* carousels only where useful
* minimal JavaScript
* good mobile behavior

Do not overuse animation.

Do not sacrifice performance for visual effects.

---

# 49. PUBLIC SEARCH/FILTERING

Property search should eventually support:

```text
location
city
property category
property type
budget
listing type
BHK
area
availability
```

Only expose filters relevant to the property category.

Residential example:

```text
City
Budget
BHK
Property type
Area
```

Commercial:

```text
City
Budget
Property type
Area
```

Land:

```text
City
Budget
Land type
Area
```

Do not execute application-side filtering over large datasets.

Build SQL queries using indexed database fields.

---

# 50. PUBLIC PAGE GENERATION / CACHING

For now, KEEP this simple.

Do not build an elaborate distributed rendering architecture.

Implement the public pages normally with Askama first.

Introduce a page rendering/cache abstraction so generated HTML can later be cached.

The initial in-process cache can have:

```text
capacity = environment value
default = 50
```

Use an appropriate TinyLFU/LFU-like in-memory cache if available.

Do not claim an exact LFU policy if the chosen library uses a related policy.

Later, this can move to Redis/shared storage.

---

# 51. STATIC GENERATED HTML

If/when filesystem HTML generation is implemented:

* keep generated HTML under a dedicated runtime directory
* do not mix generated content with source templates
* do not commit generated HTML
* serve it efficiently
* invalidate/regenerate it on content changes

Do not make filesystem generation a prerequisite for the admin panel.

---

# 52. PUBLIC PAGE INVALIDATION

Whenever any public entity changes, the relevant page cache must be invalidated.

Examples:

Project update:

```text
project page
project owner's project listing
relevant sitemap entries
```

Property update:

```text
property page
project property listing
possibly project summary/cache
```

Owner update:

```text
owner page
project references
sitemap
```

Build this through a small domain event/invalidation mechanism rather than scattering cache invalidation calls everywhere.

---

# 53. PUBLIC GENERATION QUEUE

Do not let expensive public HTML generation block admin/owner form submissions.

However, this is NOT the first thing to implement.

First build a clean synchronous rendering system.

Then add:

```text
page_build_jobs
```

and a background worker if generation becomes expensive.

The worker can initially run inside the same application process.

One job should identify the resource/page to regenerate, NOT store the entire HTML snapshot in the queue.

The worker should reload the current database state before rendering.

Coalesce multiple pending jobs for the same resource so repeated updates do not create unnecessary duplicate work.

---

# 54. SEO

Public pages must support:

* title
* description
* canonical URL
* OpenGraph metadata
* structured data where appropriate
* robots directives
* sitemap
* robots.txt

Sitemap must include active public resources.

Do not include deleted/internal/private resources.

Use canonical URLs.

Do not leak:

* S3 keys
* database IDs unless intentionally part of URLs
* admin routes
* internal APIs

---

# 55. SITEMAP

Create a clean sitemap generator.

It should include:

```text
homepage
project URLs
project owner URLs
property URLs
```

only where publicly visible.

When content changes, mark sitemap/page cache as dirty.

The first implementation can regenerate synchronously or through the background job system depending on size.

Do not create one giant sitemap if the number of URLs later becomes large.

Design the generator so sitemap indexes can be added later.

---

# 56. PUBLIC CONTACT/LEAD FORMS

Projects and properties need public enquiry forms.

Fields:

```text
name        required
phone       required
email       optional
message     max 255
```

Store submissions in database tables.

Each submission should reference the associated:

```text
project
```

or:

```text
property
```

Use separate explicit foreign keys or tables rather than polymorphic IDs.

The submission must have:

```text
created_at
read_at or is_read
```

and should be visible to:

* all superadmins
* the project owner responsible for the project/property

---

# 57. NOTIFICATIONS

Admin and owner dashboards should eventually have a notification icon.

New enquiries should create notifications.

Admins can see all.

Project owners only see notifications associated with their projects/properties.

Do not use a JSON blob containing arbitrary references.

Use proper relational references.

A notification system can later support:

* push notifications
* email
* web push
* PWA

Do not implement push notifications yet unless the repository already has infrastructure for it.

---

# 58. PROJECT OWNER DASHBOARD

After superadmin is stable, build the owner application.

The owner dashboard should show:

* total projects
* recent projects
* residential property counts
* commercial property counts
* available properties
* recent enquiries/notifications

Project owners can only operate on their own projects.

The owner application should use contextual navigation:

```text
Dashboard

Projects
```

Inside a project:

```text
Overview
Residential Properties
Commercial Properties
Media
Documents
```

This hierarchy is good.

It matches the domain:

```text
Owner
  → Projects
      → Properties
      → Media
      → Documents
```

Do NOT make the owner repeatedly select themselves or their project through dropdowns when the URL context can identify it.

---

# 59. SUPERADMIN VERSUS OWNER ACCESS

Superadmin:

```text
all owners
all projects
all properties
all users
all amenities
all notifications
```

Project owner:

```text
own profile
own projects
properties under own projects
media/documents under own projects
enquiries associated with own projects/properties
```

Project owner MUST NOT:

```text
view other owners
edit other owners
view unrelated projects
edit unrelated projects
```

Backend authorization must enforce this.

---

# 60. GARAGE STORAGE

Garage is S3-compatible.

The application uses:

```rust
aws_sdk_s3::Client
```

The S3 API is for backend access.

The browser should NOT use the S3 API directly.

Garage has separate web access for browser-visible assets.

Current local configuration:

```toml
[s3_web]
bind_addr = "[::]:3903"
root_domain = ".web.docker.localhost"
```

A bucket called:

```text
myplace
```

will be addressable through a hostname like:

```text
http://myplace.web.docker.localhost:3903
```

for local development, assuming the bucket's web access is configured.

Do NOT construct browser URLs from the S3 API endpoint.

Do NOT hardcode:

```text
http://localhost:3900
```

into templates.

Use configuration for the browser-facing asset base URL.

For local development it can be configured as:

```text
http://myplace.web.docker.localhost:3903
```

Production can later become:

```text
https://assets.example.com
```

The database should store S3 object keys, not complete environment-specific URLs.

---

# 61. DOCKER

Inspect the existing docker-compose setup.

Remember:

```text
localhost inside a container
```

is not:

```text
localhost on the host
```

Use service names for container-to-container connections.

Use host ports/hostnames only for browser-facing URLs.

Do not rebuild images unnecessarily when only port mappings/configuration change.

Make local Garage web access actually usable from the browser.

---

# 62. DATABASE INDEXING

When implementing queries, think about actual access patterns.

Add indexes for:

* foreign keys used for filtering
* project owner → projects
* project → properties
* project → media
* property → media
* active/listing status
* location filtering
* property type filtering
* created_at ordering
* slug lookup

Do NOT add indexes blindly to every column.

A primary key already has an index.

Review existing indexes for redundancy.

For example:

```sql
CREATE INDEX idx_project_owner_on_projects ON projects(project_owner_id);
```

is useful.

An index on `projects(id)` would normally be redundant because `id` is already the primary key.

---

# 63. FULL TEXT SEARCH

The current schema has PostgreSQL `tsvector` fields for property search.

Keep PostgreSQL full-text search where useful.

Do not immediately introduce Elasticsearch/OpenSearch.

Use FTS for text-heavy property/project search and normal B-tree indexes for structured filters.

If search requirements become significantly more complex later, the architecture can evolve.

---

# 64. DATA MODELING PRINCIPLE

Prefer normalized, explicit relational data when:

* it is queried
* it is filtered
* it is joined
* it has integrity requirements

Use JSON only when the data is genuinely flexible/unstructured and not central to querying.

Do not use:

```text
comma,separated,values
```

for relationships.

Do not use a giant JSON blob to avoid designing a schema.

---

# 65. SOFT DELETE

Projects and owners have `deleted_at`.

Use soft deletion where content must remain historically recoverable or where dependent records should not disappear immediately.

A soft-deleted owner should not appear in normal listings.

A soft-deleted project should not appear publicly.

Do not automatically cascade soft deletes using database `ON DELETE`.

Application logic should decide whether related content is archived, hidden or actually deleted.

Physical deletion should be explicit and cautious.

---

# 66. TRANSACTIONS

Operations involving several database records should use SQL transactions.

Examples:

Create project owner:

```text
create profile
create owner
commit
```

Delete project owner:

```text
validate deletion
delete relationships
delete owner/profile as designed
commit
```

Add media:

```text
create media DB record
create relationship
```

Storage operations are not transactional with PostgreSQL, so implement sensible compensation/cleanup logic.

Never leave a database pointing to a definitely nonexistent object if avoidable.

Periodically consider orphan-media cleanup.

---

# 67. MEDIA CLEANUP

Whenever media is replaced/deleted:

* identify all affected Garage keys
* delete database relations
* delete Garage objects
* remove DB media metadata when no longer referenced

Do not accidentally delete an object still referenced by another relationship.

Use database constraints and reference counts/queries to verify this before deletion.

---

# 68. UI RESOURCE LIST PATTERN

Resource listings should share a visual pattern:

```text
Page header
    title
    search/filter form
    primary create action

table/list

empty state
```

Actions:

```text
Edit
Delete
```

should look like a coherent action group, not random browser-default links/buttons.

Delete is visually destructive.

Do not use giant buttons.

---

# 69. PROJECT OWNER FORM

The project-owner form uses pure HTML/CSS tabs.

Editing:

```text
[ Details ] [ Image ]
```

Creating:

```text
Details only
```

Details:

```text
name
username
bio
website
email
phone
active
password/initial credential where appropriate
```

Image:

```text
current preview
replace/upload
remove
```

Keep the image form separate from the normal URL-encoded form because the image requires multipart.

This separation is intentional.

---

# 70. MARKDOWN

Project and other longer-form descriptive content should be stored as Markdown.

Use a lightweight browser Markdown editor rather than a full SPA.

A CDN-based editor is acceptable.

The backend should store markdown source, not generated HTML.

When rendering public pages:

```text
Markdown
→ sanitized HTML
→ Askama template
```

Never inject untrusted raw HTML.

Sanitize the generated HTML.

---

# 71. PUBLIC PAGE CONTENT SECURITY

Anything authored by admins/project owners and rendered as HTML must be treated as untrusted content.

Sanitize Markdown output.

Do not allow arbitrary:

```html
<script>
```

or dangerous HTML.

---

# 72. LOGGING / OBSERVABILITY

Continue using `tracing`.

Add structured logging around:

* authentication
* authorization failures
* database errors
* media processing
* Garage operations
* background jobs
* page generation

Do not log:

* passwords
* session IDs
* private contact details unnecessarily
* credentials
* storage secrets

OpenTelemetry integration can be added cleanly later.

Do not make it a prerequisite for the current feature set.

---

# 73. TESTING

Add tests around important domain behavior.

At minimum, test:

* slug generation
* authorization scope
* owner/project ownership
* property filtering
* listing availability
* admin seed behavior where practical
* media deletion safety
* session expiration
* route-level authorization

Do not test trivial HTML strings excessively.

---

# 74. IMPLEMENTATION ORDER

Do NOT attempt all features simultaneously.

Use this sequence.

## Phase 1 — Foundation

Inspect and stabilize:

* configuration
* Docker
* database migrations
* auth
* sessions
* error handling
* module structure
* storage integration

Fix clear architectural problems.

---

## Phase 2 — Database/domain model

Implement the revised schema for:

```text
profiles
admin_users
project_owners
locations
projects
amenities
properties
residential details
commercial details
land details
property listings
media
project media
property media
documents
```

Do not add speculative future tables yet.

Create migrations.

Do not modify old migrations that may already be applied.

---

## Phase 3 — Admin authentication

Implement:

```text
/auth/admin-login
```

Successful login redirects to:

```text
/admin
```

Build seeding.

Build logout.

Build admin session protection.

---

## Phase 4 — Admin dashboard

Implement the dashboard with efficient aggregate queries.

---

## Phase 5 — Project owners

Implement:

* listing
* search
* create
* edit
* delete
* owner detail
* profile creation
* active login flag
* image upload
* image replacement
* image removal

---

## Phase 6 — Projects

Implement:

* list
* search/filter
* create
* edit
* delete/soft delete
* owner
* location
* dates
* Markdown
* amenities
* project detail page
* documents
* media

---

## Phase 7 — Properties

Implement the unified property model and subtype details.

Implement:

* residential
* commercial
* land
* listing prices
* availability
* filters
* CRUD
* media

---

## Phase 8 — Owner application

Implement owner login and scoped dashboard.

---

## Phase 9 — Public website

Implement:

* home
* projects
* owners
* property listings
* property details
* SEO
* sitemap
* contact forms
* notifications foundation
* asset delivery

---

# 75. DEVELOPMENT BEHAVIOR

While implementing:

1. Read the existing file before editing it.
2. Prefer small migrations.
3. Prefer explicit code over clever generic abstractions.
4. Run `cargo fmt`.
5. Run `cargo check`.
6. Run tests.
7. Fix compiler/template/SQL errors before proceeding.
8. Verify routes against actual router registration.
9. Verify template variables against actual Rust structs.
10. Verify SQL against the actual migration schema.

Do not leave invented placeholder code pretending to work.

If a feature is intentionally deferred, mark it clearly with a TODO and do not build fake functionality.

---

# 76. DO NOT DO THESE THINGS

Do NOT:

* convert the application to React
* build a SPA
* introduce GraphQL
* introduce Elasticsearch unnecessarily
* introduce microservices
* replace Askama
* replace Axum
* replace SQLx
* replace PostgreSQL
* replace Garage
* build a generic CRUD framework
* introduce ten layers of repository abstractions
* introduce polymorphic foreign keys merely for "flexibility"
* store relational data as JSON
* put S3 URLs into the database
* trust client-side authorization
* use GET for deletion
* store passwords in plaintext
* silently change business semantics
* make large schema changes without migrations
* hardcode localhost URLs into templates
* use the AWS S3 public URL format for Garage assets
* make the frontend responsible for important business logic

---

# 77. DECISION SUMMARY

Use these decisions unless repository constraints prove otherwise:

### Authentication

```text
server sessions
shared auth abstraction
in-memory initially
Redis-capable later
```

### Admin + owner credentials

```text
shared profiles table
admin_users relation
project_owners relation
```

### Properties

```text
properties
    ↓
residential_property_details
commercial_property_details
land_property_details
```

### Property classification

Database-driven hierarchical property types.

Do NOT use a PostgreSQL enum for every type.

### Property listings

Separate listing/offer model for:

```text
sale
rent
lease
```

with price/currency/status.

### Locations

One structured location entity with optional external-provider metadata.

Do not build country/state/city master CRUD yet.

### Media

Generic media entity.

Explicit relationship tables.

### Owner image

Deterministic storage:

```text
owners/{owner_id}/original.webp
owners/{owner_id}/thumbnail.webp
```

### Project/property media

Media-ID based object paths:

```text
projects/{project_id}/{media_id}/...
properties/{property_id}/{media_id}/...
```

### Project hierarchy

```text
Project Owner
    ↓
Projects
    ↓
Properties
```

Use contextual navigation.

### Public assets

Database stores object keys.

Browser URLs are generated from configurable asset base URL.

### Public site

Askama + server rendering + cache.

No SPA.

---

# 78. SUCCESS CRITERIA

The application should eventually feel like one coherent product.

A superadmin should be able to:

```text
seed account
    ↓
login
    ↓
dashboard
    ↓
create owner
    ↓
create owner profile
    ↓
upload owner image
    ↓
create project
    ↓
assign owner
    ↓
assign location
    ↓
add amenities
    ↓
upload project media/documents
    ↓
create residential/commercial/land properties
    ↓
add listing information
    ↓
manage property media
    ↓
view everything through clean contextual pages
```

A project owner should be able to:

```text
login
    ↓
see own dashboard
    ↓
see own projects
    ↓
manage own projects
    ↓
manage properties/media/documents
    ↓
see own enquiries/notifications
```

A visitor should eventually be able to:

```text
home
    ↓
search/filter
    ↓
project
    ↓
properties
    ↓
property details
    ↓
contact/enquiry
```

The same underlying domain model should support all three experiences.

Do not implement three unrelated systems.

---

# FINAL OPERATING INSTRUCTION

Take ownership of the implementation.

First inspect the repository thoroughly.

Then produce a concise architecture assessment and implementation plan.

After that, start implementing the highest-priority phase rather than waiting for further prompting.

Make sensible decisions yourself.

When changing the schema, use migrations.

When changing authentication, preserve backward compatibility where practical.

When implementing UI, preserve the existing layout and visual language.

When implementing storage, use the actual Garage/S3 client and configuration present in the repository.

When something is uncertain, inspect the code first.

Only ask me when the repository cannot answer the question and the decision would materially affect the architecture.

The goal is to turn this existing unfinished codebase into a cohesive, production-quality foundation for a real-estate platform, not merely to produce mock pages.

