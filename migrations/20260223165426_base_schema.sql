-- Base schema for the project-centric real-estate platform.
--
-- Tables:
--   profiles              – login credentials for admins and project owners
--   admin_users           – admin membership
--   project_owners        – real estate developers / builders
--   project_types         -- database-driven classifications (Residential, Commercial, Mixed Use, Plotted)
--   projects              – primary marketing & inventory entity with embedded location & RERA info
--   amenities             – reusable amenities master (without slug)
--   project_amenities     – many-to-many link between projects and amenities
--   media                 – uploaded images, videos, documents
--   project_media         – project gallery photos / videos
--   project_documents     – brochures, legal certificates, approvals
--   project_configurations – BHK unit typologies, areas (sq ft), pricing, and 1:1 floor plan media
--   project_floor_plans   – project-level architectural blueprints (master site plans, cluster layouts)
--   project_specifications – categorized technical specifications (materials, finishes, fittings)
--   project_landmarks     – neighborhood infrastructure, connectivity, distance/commute
--   project_enquiries     – lead capture forms for projects

-- ===========================================================================
-- PROFILES
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
-- PROJECT OWNERS (BUILDERS / DEVELOPERS)
-- ===========================================================================
CREATE TABLE project_owners (
	id                    INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	profile_id            INT  UNIQUE REFERENCES profiles(id) ON DELETE RESTRICT,
	name                  VARCHAR(255) NOT NULL,
	slug                  TEXT NOT NULL,
	bio                   TEXT,
	website               TEXT,
	email                 TEXT,
	phone                 TEXT,
	active                BOOL NOT NULL DEFAULT FALSE,
	profile_img_key       TEXT,
	profile_img_thumb_key TEXT,
	created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	deleted_at            TIMESTAMPTZ
);

CREATE INDEX idx_project_owners_slug ON project_owners(slug);

