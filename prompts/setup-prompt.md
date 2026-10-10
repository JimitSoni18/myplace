You are working on an existing real-estate web application. I want you to act as a senior Rust/Axum/PostgreSQL engineer and continue development of the existing application rather than creating a new application or imposing a completely different architecture.

Your first job is to inspect and understand the existing repository thoroughly. Do NOT start writing code immediately.

## 1. Repository review — do this first

Inspect the repository structure and identify:

* Rust project structure and crates/modules
* Axum routes and handlers
* application state (`AppState`) and how dependencies are wired
* authentication and authorization implementation
* Askama templates and template inheritance
* existing admin UI and CSS conventions
* database access patterns
* SQLx usage and migrations
* error handling conventions
* configuration/environment-variable handling
* Docker and docker-compose setup
* Garage/S3 configuration and how the S3 client is initialized
* existing image upload/storage code
* existing redirect/form-handling patterns
* how static assets are served, if applicable

Read the relevant files before making architectural decisions.

Pay particular attention to existing code around:

* admin login/logout
* admin routes
* project owners
* locations
* projects
* image upload
* S3/Garage integration
* Askama templates
* database migrations

Follow the project's existing conventions whenever they are reasonable.

Do not introduce a new framework, ORM, frontend framework, JavaScript framework, CSS framework, or storage abstraction unless the repository clearly already uses one or there is a strong technical reason.

The UI is intentionally server-rendered. Prefer HTML forms, normal navigation, Askama and CSS. Use JavaScript only where it provides meaningful UX that cannot reasonably be achieved with HTML/CSS.

---

# 2. Existing product

This is an admin interface for a real-estate website.

Administrators manage:

* Locations
* Project Owners
* Projects
* Properties

  * Residential
  * Commercial
* Project images
* Project-owner profile images
* Other admin users

There is also a public-facing resource listing/navigation area that should eventually allow non-admin users to browse resources.

The admin UI currently uses a warm/off-white/brown theme.

Existing design variables are approximately:

```css
:root {
    --bg-main: #f7efe6;
    --bg-panel: #fffaf5;
    --accent: #6b3e26;
    --accent-dark: #4b2a1a;
    --border: #e6d5c3;
    --text: #3e2b21;
}
```

Continue this visual language.

The UI should be:

* clean
* compact
* readable
* intuitive
* responsive
* not excessively spacious
* not visually dense
* primarily server-rendered
* accessible where reasonably practical
* semantic HTML rather than a "div soup"

Use elements such as:

```html
<header>
<nav>
<aside>
<main>
<section>
<form>
<table>
<footer>
```

where appropriate.

---

# 3. Current admin layout

The current Askama base layout is roughly:

```text
+------------------------------------------------+
| hamburger | Real Estate Admin       admin name |
+----------------+-------------------------------+
|                |                               |
| Dashboard      |                               |
| Locations      |             main              |
| Project Owners |                               |
| Projects       |                               |
| Properties     |                               |
|                |                               |
|----------------|                               |
| Admin Users    |                               |
| Logout         |                               |
+----------------+-------------------------------+
```

Desktop:

* sidebar on the left
* top navbar/main header on the right
* main content scrolls independently

Mobile:

* sidebar becomes a CSS-only drawer
* hamburger opens it
* close button appears in the drawer
* avoid JavaScript for opening/closing the drawer
* CSS checkbox/radio techniques are acceptable

The active sidebar resource should be visually highlighted.

Admin Users and Logout belong at the bottom of the sidebar rather than mixed into resource navigation.

Do not reintroduce a global/unified search box or multiple generic "Add" buttons into the base layout.

---

# 4. Database schema

The current schema is can be viewed from the migration file; currently there is a single migration file, which has not been run, so the initial updates are to be made on the same migration.

Treat this as the current source of truth, but review it critically for obvious schema problems, inconsistencies, missing constraints, indexing mistakes, naming problems, and future maintainability issues.

Do not silently make large schema changes.

If you identify improvements, explain them and preferably create separate migrations for intentional changes.

---

# 5. Important schema observations to investigate

Please specifically investigate these rather than assuming they are correct:

### `images`

Currently:

```sql
images (
    id,
    s3_key
)
```

and project images reference it:

```sql
project_images (
    image_id,
    project_id,
    sequence
)
```

We want to support multiple images per project.

A useful conceptual storage layout is:

```text
media/{media_id}/original.webp
media/{media_id}/thumbnail.webp
```

or an equivalent structure consistent with the existing application.

For project images, each image should have its own identity, and a project can have many images.

Do not blindly redesign this into a completely different media system. First understand how the current code uses `images` and `project_images`.

### Project-owner profile images

A project owner has at most one profile image.

