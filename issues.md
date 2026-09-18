### 1.
Audit the current database/query organization against the intended architecture.

The application is a Rust + Axum + SQLx + PostgreSQL real-estate application. Database queries should not be scattered across route handlers, unrelated modules, or a single generic query file.

First inspect the existing source tree and identify where SQLx queries currently live. Do not blindly move files before understanding module dependencies.

Refactor the query layer into clear domain-oriented modules. At minimum, separate queries for:

- admin users/authentication
- project owners
- locations
- projects
- properties
- property types/details
- media
- sitemap/public pages where applicable

Keep HTTP handlers focused on request extraction, authorization, calling services, and rendering/redirecting. SQL/database access should be isolated in the query/repository layer.

Do not introduce unnecessary abstraction layers or generic repository traits just for the sake of abstraction. Prefer simple typed SQLx query functions grouped by domain.

Preserve all existing behavior.

After refactoring:
1. run cargo fmt
2. run cargo check
3. run the existing test suite
4. fix all module/import errors
5. ensure no SQL queries remain unnecessarily embedded in route handlers
6. report the final query/module structure.


### 2.
Audit the current application/server architecture.

The intended deployment architecture is three independently listening HTTP servers:

1. Admin application/server
2. Project Owner application/server
3. Public website application/server

Each must listen on its own configurable port/address.

They may share internal Rust modules for domain logic, database access, storage, models, validation, media processing, etc. Do NOT duplicate the entire application codebase merely to achieve server separation.

However, each server must have its own router and explicit access-control boundary.

Required isolation:

Admin server:
- exposes only admin routes/pages/APIs
- requires admin authentication/authorization where appropriate
- must not expose owner/public administrative functionality accidentally

Owner server:
- exposes only project-owner routes/pages/APIs
- requires authenticated project-owner access
- enforces ownership authorization at the service/query layer
- an owner must never be able to access another owner's resources by changing an ID

Public server:
- exposes only public pages/public APIs/media endpoints intended for public access
- must not mount admin or owner management routes
- must not rely on frontend hiding to enforce access

Inspect the current application before modifying it. Determine whether the current implementation already has separate routers but incorrectly serves them from one listener, or whether it genuinely has one combined server.

Create separate server/bootstrap entry points while keeping shared application/domain infrastructure reusable.

Ports must be configurable through environment variables rather than hardcoded.

Add startup logging that clearly identifies which server is starting and which address it is listening on.

Add tests proving that:
- admin routes are unavailable from the public server
- owner management routes are unavailable from the public server
- public routes do not bypass admin/owner authorization
- owner authorization is scoped to the authenticated owner

Do not create three independent database implementations. The isolation required here is HTTP/application boundary isolation, not duplication of the persistence layer.

### 3.
Audit every place where absolute public URLs are generated.

The public website is served from its own server/port, while the sitemap currently generates URLs using the wrong origin (for example localhost:3000 while the actual public content is served on localhost:8080).

Do not hardcode a port as the fix.

Introduce/use an explicit configuration value representing the canonical public website origin, for example:

PUBLIC_SITE_ORIGIN=http://localhost:8080

Production must be able to use:

PUBLIC_SITE_ORIGIN=https://example.com

Use this configuration consistently for:
- sitemap URLs
- canonical URLs
- OpenGraph URLs
- structured data URLs
- public resource URLs where absolute URLs are required
- redirects that construct absolute public URLs

Do not use the admin or owner server origin when generating public URLs.

Normalize the configured origin so generated URLs do not contain accidental double slashes.

Add tests asserting that a generated sitemap URL uses PUBLIC_SITE_ORIGIN and never the admin/owner origin.

Also audit existing templates and URL helpers for the same incorrect-origin problem.

### 4.
Replace the current monolithic sitemap implementation with a partitioned sitemap architecture.

Requirements:

1. `/sitemap.xml` must be a sitemap index, not a sitemap containing every URL.

2. Sitemap entries must be partitioned by entity type.

At minimum support:
- projects
- project owners
- properties

Design the implementation so additional public entities can be added later.

3. Each entity type must be split by configurable ID ranges.

Configuration must allow different partition sizes per entity, for example:

SITEMAP_PROJECTS_RANGE=1000
SITEMAP_OWNERS_RANGE=500
SITEMAP_PROPERTIES_RANGE=2000

Use appropriate environment-variable names consistent with the existing configuration system.

4. A sitemap shard is determined by entity ID, not by current row count.

For example, with range size 1000:
- IDs 1–1000 -> shard 1
- IDs 1001–2000 -> shard 2
- IDs 2001–3000 -> shard 3

Use deterministic shard naming.

5. Generating a shard must query only records belonging to that ID range.

Never query the entire entity table merely to regenerate one shard.

6. On entity create/update/delete:
- determine the entity's ID range
- invalidate only that shard
- regenerate it asynchronously where appropriate
- if a newly created entity falls into a previously nonexistent range, create the shard
- if a shard becomes empty after deletion, remove the shard from the sitemap index
- update the sitemap index accordingly

