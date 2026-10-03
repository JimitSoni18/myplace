# Deploying MyPlace to Render.com

This guide provides end-to-end instructions for deploying the **MyPlace** application to [Render](https://render.com) directly from GitHub.

---

## 1. Architecture Overview

MyPlace is built with Rust and Axum, following a strict three-tier server boundary:

```text
                               ┌────────────────────────────────────────────────────────┐
                               │                    Render Platform                     │
                               │                                                        │
┌────────────────────────┐     │  ┌───────────────────────┐   ┌──────────────────────┐  │
│ Browser: General Users ├────►│  │ myplace-public (Web)  │   │                      │  │
└────────────────────────┘     │  │ (Port 10000 -> 8080)  │   │                      │  │
                               │  └──────────┬────────────┘   │                      │  │
┌────────────────────────┐     │  ┌──────────▼────────────┐   │   myplace-db         │  │
│ Browser: Admin Staff   ├────►│  │ myplace-admin (Web)   ├──►│   Managed PostgreSQL │  │
└────────────────────────┘     │  │ (Port 10000 -> 8081)  │   │   (with TLS)         │  │
                               │  └──────────┬────────────┘   │                      │  │
┌────────────────────────┐     │  ┌──────────▼────────────┐   │                      │  │
│ Browser: Project Owners├────►│  │ myplace-owner (Web)   │   │                      │  │
└────────────────────────┘     │  │ (Port 10000 -> 8082)  │   │                      │  │
                               │  └──────────┬────────────┘   └──────────────────────┘  │
                               └─────────────┼──────────────────────────────────────────┘
                                             │ S3 API
                                             ▼
                               ┌───────────────────────────┐
                               │   S3-Compatible Storage   │
                               │  (Cloudflare R2 / AWS S3  │
                               │   or Self-Hosted Garage)  │
                               └───────────────────────────┘
```

### Server Boundaries
All three servers are compiled into the production binary and listen concurrently:
1. **Public Server** (`public_port`, default `8080`): Serves public listings, project pages, owner directories, and `/sitemap.xml`.
2. **Admin Server** (`admin_port`, default `8081`): Serves administrative management dashboards, staff amenities, locations, and owner approvals.
3. **Project Owner Server** (`owner_port`, default `8082`): Serves owner portal dashboards, property creation, and media management.

On Render, each service deploys the same Docker container and routes Render's `$PORT` (10000) to the designated listener port, preserving boundary isolation.

---

## 2. Prerequisites & Native Runtime Dependencies

The application uses a multi-stage Docker build (`Dockerfile`) targeting `alpine:3.23`:

- **Rust & Musl libc**: Pure Rust stack with `tls-rustls` (no OpenSSL dependency).
- **FFmpeg & FFprobe**: Installed via Alpine package `ffmpeg`. Used by `src/media/mod.rs` for:
  - Video stream inspection and dimension/duration extraction (`ffprobe`).
  - WebP poster thumbnail generation at `00:00:00.500` (`ffmpeg -c:v libwebp`).
  - Container optimization with progressive streaming moov atom (`ffmpeg -movflags +faststart`).
- **CA Certificates**: Installed via Alpine package `ca-certificates` for secure outbound TLS to PostgreSQL and S3 endpoints.
- **Static Assets**: Directory `static/` is packaged in `/app/static` for CSS, JS, and image serving.

---

## 3. Database Strategy (PostgreSQL)

### Render Managed PostgreSQL
Render provides managed PostgreSQL instances that require TLS connections (`sslmode=require`).
- `sqlx` in `Cargo.toml` is configured with `features = [..., "tls-rustls", "migrate"]`.
- The application automatically establishes secure TLS connections to Render PostgreSQL without requiring native OpenSSL libraries.

### Automated Migrations (`./migrate`)
The application includes a standalone migration binary (`src/bin/migrate.rs`):
- Embeds migration scripts from `./migrations` into the binary at compile time using `sqlx::migrate!`.
- Runs automatically during deployment via Render's `preDeployCommand: "./migrate && ./seed"` or on container startup.
- Safe and idempotent; applies unapplied migrations within advisory locks.

### Admin Account Seeding (`./seed`)
The `seed` binary reads `ADMIN_USERNAME` and `ADMIN_PASSWORD` from the environment:
- Hashes the initial password with Argon2id.
- Inserts the initial administrator account and default amenities if not already present.
- Safe and idempotent; does not overwrite existing passwords on subsequent deployments.

---

## 4. Object Storage Strategy (Garage / S3)

MyPlace uses the official AWS S3 SDK (`aws-sdk-s3`) configured with path-style addressing (`force_path_style(true)`).

### S3 Backend Compatibility
The application works identically across:
1. **Self-Hosted Garage**: Local development or dedicated VPS.
2. **Cloudflare R2** (Recommended for Render production): Zero egress fees, high performance, native S3 compatibility.
3. **AWS S3 / Wasabi / MinIO**: Standard S3 object storage.

### Why Garage is Hosted Externally in Production
Render Web Services are stateless containers designed for HTTP traffic on a single port. Garage requires:
- Port 3900 (S3 API)
- Port 3901 (RPC cluster communication)
- Port 3902 (Admin API)
- Port 3903 (S3 Web static website endpoint)
- Persistent filesystem storage for metadata (`/var/lib/garage/meta`) and block storage (`/var/lib/garage/data`).

For production deployments on Render, use **Cloudflare R2** or an external Garage instance running on a VPS with persistent storage.

### Idempotent Setup Script (`scripts/setup-garage.sh`)
When using Garage, run the automated provisioning script:
```bash
# Target local docker/podman container:
./scripts/setup-garage.sh

# Target remote Garage host:
GARAGE_CMD="garage" ./scripts/setup-garage.sh
```

The script automatically:
1. Checks and provisions cluster layout (`garage layout assign`, `garage layout apply`).
2. Creates an access key if missing (`garage key create myplace-app-key`).
3. Creates the bucket if missing (`garage bucket create myplace`).
4. Grants read/write permissions (`garage bucket allow --read --write myplace --key myplace-app-key`).
5. Enables public website read access (`garage bucket website --allow myplace`).

---

## 5. Environment Variables Reference

| Variable | Description | Local Example | Render Production Example |
| :--- | :--- | :--- | :--- |
| `APP_ENV` | Application environment (`development`, `test`, `production`) | `development` | `production` |
| `DATABASE_URL` | PostgreSQL connection string with TLS | `postgres://postgres:welcome@localhost:5432/myplace` | *Auto-populated from Render Database* |
| `MAX_DB_CONNECTIONS` | Max database connection pool size | `10` | `10` |
| `COOKIE_SIGNING_SECRET` | 32-byte Base64 secret for auth cookies | `OlPxvoUZuCWoan...` | *Auto-generated in Blueprint* |
| `PUBLIC_PORT` | Port for Public web listener | `8080` | `10000` (on public service) |
| `ADMIN_PORT` | Port for Admin web listener | `8081` | `10000` (on admin service) |
| `OWNER_PORT` | Port for Project Owner web listener | `8082` | `10000` (on owner service) |
| `PUBLIC_SITE_ORIGIN` | Canonical public origin | `http://localhost:8080` | `https://myplace.onrender.com` |
| `ADMIN_SITE_ORIGIN` | Canonical admin origin | `http://localhost:8081` | `https://myplace-admin.onrender.com` |
| `OWNER_SITE_ORIGIN` | Canonical owner origin | `http://localhost:8082` | `https://myplace-owner.onrender.com` |
| `S3_ENDPOINT` | Backend S3 API endpoint URL | `http://localhost:3900` | `https://<account_id>.r2.cloudflarestorage.com` |
| `S3_BUCKET` | S3 bucket name | `myplace` | `myplace-prod` |
| `AWS_ACCESS_KEY_ID` | S3 API Access Key ID | `GKedc4cefb0a99...` | `your_production_s3_key_id` |
| `AWS_SECRET_ACCESS_KEY` | S3 API Secret Access Key | `57c65f17b5c6...` | `your_production_s3_secret` |
| `AWS_DEFAULT_REGION` | S3 Region identifier | `garage` | `auto` (or `us-east-1`) |
| `ASSET_BASE_URL` | Public browser base URL for uploaded media | `http://myplace.web.docker.localhost:3903` | `https://assets.myplace.com` (or R2 public URL) |
| `ADMIN_USERNAME` | Initial super-admin login username | `admin` | `admin` |
| `ADMIN_PASSWORD` | Initial super-admin login password | `changeme` | *Secure random string* |
| `PAGE_CACHE_CAPACITY` | In-memory TinyLFU page cache capacity | `50` | `100` |
| `RUST_LOG` | Tracing / logging level | `info,myplace=debug` | `info` |

---

## 6. Step-by-Step Render Deployment

### Option A: Deploying with Render Blueprint (`render.yaml`)

1. **Push your repository to GitHub**:
   Ensure all changes including `render.yaml`, `Dockerfile`, `.sqlx/`, and `migrations/` are committed.

2. **Connect to Render**:
   - Log in to your [Render Dashboard](https://dashboard.render.com).
   - Click **New +** > **Blueprint**.
   - Connect your GitHub repository.

3. **Configure Environment Sync**:
   Render will inspect `render.yaml` and prompt you for the uncommitted sync variables:
   - `S3_ENDPOINT`: Your Cloudflare R2 / AWS S3 endpoint.
   - `S3_BUCKET`: Your bucket name (e.g. `myplace-prod`).
   - `AWS_ACCESS_KEY_ID`: Your S3 API Key ID.
   - `AWS_SECRET_ACCESS_KEY`: Your S3 API Secret Key.
   - `ASSET_BASE_URL`: Browser-facing URL where the bucket assets can be reached.

4. **Apply Blueprint**:
   - Click **Apply**.
   - Render creates:
     - `myplace-db` (PostgreSQL)
     - `myplace-public` (Web Service)
     - `myplace-admin` (Web Service)
     - `myplace-owner` (Web Service)

5. **Set Site Origins**:
   After the services are created and assigned `.onrender.com` URLs (or custom domains), set:
   - `PUBLIC_SITE_ORIGIN`: `https://myplace-public.onrender.com`
   - `ADMIN_SITE_ORIGIN`: `https://myplace-admin.onrender.com`
   - `OWNER_SITE_ORIGIN`: `https://myplace-owner.onrender.com`
   on all three services.

---

### Option B: Deploying Manually via Render Dashboard

If you prefer not using Blueprints:

1. **Create PostgreSQL Database**:
   - Go to **New +** > **PostgreSQL**.
   - Name: `myplace-db`.
   - Copy the **Internal Database URL**.

2. **Create Public Web Service**:
   - Go to **New +** > **Web Service**.
   - Select your GitHub repo.
   - Environment: **Docker**.
   - Health Check Path: `/healthz`.
   - Pre-deploy Command: `./migrate && ./seed`.
   - Add environment variables from the table above, setting `PUBLIC_PORT=10000`.

3. **Create Admin Web Service**:
   - Repeat for Admin, setting `ADMIN_PORT=10000`, `PUBLIC_PORT=8080`, `OWNER_PORT=8082`.
   - Health Check Path: `/healthz`.

4. **Create Project Owner Web Service**:
   - Repeat for Owner, setting `OWNER_PORT=10000`, `PUBLIC_PORT=8080`, `ADMIN_PORT=8081`.
   - Health Check Path: `/healthz`.

---

## 7. Verification & Health Monitoring

### Health Checks
Each server exposes an unauthenticated health check at:
- `GET /healthz` (and `GET /health`)
- Response:
  ```json
  {
    "status": "ok",
    "database": "connected"
  }
  ```
- If PostgreSQL connectivity fails, the endpoint returns `503 Service Unavailable`.

### Testing Deployment
1. **Public Site**: Navigate to `https://myplace-public.onrender.com/`. Verify homepage, properties, and `/sitemap.xml`.
2. **Admin Portal**: Navigate to `https://myplace-admin.onrender.com/auth/admin-login`. Log in using your `ADMIN_USERNAME` and `ADMIN_PASSWORD`.
3. **Owner Portal**: Navigate to `https://myplace-owner.onrender.com/auth/owner-login`. Log in as a registered project owner.
4. **Media Upload**: Create a test project or property in the Admin or Owner portal and upload an image and MP4 video. Verify the WebP poster and faststart streaming MP4 render in the public listing.