Current columns:

```sql
profile_img_key
profile_img_thumb_key
```

### Property type

There is currently:

```sql
CREATE TYPE property_type AS ENUM(...)
```

but the residential properties table shown does not contain a `property_type` column.

Investigate whether this is an unfinished feature/bug.

Do not invent semantics for the enum.

---

# 6. Garage / S3 storage

The application uses Garage, not AWS S3.

The Rust application uses:

```rust
aws_sdk_s3::Client
```

because Garage implements the S3 API.

The S3 API is currently available locally on:

```text
http://127.0.0.1:3900
```

Garage web access is configured separately.

Current Garage configuration includes:

```toml
[s3_web]
bind_addr = "[::]:3903"
root_domain = ".web.docker.localhost"
```

Docker exposes the relevant ports, currently including:

```yaml
ports:
  - "3900:3900"
  - "3901:3901"
  - "3902:3902"
  - "8080:8080"
```

The Garage web endpoint is different from the S3 API.

Garage does not support anonymous access through the normal S3 endpoint, so this does NOT work:

```text
http://localhost:3900/myplace/project-images/1/thumbnail.webp
```

The S3 API correctly returns AccessDenied.

Garage's web hosting uses the bucket as a virtual host. For example, with an appropriate root domain:

```text
http://myplace.web.docker.localhost:3903/project-images/1/thumbnail.webp
```

The existing object storage has been verified with:

```bash
aws s3 ls --endpoint-url http://127.0.0.1:3900 s3://myplace/project-images/1/
```

and contains:

```text
original.webp
thumbnail.webp
```

The web endpoint returned 404 when accessed as:

```text
http://localhost:3903/project-images/1/thumbnail.webp
```

because Garage web hosting expects the bucket to be represented in the hostname rather than as a path.

We are still in local development and have NOT deployed production infrastructure.

Do not hardcode:

```text
localhost:3900
```

into application templates.

Prefer configuration for the public asset base URL, for example:

```text
ASSET_BASE_URL
```

or an equivalent configuration mechanism consistent with the existing application.

For local development, the public URL may be something like:

```text
http://myplace.web.docker.localhost:3903
```

but inspect the actual Garage configuration before implementing this.

The backend S3 endpoint and browser-facing asset URL are two different concepts:

```text
Backend:
Axum -> Garage S3 API

Browser:
Browser -> Garage Web endpoint
```

Keep that separation.

---

# 7. Project owner UI already implemented

There is already an Askama project-owner form with:

* name
* description
* image upload when editing an existing owner
* details/image tabs
* image upload via `multipart/form-data`
* server-side validation
* optional error rendering
* themed CSS

The form currently looks conceptually like:

```text
Edit Owner

[ Details ] [ Image ]

Name
[________________________]

Description
[________________________]
[________________________]

Error if applicable

[ Save ] [ Cancel ]
```

Image tab:

```text
Current image (if one exists)

[ image preview ]

[ Remove Image ]

[ Drop image here or click to upload ]

[ Upload ] [ Cancel ]
```

The image input is intentionally visually hidden while its `<label>` forms the drop target:

```html
<label class="file-drop">
    <input
        type="file"
        name="image"
        accept="image/*"
    >
    <span class="file-text">
        Drop image here or click to upload
    </span>
</label>
```

The input should cover the entire label and be transparent so normal browser file dropping works.

A small amount of JavaScript is acceptable to preview a selected image before submitting it.

When opening an edit form, an existing image should also be shown.

If there is no image, show the upload UI normally.

For creating a new owner, there should currently be no image-upload tab. An owner must first exist before an image can be associated with it.

---

# 8. Current image upload behavior

The existing Rust handler roughly does:

```rust
pub async fn add_project_owner_image(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Redirect, ProjectOwnerImageUploadError>
```

It:

1. extracts the `image` multipart field
2. reads bytes
3. uses `spawn_blocking`
4. decodes the image
5. encodes WebP
6. generates a 300x300 thumbnail
7. uploads both objects to Garage
8. stores their keys in `project_owners`

Current key generation is approximately:

```rust
let base_path = format!("project-images/{id}");

let thumb_key = format!("{base_path}/thumbnail.webp");
let original_img_key = format!("{base_path}/original.webp");
```

For a project-owner profile image, this should probably become owner-specific:

```text
owners/{id}/original.webp
owners/{id}/thumbnail.webp
```

because the owner only has one profile image.

Uploading a replacement can overwrite those same keys.

The image removal functionality should:

1. load the current keys from the database
2. delete the corresponding Garage objects
3. clear the DB columns
4. redirect back to the edit page

Consider failure cases carefully. For example, avoid leaving the DB pointing at an object that has already been deleted if an operation fails midway.