7. Avoid regenerating unrelated entity shards.

8. Handle updates that affect URL/canonical information correctly. If an entity's public URL changes, invalidate the old and new affected sitemap shards if their entity IDs are different only where necessary; normally the same entity ID remains in the same shard.

9. Sitemap generation must use PUBLIC_SITE_ORIGIN.

10. The sitemap index itself should not require querying every entity. It should be constructed from the known/configured/registered shard set.

11. Make sitemap generation concurrency-safe so two simultaneous updates cannot corrupt the same shard.

12. Make generation idempotent.

13. Do not block an admin mutation unnecessarily while rebuilding large sitemap files. Use the existing background task/queue mechanism if one exists.

14. Add tests for:
- multiple entity types
- different configured ranges
- entity creation
- entity update
- entity deletion
- creation of a new shard
- removal of an empty shard
- correct sitemap index contents
- correct PUBLIC_SITE_ORIGIN
- ensuring unrelated shards are not regenerated

First inspect the existing sitemap implementation and reuse existing cache/background infrastructure where appropriate rather than creating a second competing system.

### 5.
Fix the HTML form deserialization issue where:

amenities: invalid type: string "1", expected a sequence

The application must continue using normal server-rendered HTML forms and native browser navigation. Do NOT introduce client-side routing or require SPA behavior.

Audit how checkbox groups and multi-value form controls are rendered and submitted.

The backend must correctly support:
1. no amenities selected
2. exactly one amenity selected
3. multiple amenities selected

The form representation must use standard application/x-www-form-urlencoded semantics and repeated field names where appropriate.

Do not solve this by requiring JavaScript to serialize arrays.

Inspect the current Axum/Serde form implementation and the exact request body being generated by the browser.

Choose a robust reusable solution rather than adding one-off parsing code to the project creation handler. If the current form deserializer cannot reliably handle the browser representation, introduce a small reusable form-field deserializer/parser for scalar-or-repeated values and use it for appropriate checkbox/multi-select fields.

Do not weaken validation.

Add request/API tests for every form endpoint that accepts repeated fields.

At minimum test:
- project creation with zero amenities
- project creation with one amenity
- project creation with multiple amenities
- invalid amenity IDs
- duplicate amenity IDs
- missing required fields
- malformed numeric fields

Tests should exercise the actual HTTP request/form parsing path as much as practical, not only test an internal helper.

Also audit other Vec/array fields throughout the application for the same problem.

Form parsing/validation failures must return a controlled 4xx response or re-render the form with validation errors. They must never become HTTP 500 responses.

Keep normal HTML form submission and server-side rendering intact.

### 6.
Audit the property routing and UI.

The current application exposes/links to /admin/properties/new, but that route returns 404.

Determine whether property creation is intentionally project-scoped. The intended domain model is that every property belongs to a project, therefore property creation should remain contextual to a project.

If the existing implementation confirms this architecture, DO NOT create a global /admin/properties/new route merely to hide the bug.

Instead:
- remove global "New Property" links
- remove/adjust any sidebar action implying global property creation
- ensure all property creation links originate from the relevant project
- use the existing project-scoped routes for residential/commercial property creation
- ensure property edit/delete routes are consistent with the chosen URL hierarchy
- ensure APIs follow the same ownership/context model
- retain a global property listing/search page only if it is useful for administration

If any backend endpoint or template currently points to /admin/properties/new, correct it.

If the architecture instead contains a genuine requirement for standalone property creation, then implement the missing route properly. Do not make that assumption without inspecting the existing domain model.

Add route-level tests proving that all advertised property creation links resolve successfully and that invalid/missing project contexts return an appropriate 4xx response rather than 500.

### 7.
Audit the entire application for project and property video support before implementing anything.

The intended application architecture includes media support for:
- project images
- project videos
- property images
- property videos

First determine whether video support is:
1. already implemented in the data model but not exposed in UI,
2. partially implemented in backend/storage/processing,
3. or completely missing.

Inspect:
- media tables
- project/property media relationship tables
- upload endpoints
- multipart handling
- Garage/S3 storage code
- media metadata
- video processing/background jobs
- ez-ffmpeg integration/version actually present in Cargo.lock/Cargo.toml
- video codec/container configuration
- public media serving
- cleanup/deletion code

Do not duplicate an existing media abstraction.

If the backend already supports video, connect the missing routes/UI to it.

If it is incomplete, finish it according to the existing architecture.

Use environment configuration for video processing settings. Do not hardcode codec/container settings.

For browser-facing public video, use a browser-compatible delivery format/container and preserve subtitles where technically supported. Do not blindly expose uploaded MKV files as the final public browser format.

Track processing state and failures so an upload cannot silently appear successful when processing failed.

Ensure failed uploads/processing do not leave untracked Garage objects.