-- ===========================================================================
-- PROJECT TYPES (Primary classification: Residential, Commercial, Mixed Use, Plotted)
-- ===========================================================================
CREATE TABLE project_types (
	id         INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name       TEXT NOT NULL UNIQUE,
	is_active  BOOL NOT NULL DEFAULT TRUE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO project_types (name) VALUES
	('Residential'),
	('Commercial'),
	('Mixed Use'),
	('Plotted');

-- ===========================================================================
-- PROJECTS (Unified central entity with embedded location & RERA)
-- ===========================================================================
CREATE TABLE projects (
	id                 INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_owner_id   INT  NOT NULL REFERENCES project_owners(id) ON DELETE RESTRICT,
	project_type_id    INT  NOT NULL REFERENCES project_types(id)  ON DELETE RESTRICT,
	name               TEXT NOT NULL,
	slug               TEXT NOT NULL,
	description        TEXT,   -- stored as Markdown
	-- Embedded Location
	formatted_address  TEXT NOT NULL DEFAULT '',
	city               TEXT NOT NULL DEFAULT '',
	area               TEXT NOT NULL DEFAULT '',
	state_or_province  TEXT NOT NULL DEFAULT '',
	postal_code        TEXT,
	latitude           DOUBLE PRECISION,
	longitude          DOUBLE PRECISION,
	-- RERA & Scale
	rera_number        TEXT,
	rera_authority     TEXT,
	total_land_area    DOUBLE PRECISION, -- canonical sq ft
	towers_count       INT,
	floors_count       INT,
	units_count        INT,
	-- Milestone Dates
	start_date         DATE,
	launch_date        DATE,
	possession_date    DATE,
	created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	deleted_at         TIMESTAMPTZ
);

CREATE INDEX idx_projects_owner       ON projects(project_owner_id);
CREATE INDEX idx_projects_type        ON projects(project_type_id);
CREATE INDEX idx_projects_city        ON projects(city);
CREATE INDEX idx_projects_area        ON projects(area);
CREATE INDEX idx_projects_deleted_at  ON projects(deleted_at);
CREATE INDEX idx_projects_slug        ON projects(slug);

-- ===========================================================================
-- AMENITIES (Reusable, normalized, admin-managed without slug)
-- ===========================================================================
CREATE TABLE amenities (
	id         INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name       TEXT NOT NULL UNIQUE,
	is_active  BOOL NOT NULL DEFAULT TRUE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Many-to-many: projects <-> amenities
CREATE TABLE project_amenities (
	project_id INT NOT NULL REFERENCES projects(id)  ON DELETE CASCADE,
	amenity_id INT NOT NULL REFERENCES amenities(id) ON DELETE CASCADE,
	PRIMARY KEY (project_id, amenity_id)
);

-- ===========================================================================
-- MEDIA (Images, videos, documents)
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

-- Project media gallery
CREATE TABLE project_media (
	project_id INT      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	media_id   UUID     NOT NULL REFERENCES media(id)    ON DELETE RESTRICT,
	sequence   SMALLINT NOT NULL,
	PRIMARY KEY (project_id, media_id),
	UNIQUE (project_id, sequence)
);

CREATE INDEX idx_project_media_project ON project_media(project_id);

-- Project documents (brochure, approvals, deeds)
CREATE TABLE project_documents (
	id           INT      NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id   INT      NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	media_id     UUID     NOT NULL REFERENCES media(id)    ON DELETE RESTRICT,
	display_name TEXT     NOT NULL,
	doc_type     TEXT     NOT NULL CHECK (doc_type IN ('rera', 'bu_approval', 'brochure', 'other')),
	created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ===========================================================================
-- PROJECT CONFIGURATIONS (BHK Typologies, areas in sq ft, price, 1:1 floor plan)
-- ===========================================================================
CREATE TABLE project_configurations (
	id                  INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id          INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	name                TEXT NOT NULL, -- e.g. "2 BHK Standard", "3 BHK Luxury"
	bhk                 DOUBLE PRECISION,
	carpet_area         DOUBLE PRECISION, -- sq ft
	built_up_area       DOUBLE PRECISION, -- sq ft
	super_built_up_area DOUBLE PRECISION, -- sq ft
	price               DOUBLE PRECISION,
	price_on_request    BOOL NOT NULL DEFAULT FALSE,
	floor_plan_media_id UUID REFERENCES media(id) ON DELETE SET NULL,
	sequence            SMALLINT NOT NULL DEFAULT 0,
	is_active           BOOL NOT NULL DEFAULT TRUE,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_proj_configs_project ON project_configurations(project_id);
CREATE INDEX idx_proj_configs_bhk     ON project_configurations(bhk);

-- ===========================================================================
-- PROJECT FLOOR PLANS (Master site plans, cluster layouts)
-- ===========================================================================
CREATE TABLE project_floor_plans (
	id         UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	media_id   UUID NOT NULL REFERENCES media(id)    ON DELETE RESTRICT,
	title      TEXT NOT NULL,
	sequence   SMALLINT NOT NULL DEFAULT 0,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_proj_floor_plans_project ON project_floor_plans(project_id);

-- ===========================================================================
-- PROJECT SPECIFICATIONS (Categorized materials, finishes, fixtures)
-- ===========================================================================
CREATE TABLE project_specifications (
	id         INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	category   TEXT NOT NULL, -- e.g. "Flooring", "Doors & Windows", "Electrical"
	items      TEXT[] NOT NULL,
	sequence   SMALLINT NOT NULL DEFAULT 0,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_proj_specs_project ON project_specifications(project_id);

-- ===========================================================================
-- PROJECT LANDMARKS (Neighborhood infrastructure, connectivity, distance/commute)
-- ===========================================================================
CREATE TABLE project_landmarks (
	id         INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	category   TEXT NOT NULL, -- 'Connectivity', 'Education', 'Healthcare', 'Shopping', 'Business'
	name       TEXT NOT NULL,
	distance   TEXT,
	duration   TEXT,
	latitude   DOUBLE PRECISION,
	longitude  DOUBLE PRECISION,
	source     TEXT NOT NULL DEFAULT 'manual', -- 'manual', 'osm', 'google'
	sequence   SMALLINT NOT NULL DEFAULT 0,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_proj_landmarks_project ON project_landmarks(project_id);

-- ===========================================================================
-- PROJECT ENQUIRIES / LEADS
-- ===========================================================================
CREATE TABLE project_enquiries (
	id               INT  NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	project_id       INT  NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	name             TEXT NOT NULL,
	phone            TEXT NOT NULL,
	email            TEXT,
	message          VARCHAR(255),
	configuration_id INT  REFERENCES project_configurations(id) ON DELETE SET NULL,
	is_read          BOOL NOT NULL DEFAULT FALSE,
	created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_project_enquiries_project ON project_enquiries(project_id);