Do not pretend S3/Garage operations are transactional; reason about the ordering and failure behavior.

---

# 9. What I want you to build

Continue the admin application into a complete CRUD management interface.

Build incrementally, preserving the existing architecture.

The main resources are:

## Locations

Fields:

```text
area
```

The location page should have:

* list/table
* search field
* Apply button
* create location action
* edit action
* delete action
* sensible empty state
* validation/error display

Keep the table compact.

## Project Owners

Fields:

```text
name
description
profile image
slug
created_at
```

The list should be visually pleasant but not oversized.

A compact table/list with a small thumbnail is appropriate.

Consider showing:

```text
Image | Name | Description | Created | Actions
```

Do not display the full description if it makes the list unwieldy. Truncate visually if appropriate while keeping useful information.

Actions should be visually consistent.

Delete should use JavaScript with `fetch()` and an HTTP DELETE request rather than a form.

No unnecessary JavaScript framework.

The edit page has the Details/Image tabs described above.

## Projects

A project belongs to:

```text
Location
Project Owner
```

and has:

```text
name
slug
```

Projects can have multiple images.

Design the CRUD UI appropriately.

The project edit page should eventually have a sensible image-management section supporting:

* multiple images
* thumbnails
* ordering/sequence
* adding images
* deleting images
* potentially reordering images

Do not over-engineer drag-and-drop ordering unless it is simple and justified.

## Properties

There are two resource types:

```text
Residential
Commercial
```

Build a clear UI for selecting/managing the two types.

Residential currently has:

```text
project
sold_at
house_number
building
is_duplex
bedroom_count
description
status
rent_or_lease_duration
slug
created_at
```

Commercial currently has:

```text
project
sold_at
unit_number
building
description
status
rent_or_lease_duration
slug
created_at
```

Use appropriate HTML controls:

* `<select>` for project
* checkbox for boolean values
* numeric input where appropriate
* textarea for descriptions
* appropriate status selection

Do not invent additional business fields.

---

# 10. Search/listing behavior

The list pages should generally support:

* search
* Apply button
* pagination if needed
* useful columns
* edit action
* delete action where appropriate
* empty states

Do not introduce one giant global search.

Search should be resource-specific.

Where PostgreSQL full-text search already exists, understand and use it appropriately.

For example, residential and commercial properties already have:

```sql
searchable_tokens tsvector
```

with GIN indexes.

Do not replace these with application-side string scanning.

---

# 11. HTTP semantics

The application should use appropriate HTTP methods.

HTML forms natively give us:

```text
GET
POST
```

For operations such as deletion, it is acceptable to use minimal JavaScript:

```javascript
fetch(url, {
    method: "DELETE"
})
```

Do not introduce JavaScript merely to emulate ordinary form submissions.

If a `fetch()` request receives a `Refresh` header, remember that fetch itself does not cause the browser to navigate/refresh the page. If navigation is desired, explicitly do it with minimal JS.

For normal form submissions, prefer server redirects.

---

# 12. Authentication

Admin pages must remain protected by the application's existing authentication/authorization mechanism.

Do not weaken authentication in order to make development easier.

Inspect the existing auth implementation and follow it.

Admin users should be able to:

* log in
* log out
* access admin resources
* manage other admin users

Do not expose admin-management operations to unauthenticated users.

---

# 13. Error handling

Follow existing Rust error handling patterns.

Forms should preferably work like:

```text
POST
  ↓
validation/database error
  ↓
render same form with entered values + error

or

POST
  ↓
success
  ↓
Redirect
```

Do not redirect to a generic error page for ordinary form validation errors.

Preserve entered values on validation failures where practical.

---

# 14. Askama conventions

Continue using Askama.

The base layout should expose blocks for things such as:

```text
title
head_tags
content
```

where appropriate.

Do not duplicate the entire HTML document in child templates.

Use template inheritance.

Be mindful of Askama syntax and Rust types. Do not invent template syntax unsupported by Askama.

---

# 15. Responsive design

Desktop should use the sidebar layout.

On small screens:

* hide sidebar
* show hamburger
* open sidebar as drawer using CSS where practical
* show close button at the same location as the hamburger when drawer is open
* keep main content usable
* avoid horizontal overflow
* tables should have a sensible mobile strategy
* don't make every component huge just to be "mobile friendly"

CSS-only drawer is preferred.

A small amount of JavaScript is fine elsewhere, but the navigation drawer does not need JS.

---

# 16. Docker/development setup

Inspect the existing:

* Dockerfile
* docker-compose.yml
* Garage container
* PostgreSQL container
* application container
* environment variables
* mounted volumes
* network configuration

Verify that the local development setup is internally consistent.

In particular, distinguish:

