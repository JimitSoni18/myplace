# Deploying MyPlace to Render.com

This guide provides end-to-end instructions for deploying the **MyPlace** application to [Render](https://render.com) directly from GitHub using Render's **native Rust runtime** and **Supabase Storage**.

---

## 1. Architecture Overview

MyPlace runs as a unified, path-multiplexed web application on Render backed by Render Managed PostgreSQL and Supabase S3 storage:

```text
                               ┌────────────────────────────────────────────────────────┐
                               │                    Render Platform                     │
                               │                                                        │
┌────────────────────────┐     │  ┌──────────────────────────────────────────────────┐  │
│ Browser: All Users     ├────►│  │ myplace (Web Service, Native Rust)               │  │
│ - Public: /            │     │  │ - Port 10000 (Path Multiplexed)                  │  │
│ - Admin: /admin        │     │  │ - Embedded migrations & admin seed               │  │
│ - Owner: /owner        │     │  └──────────┬───────────────────────────┬───────────┘  │
│ - Auth: /auth          │     │             │                           │              │
└───────────┬────────────┘     │             │ PostgreSQL (TLS)          │ S3 API       │
            │                  │             ▼                           │ (put/delete) │
            │ Direct           │  ┌──────────────────────┐               │              │
            │ Public CDN Hits  │  │ myplace-db           │               │              │
            │ (ASSET_BASE_URL) │  │ Managed PostgreSQL   │               │              │
            │                  │  └──────────────────────┘               │              │
            ▼                  └─────────────────────────────────────────┼──────────────┘
┌──────────────────────────────────────────────┐                         │
│ Supabase Public Storage (S3-Compatible)      │◄────────────────────────┘
│ - Public URL: {ASSET_BASE_URL}/{s3_key}      │
│ - Global CDN for images, videos, documents   │
└──────────────────────────────────────────────┘
```

---

## 2. Environment Variables Reference for Render

| Variable | Description | Value / Example |
| :--- | :--- | :--- |
| `APP_ENV` | Environment mode | `production` |
| `SQLX_OFFLINE` | Use checked-in `.sqlx` cache for compile-time queries | `true` |
| `DATABASE_URL` | PostgreSQL connection string | *Auto-populated from Render `myplace-db`* |
| `MAX_DB_CONNECTIONS` | DB connection pool size | `10` |
| `COOKIE_SIGNING_SECRET` | 32-byte Base64 secret for session cookies | *Auto-generated in Blueprint* |
| `JWT_PRIVATE_KEY` | Ed25519 Private Key PEM string | Multiline or `\n`-escaped PEM |
| `JWT_PUBLIC_KEY` | Ed25519 Public Key PEM string | Multiline or `\n`-escaped PEM |
| `PORT` / `PUBLIC_PORT` | Listener port | `10000` |
| `ENABLE_ROUTE_MULTIPLEXING`| Enables path routing (`/`, `/admin`, `/owner`) | `true` |
| `PUBLIC_SITE_ORIGIN` | Canonical public origin | `https://myplace-ds98.onrender.com` |
| `S3_ENDPOINT` | Supabase S3 endpoint URL | `https://<project-ref>.storage.supabase.co/storage/v1/s3` |
| `S3_BUCKET` | Supabase public bucket name | `myplace` |
| `AWS_ACCESS_KEY_ID` | Supabase S3 Access Key ID | Generated in Supabase Storage Settings |
| `AWS_SECRET_ACCESS_KEY` | Supabase S3 Secret Key | Generated in Supabase Storage Settings |
| `AWS_DEFAULT_REGION` | Supabase project region | `us-east-1` (or your project's region) |
| `ASSET_BASE_URL` | Browser-facing public base URL for assets | `https://<project-ref>.supabase.co/storage/v1/object/public/<bucket-name>` |
| `ADMIN_USERNAME` | Super-admin seed username | `admin` |
| `ADMIN_PASSWORD` | Super-admin seed password | *Secure random string* |

---

## 3. How to Obtain Supabase S3 Values

1. **`S3_ENDPOINT`**:
   * In Supabase, go to **Project Settings** → **Storage** → **S3 Access Keys** (or **Storage** → **Settings**).
   * URL format: `https://<project-ref>.storage.supabase.co/storage/v1/s3`
2. **`AWS_ACCESS_KEY_ID` & `AWS_SECRET_ACCESS_KEY`**:
   * Click **New Access Key** in the S3 Configuration section to generate your Access Key ID and Secret Access Key.
3. **`AWS_DEFAULT_REGION`**:
   * Found in **Project Settings** → **General** → **Region** (e.g. `us-east-1`, `eu-central-1`, `ap-southeast-1`).
4. **`ASSET_BASE_URL`**:
   * Base URL: `https://<project-ref>.supabase.co/storage/v1/object/public/<bucket-name>`
   * Example: `https://abcdefghijkl.supabase.co/storage/v1/object/public/myplace`
   * *Verify by uploading any test file in your public bucket and clicking "Copy URL".*

---

## 4. Render Deployment via Blueprint (`render.yaml`)

1. Commit and push the updated `render.yaml` to GitHub.
2. In the Render Dashboard, navigate to your Blueprint or create a new one pointing to your repo.
3. Render will prompt you for the uncommitted sync variables:
   * `S3_ENDPOINT`
   * `S3_BUCKET`
   * `AWS_ACCESS_KEY_ID`
   * `AWS_SECRET_ACCESS_KEY`
   * `AWS_DEFAULT_REGION`
   * `ASSET_BASE_URL`
   * `JWT_PRIVATE_KEY` & `JWT_PUBLIC_KEY`
   * `PUBLIC_SITE_ORIGIN`
4. Apply the Blueprint. Render will:
   * Build the project using native Rust runtime: `SQLX_OFFLINE=true cargo build --release`.
   * Run pre-deploy migrations and seed: `./target/release/migrate && ./target/release/seed`.
   * Launch the multiplexed server: `./target/release/myplace`.