Add backend tests for:
- accepted video upload
- invalid file
- processing failure
- metadata persistence
- association with project
- association with property
- deletion and storage cleanup
- authorization

### 8.
Implement a proper multiple-media upload experience for projects and properties.

This is NOT client-side routing. Keep the application server-rendered.

Backend:
- accept multiple image/video files in one upload operation
- create separate media records
- associate each media record with the correct project/property
- preserve ordering/sequence
- validate file type and size server-side
- process images and videos using the existing media pipeline
- clean up partial uploads on failure
- prevent orphaned Garage objects
- enforce configured media-count limits

Frontend:
- provide a single interactive drop/select area
- allow selecting multiple files using the native multiple file input
- allow drag-and-drop
- show selected files before submission
- show image thumbnails where possible
- show video file previews/metadata where practical
- allow removing an individual selected file before submission
- preserve ordering
- show upload/processing status
- use only small, focused JavaScript for this interaction
- do not introduce a frontend framework or client-side routing

For existing media, provide:
- thumbnail/preview
- ordering
- delete control
- appropriate media type indicator

The server must remain authoritative; never rely on client-side validation for security or correctness.

Add backend tests for multiple files, mixed image/video files, invalid files, limits, partial failures, and authorization.

### 9.
Redesign public resource URL resolution so resource identity does not depend solely on the current slug.

The current public URLs use slug-only paths, for example:

/owners/test-owner

This is insufficient because changing a resource's name/slug breaks previously saved URLs.

Use a stable, parseable resource ID in public URLs.

Preferred format:

/owners/{id}-{slug}
/projects/{id}-{slug}
/properties/{id}-{slug}

The numeric/database ID is the authoritative resource identifier. The slug is SEO/presentation data.

Do NOT use a random slug as the authoritative public identifier.

Implement a reusable public-resource route parser that:
1. extracts the stable ID
2. validates the ID
3. loads the resource by ID
4. computes the resource's current canonical slug
5. compares the requested slug to the current canonical slug

Behavior:

Canonical URL:
GET /projects/123-current-name
-> serve the page normally.

Old/stale slug:
GET /projects/123-old-name
-> load project 123
-> determine current canonical URL
-> return HTTP 301 Moved Permanently to /projects/123-current-name.

Wrong slug but valid ID:
GET /projects/123-completely-wrong
-> also redirect permanently to the canonical URL.

Nonexistent ID:
GET /projects/999999-anything
-> return normal 404.

Malformed ID:
GET /projects/not-an-id-name
-> return normal 404 rather than performing an expensive slug-only search.

Do not make slug matching the primary lookup mechanism.

Where backwards compatibility is required for existing slug-only URLs, implement a temporary/legacy resolution mechanism:
- first attempt the new ID-based route
- for legacy slug-only routes, perform an exact slug lookup
- if found, permanently redirect to the new ID+slug canonical URL
- do not use fuzzy matching
- do not redirect ambiguous results

Review all public entities that have slug-based URLs and apply the same mechanism consistently.

Also review slug generation:
- normalize consistently
- ensure uniqueness
- regenerate when the resource's canonical name changes according to the application's existing slug policy
- preserve stable IDs
- do not put route prefixes inside the stored slug

Add tests for:
- canonical URL
- stale slug
- incorrect slug with correct ID
- nonexistent ID
- malformed ID
- changed resource name
- legacy slug-only URL
- ambiguous legacy slug
- deleted resource

All canonical redirects must use the canonical public-site origin/configuration when an absolute URL is required.

Do not use random slugs as a substitute for stable IDs.

### 10.
Fix the residential property BHK and bathroom overflow bug.

The current PostgreSQL definition uses a numeric type equivalent to:

NUMERIC(2,1)

This only permits values below 10 and causes PostgreSQL error 22003 when larger valid real-estate values are submitted.

Audit the intended domain range and choose an appropriate database representation for:
- bedroom/BHK count
- bathroom count

Do not simply increase precision arbitrarily. Determine the actual valid range supported by the UI/domain.

The database constraint, Rust input type, HTML input min/max/step, and server-side validation must all agree.

Support fractional values where the domain requires them, such as:
- 1
- 1.5
- 2
- 2.5
- 3

Validate values before attempting the database INSERT/UPDATE.

Most importantly, invalid or out-of-range user input must never become HTTP 500.

Expected behavior:
- invalid value -> validation error
- form is re-rendered
- submitted values are preserved where safe
- user sees a useful field-level error
- database is not reached when input is obviously invalid

Audit other numeric property fields for similar overly restrictive PostgreSQL precision/scale definitions.

Add API/request tests for:
- normal BHK values
- fractional BHK values if supported
- large but valid values
- zero
- negative values
- excessive values
- invalid textual values
- bathroom equivalents

Verify that all validation failures produce controlled 4xx/form-validation behavior and never SQL numeric-overflow 500 responses.