```text
container-to-container address
```

from:

```text
browser/host address
```

Do not assume `localhost` means the same thing inside and outside Docker.

If the Rust application is running inside Docker, its S3 endpoint should not necessarily be:

```text
localhost:3900
```

because that would refer to the application container itself.

Use the existing Docker network/service names appropriately.

Likewise, browser-facing Garage URLs need to use host-accessible ports/domains.

Do not blindly modify Docker configuration. First inspect it.

---

# 17. Security review

As part of implementation, identify obvious security issues, especially:

* password storage
* authentication cookies
* CSRF considerations for state-changing forms
* unrestricted image uploads
* image size limits
* decompression bombs / extremely large images
* malicious file types
* filename handling
* S3/Garage credentials
* admin authorization
* IDOR/resource authorization
* delete endpoints
* SQL injection
* unsafe redirects
* HTML escaping

Do not turn this into a giant security rewrite.

Fix clear issues that are directly relevant to the implementation and flag larger architectural issues separately.

For image uploads, do not trust the browser's MIME type or filename. The server should validate/decode the image as it already does.

Also consider upload size limits before reading arbitrarily large files into memory.

---

# 18. Database review

Before adding migrations, review the current schema and migration structure.

Look for:

* redundant indexes
* missing indexes on foreign keys that are frequently queried
* uniqueness constraints
* nullable/non-nullable mistakes
* soft-delete semantics
* slug uniqueness
* cascade behavior
* sequence/order constraints
* appropriate data types

If you make schema changes, create proper SQLx migrations rather than modifying old migrations that may already have been applied.

Do not destroy existing data.

---

# 19. Implementation strategy

Work in stages.

### Stage 1 — Understand

Inspect the repository and summarize:

* architecture
* important files
* current routes
* templates
* DB layer
* auth
* storage
* Docker setup
* anything that conflicts with the requirements above

Do not code yet.

### Stage 2 — Identify issues

Create a concise implementation plan.

Separate:

```text
Required for requested functionality
```

from:

```text
Recommended improvements
```

Do not implement speculative improvements without justification.

### Stage 3 — Implement

Start with the most foundational pieces.

Prefer small coherent changes rather than one enormous rewrite.

After each meaningful change:

* run formatting
* run compilation/checks
* run tests if present
* fix errors
* inspect generated/rendered templates if possible

Use:

```bash
cargo fmt
cargo check
cargo test
```

as appropriate.

Also validate SQL migrations.

### Stage 4 — Review

After implementation, inspect your own changes for:

* broken Askama templates
* incorrect route methods
* missing auth
* incorrect SQL
* incorrect foreign-key behavior
* Docker networking mistakes
* incorrect Garage URLs
* responsive CSS regressions
* unnecessary JavaScript
* inconsistent UI

---

# 20. Important instruction: don't hallucinate project infrastructure

Do not assume that a type/function/configuration exists merely because it would be convenient.

For example, do NOT invent something like:

```rust
state.storage.public_url(...)
```

unless such a type actually exists in the repository.

The application currently uses:

```rust
aws_sdk_s3::Client
```

for Garage.

Inspect the actual code before referencing:

* configuration structs
* S3 helpers
* storage abstractions
* URL generators
* authentication helpers
* database repositories
* application state

If something required is missing, create it deliberately and explain why.

---

# 21. Important instruction: don't over-engineer

This is a relatively small real-estate application.

Prefer:

```text
simple
explicit
server-rendered
type-safe
maintainable
```

over:

```text
framework-heavy
abstract
generic
over-engineered
```

For example, do not build a generic CRUD framework for every table just because several resources have CRUD operations.

Some duplication is preferable if it keeps the code straightforward.

Likewise, don't build a frontend SPA.

---

# 22. Expected final state

The end result should feel like a cohesive admin application rather than a collection of unrelated CRUD pages.

Navigation:

```text
Dashboard
Locations
Project Owners
Projects
Properties
----------------
Admin Users
Logout
```

Each resource should have:

```text
list
create
edit
delete (where appropriate)
search/filter where useful
```

The UI should retain the existing warm real-estate aesthetic:

```text
off-white / bisque backgrounds
dark brown / coffee accents
subtle borders
compact controls
small readable text
```

The admin interface should be usable on both desktop and mobile.

The public-facing resource URLs should remain conceptually separate from admin URLs.

---

## One final instruction

Do not simply implement everything in one giant pass.

First inspect the repository and report what you found and what you think should be changed.

Then proceed with implementation using the existing architecture.

If you encounter an ambiguity where the correct implementation depends on information that genuinely cannot be inferred from the repository, stop and ask me rather than inventing an API, type, configuration field, database relationship, or storage mechanism.
