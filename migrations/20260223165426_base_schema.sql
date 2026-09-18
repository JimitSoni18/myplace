-- Revised base schema for the real-estate platform.
-- This replaces the initial prototype schema entirely.
--
-- Tables NOT touched by this migration (intentionally preserved):
--   profiles       – login credentials (unchanged)
--   admin_users    – admin membership (add PK, otherwise unchanged)
--   project_owners – updated in place with new columns
--   locations      – updated in place with new columns
--   projects       – updated in place with new columns
--
-- Tables dropped from the prototype:
--   images, project_images, residential_properties, commercial_properties
--   (and the property_type / property_status ENUMs)
--
-- New tables added:
--   amenities, project_amenities
--   media, project_media, project_documents
--   property_types, properties
--   residential_property_details, commercial_property_details, land_property_details
--   property_amenities, property_listings, property_media
--   project_enquiries, property_enquiries

-- ===========================================================================
-- PROFILES  (unchanged – login credentials for both admins and owners)
-- ===========================================================================
CREATE TABLE profiles (
	id         INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	username   VARCHAR(255) NOT NULL UNIQUE,
	password   TEXT NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ===========================================================================
-- ADMIN USERS
-- ===========================================================================
CREATE TABLE admin_users (
	profile_id INT NOT NULL PRIMARY KEY REFERENCES profiles(id) ON DELETE RESTRICT
);

-- ===========================================================================
-- PROJECT OWNERS
-- ===========================================================================
CREATE TABLE project_owners (
	id                   INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	-- login credentials link
	profile_id           INT  UNIQUE REFERENCES profiles(id) ON DELETE RESTRICT,
	-- public-facing information
	name                 VARCHAR(255) NOT NULL,
	slug                 TEXT NOT NULL,
	bio                  TEXT,
	website              TEXT,
	email                TEXT,
	phone                TEXT,
	-- access control
	active               BOOL NOT NULL DEFAULT FALSE,
	-- profile image (deterministic keys: owners/{id}/original.webp etc.)
	profile_img_key      TEXT,
	profile_img_thumb_key TEXT,
	-- timestamps
	created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	deleted_at           TIMESTAMPTZ
);

CREATE INDEX idx_project_owners_slug ON project_owners(slug);

-- ===========================================================================
-- LOCATIONS
-- ===========================================================================
CREATE TABLE locations (
	id                 INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	-- structured geographic fields
	country_code       CHAR(2),
	country_name       TEXT,
	state_or_province  TEXT,
	city               TEXT,
	locality           TEXT,
	area               TEXT,
	postal_code        TEXT,
	formatted_address  TEXT NOT NULL,
	-- coordinates (for future map integration)
	latitude           NUMERIC(10, 7),
	longitude          NUMERIC(10, 7),
	-- external provider metadata (Google Places / OSM etc.)
	provider           TEXT,
	provider_place_id  TEXT,
	-- timestamps
	created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================================================
-- PROJECTS
-- ===========================================================================
CREATE TABLE projects (
	id               INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_owner_id INT  NOT NULL REFERENCES project_owners(id) ON DELETE RESTRICT,
	location_id      INT  NOT NULL REFERENCES locations(id)      ON DELETE RESTRICT,
	name             TEXT NOT NULL,
	slug             TEXT NOT NULL,
	description      TEXT,   -- stored as Markdown
	category         TEXT NOT NULL DEFAULT 'Residential'
	                 CHECK (category IN ('Residential', 'Commercial', 'Mixed')),
	start_date       DATE,
	launch_date      DATE,
	possession_date  DATE,
	created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	deleted_at       TIMESTAMPTZ
);

CREATE INDEX idx_projects_owner      ON projects(project_owner_id);
CREATE INDEX idx_projects_location   ON projects(location_id);
CREATE INDEX idx_projects_deleted_at ON projects(deleted_at);
CREATE INDEX idx_projects_slug       ON projects(slug);

-- ===========================================================================
-- AMENITIES  (reusable, normalized, admin-managed)
-- ===========================================================================
CREATE TABLE amenities (
	id         INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name       TEXT NOT NULL UNIQUE,
	slug       TEXT NOT NULL UNIQUE,
	is_active  BOOL NOT NULL DEFAULT TRUE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Many-to-many: projects ↔ amenities
CREATE TABLE project_amenities (
	project_id INT NOT NULL REFERENCES projects(id)  ON DELETE CASCADE,
	amenity_id INT NOT NULL REFERENCES amenities(id) ON DELETE CASCADE,
	PRIMARY KEY (project_id, amenity_id)
);

-- ===========================================================================
-- MEDIA  (generic – images, videos, documents)
-- ===========================================================================
CREATE TABLE media (
	id             UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	media_type     TEXT NOT NULL CHECK (media_type IN ('image', 'video', 'document')),
	mime_type      TEXT NOT NULL,
	s3_key         TEXT NOT NULL UNIQUE,
	thumbnail_key  TEXT,           -- NULL for non-visual media
	file_size      BIGINT,         -- bytes
	width          INT,            -- pixels, images/video only
	height         INT,
	duration_secs  INT,            -- video only
	created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Project media gallery (ordered, up to 10 items in phase 1)
CREATE TABLE project_media (
	project_id INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	media_id   UUID NOT NULL REFERENCES media(id)    ON DELETE RESTRICT,
	sequence   SMALLINT NOT NULL,
	PRIMARY KEY (project_id, media_id),
	UNIQUE (project_id, sequence)
);

CREATE INDEX idx_project_media_project ON project_media(project_id);

-- Project documents (RERA, BU approval, brochure, etc.)
CREATE TABLE project_documents (
	id           INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id   INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	media_id     UUID NOT NULL REFERENCES media(id)    ON DELETE RESTRICT,
	display_name TEXT NOT NULL,
	doc_type     TEXT NOT NULL
	             CHECK (doc_type IN ('rera', 'bu_approval', 'brochure', 'other')),
	created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================================================
-- PROPERTY TYPES  (database-driven classification — NO enums)
-- ===========================================================================
CREATE TABLE property_types (
	id        INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	category  TEXT NOT NULL CHECK (category IN ('RESIDENTIAL', 'COMMERCIAL', 'LAND')),
	name      TEXT NOT NULL,
	slug      TEXT NOT NULL UNIQUE,
	parent_id INT  REFERENCES property_types(id),
	is_active BOOL NOT NULL DEFAULT TRUE
);

-- Seed the standard property types
INSERT INTO property_types (category, name, slug) VALUES
	('RESIDENTIAL', 'Bungalow',              'bungalow'),
	('RESIDENTIAL', 'Villa',                 'villa'),
	('RESIDENTIAL', 'Tenement',              'tenement'),
	('RESIDENTIAL', 'Row House',             'row-house'),
	('RESIDENTIAL', 'Penthouse',             'penthouse'),
	('RESIDENTIAL', 'Flat',                  'flat'),
	('RESIDENTIAL', 'Farmhouse',             'farmhouse'),
	('COMMERCIAL',  'Office',                'office'),
	('COMMERCIAL',  'Showroom',              'showroom'),
	('COMMERCIAL',  'Shop',                  'shop'),
	('COMMERCIAL',  'Warehouse',             'warehouse'),
	('COMMERCIAL',  'Factory',               'factory'),
	('LAND',        'Residential Plot',      'residential-plot'),
	('LAND',        'Commercial Plot',       'commercial-plot'),
	('LAND',        'Agricultural Land',     'agricultural-land'),
	('LAND',        'Building-approved Land','building-approved-land'),
	('LAND',        'Industrial Land',       'industrial-land');

-- ===========================================================================
-- PROPERTIES  (unified — replaces residential_properties + commercial_properties)
-- ===========================================================================
CREATE TABLE properties (
	id               INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id       INT  NOT NULL REFERENCES projects(id)       ON DELETE RESTRICT,
	property_type_id INT  NOT NULL REFERENCES property_types(id),
	unit_number      TEXT,
	building         TEXT,
	floor_number     INT,
	total_floors     INT,
	built_up_area    NUMERIC(12, 2),   -- square feet (canonical unit)
	usable_area      NUMERIC(12, 2),   -- square feet
	description      TEXT,
	slug             TEXT NOT NULL,
	created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	deleted_at       TIMESTAMPTZ,
	searchable_tokens tsvector GENERATED ALWAYS AS (
		to_tsvector(
			'english',
			coalesce(unit_number, '') || ' ' ||
			coalesce(building,    '') || ' ' ||
			coalesce(description, '')
		)
	) STORED
);

CREATE INDEX idx_properties_project    ON properties(project_id);
CREATE INDEX idx_properties_type       ON properties(property_type_id);
CREATE INDEX idx_properties_deleted_at ON properties(deleted_at);
CREATE INDEX idx_properties_slug       ON properties(slug);
CREATE INDEX idx_properties_search     ON properties USING GIN(searchable_tokens);

-- ---------------------------------------------------------------------------
-- Subtype detail tables  (1-to-1 with properties, only the relevant one exists)
-- ---------------------------------------------------------------------------

CREATE TABLE residential_property_details (
	property_id    INT NOT NULL PRIMARY KEY REFERENCES properties(id) ON DELETE CASCADE,
	bedroom_count  NUMERIC(2, 1),   -- 2.5 for studios/lofts etc.
	bathroom_count SMALLINT,
	balcony_count  SMALLINT,
	is_duplex      BOOL NOT NULL DEFAULT FALSE,
	parking        TEXT            -- free-form for now (e.g. "2 covered")
);

CREATE TABLE commercial_property_details (
	property_id INT NOT NULL PRIMARY KEY REFERENCES properties(id) ON DELETE CASCADE,
	parking     TEXT
);

CREATE TABLE land_property_details (
	property_id        INT NOT NULL PRIMARY KEY REFERENCES properties(id) ON DELETE CASCADE,
	parcel_number      TEXT,
	zoning             TEXT,
	approval_status    TEXT,
	development_status TEXT
);

-- Many-to-many: properties ↔ amenities
CREATE TABLE property_amenities (
	property_id INT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
	amenity_id  INT NOT NULL REFERENCES amenities(id)  ON DELETE CASCADE,
	PRIMARY KEY (property_id, amenity_id)
);

-- ===========================================================================
-- PROPERTY LISTINGS  (sale / rent / lease — replaces status enum on property)
-- ===========================================================================
CREATE TABLE property_listings (
	id             INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	property_id    INT  NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
	listing_type   TEXT NOT NULL CHECK (listing_type IN ('sale', 'rent', 'lease')),
	status         TEXT NOT NULL CHECK (status IN ('active', 'closed', 'sold', 'withdrawn'))
	               DEFAULT 'active',
	currency_code  CHAR(3) NOT NULL DEFAULT 'INR',
	price          NUMERIC(18, 2),
	billing_period TEXT,            -- NULL for sale; 'monthly' / 'yearly' for rent/lease
	created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_property_listings_property ON property_listings(property_id);
CREATE INDEX idx_property_listings_status   ON property_listings(status);

-- ===========================================================================
-- PROPERTY MEDIA  (ordered gallery, up to 20 items)
-- ===========================================================================
CREATE TABLE property_media (
	property_id INT  NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
	media_id    UUID NOT NULL REFERENCES media(id)      ON DELETE RESTRICT,
	sequence    SMALLINT NOT NULL,
	PRIMARY KEY (property_id, media_id),
	UNIQUE (property_id, sequence)
);

CREATE INDEX idx_property_media_prop ON property_media(property_id);

-- ===========================================================================
-- ENQUIRY / LEAD FORMS
-- ===========================================================================
CREATE TABLE project_enquiries (
	id         INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	name       TEXT NOT NULL,
	phone      TEXT NOT NULL,
	email      TEXT,
	message    VARCHAR(255),
	is_read    BOOL NOT NULL DEFAULT FALSE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE property_enquiries (
	id          INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	property_id INT  NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
	name        TEXT NOT NULL,
	phone       TEXT NOT NULL,
	email       TEXT,
	message     VARCHAR(255),
	is_read     BOOL NOT NULL DEFAULT FALSE,
	created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
