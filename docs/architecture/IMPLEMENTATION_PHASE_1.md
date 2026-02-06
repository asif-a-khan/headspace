# Implementation Phase 1: Foundation

## Goal

A working server-rendered CRM with authentication, multi-tenant isolation, and full CRUD for all core entities. No JavaScript required. Every page loads as complete HTML.

**After this phase:** Users can log in, navigate the sidebar, view/create/edit/delete leads, contacts, organizations, products, and configure settings. All server-rendered, all functional.

---

## Application Structure

```
headspace/
├── Cargo.toml
├── .env
├── migrations/
│   ├── main/                      # Shared tables (run once)
│   │   ├── 001_create_schema.sql          # CREATE SCHEMA IF NOT EXISTS main
│   │   ├── 002_create_companies.sql       # Tenant registry
│   │   ├── 003_create_super_roles.sql     # Super admin roles
│   │   ├── 004_create_super_admins.sql    # Super admin users (FK → super_roles)
│   │   └── 005_create_sessions.sql        # Session store
│   └── tenant/                    # Per-tenant tables (run once per tenant)
│       ├── 001_create_roles.sql
│       ├── 002_create_groups.sql
│       ├── 003_create_users.sql
│       ├── 004_create_user_groups.sql
│       ├── 005_create_pipelines.sql
│       ├── 006_create_leads.sql
│       ├── 007_create_persons.sql
│       ├── 008_create_organizations.sql
│       ├── 009_create_products.sql
│       ├── 010_create_activities.sql
│       └── ...
├── src/
│   ├── main.rs                    # Thin entry point (calls lib::run)
│   ├── lib.rs                     # Module tree + server startup
│   ├── config/
│   │   └── mod.rs                 # Environment configuration
│   ├── error/
│   │   └── mod.rs                 # Error types (thiserror + IntoResponse)
│   ├── db/
│   │   ├── mod.rs                 # Database struct (writer/reader pools)
│   │   ├── tenant.rs              # SET search_path, tenant resolution
│   │   └── migrate.rs             # Migration runner (main + per-tenant)
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── acl.rs                 # ACL permission tree definition
│   │   ├── bouncer.rs             # Permission checking (has_permission, allow, get_authorized_user_ids)
│   │   └── password.rs            # Argon2 hashing + verification
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── tenant.rs              # Extract tenant from subdomain
│   │   ├── auth.rs                # Session-based authentication guard + permission check
│   │   └── csrf.rs                # CSRF token generation + validation
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs                # User (with role_id, view_permission)
│   │   ├── role.rs                # Role (permission_type, permissions JSON)
│   │   ├── group.rs               # Group + user_groups
│   │   ├── lead.rs                # Lead, LeadStage, LeadPipeline
│   │   ├── person.rs              # Person (contact)
│   │   ├── organization.rs        # Organization
│   │   ├── product.rs             # Product
│   │   ├── activity.rs            # Activity (calls, meetings, notes)
│   │   ├── super_admin.rs         # SuperAdmin (main schema)
│   │   └── tenant.rs              # Company/tenant metadata
│   ├── handlers/                  # HTML route handlers (return Askama templates)
│   │   ├── mod.rs
│   │   ├── health.rs              # Health check endpoint
│   │   ├── auth.rs                # Login, logout, password reset
│   │   ├── dashboard.rs           # Dashboard page
│   │   ├── leads.rs               # Lead CRUD handlers
│   │   ├── persons.rs             # Person CRUD handlers
│   │   ├── organizations.rs       # Organization CRUD handlers
│   │   ├── products.rs            # Product CRUD handlers
│   │   ├── activities.rs          # Activity CRUD handlers
│   │   ├── settings/
│   │   │   ├── users.rs           # User management (assign role, groups, view_permission)
│   │   │   ├── roles.rs           # Role CRUD (permission_type, permission tree)
│   │   │   ├── groups.rs          # Group CRUD
│   │   │   └── pipelines.rs       # Pipeline configuration
│   │   └── super_admin/           # Super admin panel (main domain)
│   │       ├── auth.rs            # Super admin login/logout
│   │       ├── tenants.rs         # Tenant CRUD + provisioning
│   │       ├── agents.rs          # Super admin user management
│   │       └── roles.rs           # Super admin role management
│   ├── api/                       # JSON API handlers (return serde JSON, consumed by Vue)
│   │   ├── mod.rs
│   │   ├── leads.rs               # Lead API (kanban, CRUD, stage updates)
│   │   ├── activities.rs          # Activity API (calendar data)
│   │   ├── dashboard.rs           # Dashboard stats API (chart data)
│   │   ├── persons.rs             # Person search/autocomplete
│   │   ├── organizations.rs       # Organization search/autocomplete
│   │   ├── products.rs            # Product API
│   │   ├── tags.rs                # Tag search, attach, detach
│   │   └── quotes.rs              # Quote line items API
│   ├── routes/
│   │   └── mod.rs                 # All route definitions (HTML + API groups)
│   └── views/                     # Askama template structs (Rust side)
│       ├── mod.rs
│       ├── layouts.rs             # Layout template structs
│       ├── leads.rs               # Lead page template structs
│       ├── persons.rs
│       └── ...
├── frontend/                      # Vue.js source (built by Vite → static/)
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── main.ts                # Vue app bootstrap
│   │   ├── components/            # Vue components (kanban, calendar, etc.)
│   │   ├── stores/                # Pinia stores
│   │   └── composables/           # Shared Vue composables (api client, etc.)
│   └── ...
├── templates/                     # Askama template files (HTML side)
│   ├── layouts/
│   │   ├── base.html
│   │   ├── authenticated.html
│   │   └── anonymous.html
│   ├── components/
│   │   ├── breadcrumbs.html
│   │   ├── flash.html
│   │   └── avatar.html
│   ├── pages/
│   │   ├── auth/
│   │   │   ├── login.html
│   │   │   └── reset_password.html
│   │   ├── dashboard/
│   │   │   └── index.html
│   │   ├── leads/
│   │   │   ├── index.html         # Shell with Vue mount point + initial data
│   │   │   └── show.html
│   │   ├── contacts/
│   │   │   ├── persons/
│   │   │   └── organizations/
│   │   ├── products/
│   │   ├── activities/
│   │   └── settings/
│   │       ├── users/
│   │       ├── roles/
│   │       └── pipelines/
│   └── partials/
│       ├── header.html
│       ├── sidebar.html
│       └── footer.html
└── static/                        # Vite build output + static assets
    ├── dist/                      # Vite-built Vue bundles (gitignored)
    ├── css/
    │   └── app.css                # Compiled Tailwind
    └── images/
        └── favicon.svg
```

---

## Configuration

```rust
// src/config.rs
use dotenvy::dotenv;

pub struct Config {
    pub database_writer_url: String,
    pub database_reader_url: String,
    pub app_host: String,              // "headspace.local"
    pub app_port: u16,                 // 8000
    pub session_secret: String,        // 64+ char random string
    pub primary_domain: String,        // "headspace.local" (super admin)
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenv().ok(); // Load .env if present

        Ok(Self {
            database_writer_url: std::env::var("DATABASE_WRITER_URL")?,
            database_reader_url: std::env::var("DATABASE_READER_URL")?,
            app_host: std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            app_port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "8000".into())
                .parse()?,
            session_secret: std::env::var("SESSION_SECRET")?,
            primary_domain: std::env::var("PRIMARY_DOMAIN")?,
        })
    }
}
```

```bash
# .env
DATABASE_WRITER_URL=postgres:///headspace?host=/var/run/postgresql
DATABASE_READER_URL=postgres:///headspace?host=/var/run/postgresql
APP_HOST=0.0.0.0
APP_PORT=8000
SESSION_SECRET=<64-char-random-string>
PRIMARY_DOMAIN=headspace.local
```

---

## Database Layer

### Connection Pool

```rust
// src/db/mod.rs
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub struct Database {
    writer: PgPool,
    reader: PgPool,
}

impl Database {
    pub async fn connect(writer_url: &str, reader_url: &str) -> Result<Self, sqlx::Error> {
        let writer = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(writer_url)
            .await?;

        let reader = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(reader_url)
            .await?;

        Ok(Self { writer, reader })
    }

    pub fn writer(&self) -> &PgPool { &self.writer }
    pub fn reader(&self) -> &PgPool { &self.reader }
}
```

### Tenant Context

```rust
// src/db/tenant.rs
use sqlx::PgConnection;

/// Set the search_path for a connection to scope all queries to a tenant.
/// MUST be called on every connection checkout from the pool.
pub async fn set_tenant(conn: &mut PgConnection, schema: &str) -> Result<(), sqlx::Error> {
    // Validate schema name to prevent SQL injection (alphanumeric + underscore only)
    assert!(
        schema.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "Invalid schema name: {schema}"
    );

    sqlx::query(&format!("SET search_path TO {schema}, main"))
        .execute(&mut *conn)
        .await?;

    Ok(())
}
```

### Migrations

Migration strategy: write each migration once, applied everywhere automatically on startup.

**How it works:**

- `migrations/main/` — shared tables. Run once. Tracked by `main._sqlx_migrations`.
- `migrations/tenant/` — per-tenant tables. Run once per tenant. Each tenant tracks independently via `{tenant_schema}._sqlx_migrations`.
- `sqlx::migrate!().run()` is idempotent — it checks the `_sqlx_migrations` table and only runs what's new.
- `SET search_path` before running controls where tables AND the tracker land.

**Adding a new migration:**

```
You add: migrations/tenant/008_create_tags.sql

Server starts → run_all_migrations()
  ├── main schema: 001-004 already applied, nothing to run
  ├── tenant_asif: 001-007 applied, runs 008 ✓
  ├── tenant_acme: 001-007 applied, runs 008 ✓
  └── tenant_newco: 001-007 applied, runs 008 ✓
```

Adding main + tenant migrations at the same time works too — main always runs first, so tenant migrations can safely reference `main.*` tables.

**Schema layout:**

```
headspace_db
├── main
│   ├── _sqlx_migrations     ← tracks main/001 through main/005
│   ├── companies
│   ├── super_roles
│   ├── super_admins          (FK → super_roles.id)
│   └── sessions
├── tenant_asif
│   ├── _sqlx_migrations     ← tracks tenant/001 through tenant/010
│   ├── roles
│   ├── groups
│   ├── users                 (FK → roles.id)
│   ├── user_groups           (junction: users ↔ groups)
│   ├── pipelines
│   ├── leads
│   └── ...
└── tenant_acme
    ├── _sqlx_migrations     ← same migrations, tracked independently
    ├── roles
    ├── groups
    ├── users
    └── ...
```

```rust
// src/db/migrate.rs
use sqlx::PgPool;

/// Full migration flow — called on every application startup.
/// Safe to run repeatedly (idempotent).
pub async fn run_all_migrations(pool: &PgPool) -> Result<(), anyhow::Error> {
    // Step 1: Bootstrap the main schema.
    // This raw query runs BEFORE the migration tracker exists.
    sqlx::query("CREATE SCHEMA IF NOT EXISTS main")
        .execute(pool)
        .await?;

    // Step 2: Run main schema migrations.
    // SET search_path so _sqlx_migrations lands inside main.
    let mut conn = pool.acquire().await?;
    sqlx::query("SET search_path TO main")
        .execute(&mut *conn)
        .await?;

    sqlx::migrate!("migrations/main")
        .run(&mut *conn)
        .await?;

    // Step 3: Run tenant migrations for every existing tenant.
    let tenants: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, schema_name FROM main.companies WHERE schema_name IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;

    for (id, schema) in &tenants {
        tracing::info!(tenant_id = id, schema = %schema, "Migrating tenant");
        migrate_tenant(pool, schema).await?;
    }

    tracing::info!(tenant_count = tenants.len(), "All migrations complete");
    Ok(())
}

/// Run tenant migrations within a specific tenant schema.
/// Called on startup for existing tenants AND during provisioning for new tenants.
pub async fn migrate_tenant(pool: &PgPool, schema: &str) -> Result<(), anyhow::Error> {
    // Validate schema name (prevent SQL injection)
    anyhow::ensure!(
        schema.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "Invalid schema name: {schema}"
    );

    let mut conn = pool.acquire().await?;

    // Ensure tenant schema exists (no-op if already there)
    sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {schema}"))
        .execute(&mut *conn)
        .await?;

    // SET search_path so all tables + _sqlx_migrations go into tenant schema
    sqlx::query(&format!("SET search_path TO {schema}"))
        .execute(&mut *conn)
        .await?;

    // Run tenant migrations — each tenant tracks its own _sqlx_migrations
    sqlx::migrate!("migrations/tenant")
        .run(&mut *conn)
        .await?;

    Ok(())
}
```

---

## Multi-Tenant Middleware

```rust
// src/middleware/tenant.rs
use axum::{extract::Host, middleware::Next, response::Response, http::Request};

/// Tenant context extracted from the subdomain.
#[derive(Clone, Debug)]
pub struct Tenant {
    pub company_id: i64,
    pub schema_name: String,   // "tenant_asif"
    pub subdomain: String,     // "asif"
    pub domain: String,        // "asif.headspace.local"
}

/// Middleware: resolve tenant from subdomain, inject into request extensions.
pub async fn resolve_tenant<B>(
    Host(host): Host,
    db: Extension<Database>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let primary_domain = &req.extensions().get::<Config>()
        .expect("Config not in extensions")
        .primary_domain;

    // Strip port if present
    let host_without_port = host.split(':').next().unwrap_or(&host);

    // Extract subdomain: "asif.headspace.local" → "asif"
    let subdomain = host_without_port
        .strip_suffix(&format!(".{primary_domain}"))
        .ok_or(AppError::TenantNotFound)?;

    // Look up tenant in main.companies
    let tenant = sqlx::query_as::<_, TenantRow>(
        "SELECT id, schema_name, domain FROM main.companies WHERE domain = $1"
    )
    .bind(host_without_port)
    .fetch_optional(db.reader())
    .await?
    .ok_or(AppError::TenantNotFound)?;

    req.extensions_mut().insert(Tenant {
        company_id: tenant.id,
        schema_name: tenant.schema_name,
        subdomain: subdomain.to_string(),
        domain: host_without_port.to_string(),
    });

    Ok(next.run(req).await)
}
```

---

## Authentication

### Password Hashing

```rust
// src/auth/password.rs
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;

pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, anyhow::Error> {
    let parsed = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}
```

### Session Middleware

```rust
// In main.rs router setup
use tower_sessions::{SessionManagerLayer, PostgresStore};
use tower_sessions::cookie::SameSite;

let session_store = PostgresStore::new(db.writer().clone());
session_store.migrate().await?; // Creates session table if needed

let session_layer = SessionManagerLayer::new(session_store)
    .with_secure(false)            // true in production (HTTPS)
    .with_same_site(SameSite::Lax)
    .with_http_only(true);
```

### Auth Guard Middleware

```rust
// src/middleware/auth.rs
use tower_sessions::Session;

/// Middleware: require authenticated user, redirect to login if not.
pub async fn require_auth<B>(
    session: Session,
    tenant: Extension<Tenant>,
    db: Extension<Database>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, Response> {
    let user_id: Option<i64> = session.get("user_id").await
        .map_err(|_| Redirect::to("/login").into_response())?;

    let user_id = user_id
        .ok_or_else(|| Redirect::to("/login").into_response())?;

    // Fetch user from tenant schema
    let mut conn = db.reader().acquire().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    set_tenant(&mut conn, &tenant.schema_name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
        .ok_or_else(|| Redirect::to("/login").into_response())?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
```

---

## CSRF Protection

```rust
// src/middleware/csrf.rs
use rand::Rng;
use tower_sessions::Session;

const CSRF_TOKEN_KEY: &str = "csrf_token";

/// Generate or retrieve CSRF token for the current session.
pub async fn get_csrf_token(session: &Session) -> Result<String, anyhow::Error> {
    if let Some(token) = session.get::<String>(CSRF_TOKEN_KEY).await? {
        return Ok(token);
    }

    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    session.insert(CSRF_TOKEN_KEY, &token).await?;
    Ok(token)
}

/// Validate CSRF token from form submission.
pub async fn validate_csrf(session: &Session, submitted: &str) -> Result<bool, anyhow::Error> {
    let stored = session.get::<String>(CSRF_TOKEN_KEY).await?;
    Ok(stored.as_deref() == Some(submitted))
}
```

Every form includes a hidden field:
```html
<input type="hidden" name="_csrf" value="{{ csrf_token }}">
```

POST handlers validate the token before processing.

---

## Authorization (RBAC)

Three-layer authorization system mirroring Krayin's approach, adapted for schema-per-tenant isolation.

### Layer 1: Roles & Permissions

Each user has one **role**. A role either grants all permissions or a custom subset.

```sql
-- migrations/tenant/001_create_roles.sql
CREATE TABLE roles (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    permission_type TEXT NOT NULL DEFAULT 'custom',   -- 'all' or 'custom'
    permissions     JSONB NOT NULL DEFAULT '[]',      -- array of permission keys
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

- `permission_type = 'all'` → full access, `permissions` column ignored
- `permission_type = 'custom'` → only the keys in `permissions` are granted
- Permission keys use dot notation: `"leads.create"`, `"settings.user.roles.delete"`

### Layer 2: Groups (Data Scoping Teams)

Groups define teams for data visibility. They do NOT grant permissions — they control which records a user can see.

```sql
-- migrations/tenant/002_create_groups.sql
CREATE TABLE groups (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

```sql
-- migrations/tenant/004_create_user_groups.sql
CREATE TABLE user_groups (
    group_id        BIGINT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id         BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, user_id)
);
```

### Layer 3: View Permission (Data Visibility)

Each user has a `view_permission` controlling which records they see:

| Value | Behavior | SQL Effect |
|-------|----------|------------|
| `global` | Sees all records | No WHERE clause on `user_id` |
| `group` | Sees records from users in shared groups | `WHERE user_id IN (group members)` |
| `individual` | Sees only own records | `WHERE user_id = $current_user_id` |

```sql
-- migrations/tenant/003_create_users.sql
CREATE TABLE users (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    image           TEXT,
    status          BOOLEAN NOT NULL DEFAULT true,
    role_id         BIGINT NOT NULL REFERENCES roles(id),
    view_permission TEXT NOT NULL DEFAULT 'global',   -- 'global', 'group', 'individual'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Super Admin RBAC (Main Schema)

The super admin panel has its own parallel role system in the `main` schema:

```sql
-- migrations/main/003_create_super_roles.sql
CREATE TABLE main.super_roles (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    permission_type TEXT NOT NULL DEFAULT 'custom',
    permissions     JSONB NOT NULL DEFAULT '[]',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

```sql
-- migrations/main/004_create_super_admins.sql
CREATE TABLE main.super_admins (
    id              BIGSERIAL PRIMARY KEY,
    first_name      TEXT NOT NULL,
    last_name       TEXT NOT NULL,
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    image           TEXT,
    status          BOOLEAN NOT NULL DEFAULT true,
    role_id         BIGINT NOT NULL REFERENCES main.super_roles(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Super admins have no groups or view_permission — they operate on tenants, not tenant data.

### ACL Permission Tree

All available permissions defined in Rust code. Shared across all tenants — individual tenants control which permissions each role has, but the available set is the same.

```rust
// src/auth/acl.rs

/// A single permission node in the ACL tree.
pub struct AclItem {
    pub key: &'static str,         // "leads.create"
    pub name: &'static str,        // "Create Leads"
    pub children: &'static [AclItem],
}

/// Complete tenant ACL tree — every permission available to tenant users.
pub static TENANT_ACL: &[AclItem] = &[
    AclItem { key: "dashboard", name: "Dashboard", children: &[] },

    AclItem { key: "leads", name: "Leads", children: &[
        AclItem { key: "leads.create", name: "Create", children: &[] },
        AclItem { key: "leads.view", name: "View", children: &[] },
        AclItem { key: "leads.edit", name: "Edit", children: &[] },
        AclItem { key: "leads.delete", name: "Delete", children: &[] },
    ]},

    AclItem { key: "quotes", name: "Quotes", children: &[
        AclItem { key: "quotes.create", name: "Create", children: &[] },
        AclItem { key: "quotes.edit", name: "Edit", children: &[] },
        AclItem { key: "quotes.print", name: "Print", children: &[] },
        AclItem { key: "quotes.delete", name: "Delete", children: &[] },
    ]},

    AclItem { key: "mail", name: "Mail", children: &[
        AclItem { key: "mail.compose", name: "Compose", children: &[] },
        AclItem { key: "mail.view", name: "View", children: &[] },
        AclItem { key: "mail.edit", name: "Edit", children: &[] },
        AclItem { key: "mail.delete", name: "Delete", children: &[] },
    ]},

    AclItem { key: "activities", name: "Activities", children: &[
        AclItem { key: "activities.create", name: "Create", children: &[] },
        AclItem { key: "activities.edit", name: "Edit", children: &[] },
        AclItem { key: "activities.delete", name: "Delete", children: &[] },
    ]},

    AclItem { key: "contacts", name: "Contacts", children: &[
        AclItem { key: "contacts.persons", name: "Persons", children: &[
            AclItem { key: "contacts.persons.create", name: "Create", children: &[] },
            AclItem { key: "contacts.persons.view", name: "View", children: &[] },
            AclItem { key: "contacts.persons.edit", name: "Edit", children: &[] },
            AclItem { key: "contacts.persons.delete", name: "Delete", children: &[] },
        ]},
        AclItem { key: "contacts.organizations", name: "Organizations", children: &[
            AclItem { key: "contacts.organizations.create", name: "Create", children: &[] },
            AclItem { key: "contacts.organizations.edit", name: "Edit", children: &[] },
            AclItem { key: "contacts.organizations.delete", name: "Delete", children: &[] },
        ]},
    ]},

    AclItem { key: "products", name: "Products", children: &[
        AclItem { key: "products.create", name: "Create", children: &[] },
        AclItem { key: "products.view", name: "View", children: &[] },
        AclItem { key: "products.edit", name: "Edit", children: &[] },
        AclItem { key: "products.delete", name: "Delete", children: &[] },
    ]},

    AclItem { key: "settings", name: "Settings", children: &[
        AclItem { key: "settings.user", name: "Users & Permissions", children: &[
            AclItem { key: "settings.user.users", name: "Users", children: &[
                AclItem { key: "settings.user.users.create", name: "Create", children: &[] },
                AclItem { key: "settings.user.users.edit", name: "Edit", children: &[] },
                AclItem { key: "settings.user.users.delete", name: "Delete", children: &[] },
            ]},
            AclItem { key: "settings.user.roles", name: "Roles", children: &[
                AclItem { key: "settings.user.roles.create", name: "Create", children: &[] },
                AclItem { key: "settings.user.roles.edit", name: "Edit", children: &[] },
                AclItem { key: "settings.user.roles.delete", name: "Delete", children: &[] },
            ]},
            AclItem { key: "settings.user.groups", name: "Groups", children: &[
                AclItem { key: "settings.user.groups.create", name: "Create", children: &[] },
                AclItem { key: "settings.user.groups.edit", name: "Edit", children: &[] },
                AclItem { key: "settings.user.groups.delete", name: "Delete", children: &[] },
            ]},
        ]},
        AclItem { key: "settings.lead", name: "Lead Settings", children: &[
            AclItem { key: "settings.lead.pipelines", name: "Pipelines", children: &[
                AclItem { key: "settings.lead.pipelines.create", name: "Create", children: &[] },
                AclItem { key: "settings.lead.pipelines.edit", name: "Edit", children: &[] },
                AclItem { key: "settings.lead.pipelines.delete", name: "Delete", children: &[] },
            ]},
            AclItem { key: "settings.lead.sources", name: "Sources", children: &[
                AclItem { key: "settings.lead.sources.create", name: "Create", children: &[] },
                AclItem { key: "settings.lead.sources.edit", name: "Edit", children: &[] },
                AclItem { key: "settings.lead.sources.delete", name: "Delete", children: &[] },
            ]},
            AclItem { key: "settings.lead.types", name: "Types", children: &[
                AclItem { key: "settings.lead.types.create", name: "Create", children: &[] },
                AclItem { key: "settings.lead.types.edit", name: "Edit", children: &[] },
                AclItem { key: "settings.lead.types.delete", name: "Delete", children: &[] },
            ]},
        ]},
    ]},

    AclItem { key: "configuration", name: "Configuration", children: &[] },
];

/// Super admin ACL tree — permissions for the main admin panel.
pub static SUPER_ADMIN_ACL: &[AclItem] = &[
    AclItem { key: "tenants", name: "Tenants", children: &[
        AclItem { key: "tenants.create", name: "Create", children: &[] },
        AclItem { key: "tenants.edit", name: "Edit", children: &[] },
        AclItem { key: "tenants.delete", name: "Delete", children: &[] },
    ]},
    AclItem { key: "settings", name: "Settings", children: &[
        AclItem { key: "settings.agents", name: "Agents", children: &[
            AclItem { key: "settings.agents.create", name: "Create", children: &[] },
            AclItem { key: "settings.agents.edit", name: "Edit", children: &[] },
            AclItem { key: "settings.agents.delete", name: "Delete", children: &[] },
        ]},
        AclItem { key: "settings.roles", name: "Roles", children: &[
            AclItem { key: "settings.roles.create", name: "Create", children: &[] },
            AclItem { key: "settings.roles.edit", name: "Edit", children: &[] },
            AclItem { key: "settings.roles.delete", name: "Delete", children: &[] },
        ]},
    ]},
];

/// Build a route → permission key map for middleware.
/// Maps each handler route name to its required ACL key.
pub fn route_permission_map() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("admin.leads.index", "leads");
    map.insert("admin.leads.create", "leads.create");
    map.insert("admin.leads.store", "leads.create");
    map.insert("admin.leads.edit", "leads.edit");
    map.insert("admin.leads.update", "leads.edit");
    map.insert("admin.leads.destroy", "leads.delete");
    // ... every route maps to a permission key
    map
}
```

### Bouncer (Permission Checker)

```rust
// src/auth/bouncer.rs

use crate::models::user::User;
use crate::models::role::Role;

/// Check if a user has a specific permission.
pub fn has_permission(user: &User, role: &Role, permission: &str) -> bool {
    match role.permission_type.as_str() {
        "all" => true,
        "custom" => {
            role.permissions
                .as_array()
                .map(|perms| perms.iter().any(|p| p.as_str() == Some(permission)))
                .unwrap_or(false)
        }
        _ => false,
    }
}

/// Get the list of user IDs this user is authorized to see data for.
/// Returns None for global (no filter), Some(vec) for group/individual.
pub async fn get_authorized_user_ids(
    conn: &mut PgConnection,
    user: &User,
) -> Result<Option<Vec<i64>>, sqlx::Error> {
    match user.view_permission.as_str() {
        "global" => Ok(None), // No filtering — user sees everything

        "group" => {
            // Get all user IDs in any group this user belongs to
            let user_ids: Vec<i64> = sqlx::query_scalar(
                "SELECT DISTINCT ug2.user_id
                 FROM user_groups ug1
                 JOIN user_groups ug2 ON ug1.group_id = ug2.group_id
                 WHERE ug1.user_id = $1"
            )
            .bind(user.id)
            .fetch_all(&mut *conn)
            .await?;

            Ok(Some(user_ids))
        }

        "individual" | _ => Ok(Some(vec![user.id])), // Only own records
    }
}
```

### Auth Middleware with Permission Checking

The auth middleware now fetches the user's role and checks route permissions:

```rust
// src/middleware/auth.rs

/// Middleware: require authenticated user + check route permission.
pub async fn require_auth<B>(
    session: Session,
    tenant: Extension<Tenant>,
    db: Extension<Database>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, Response> {
    let user_id: Option<i64> = session.get("user_id").await
        .map_err(|_| Redirect::to("/login").into_response())?;

    let user_id = user_id
        .ok_or_else(|| Redirect::to("/login").into_response())?;

    // Fetch user WITH role in a single query
    let mut conn = db.reader().acquire().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    set_tenant(&mut conn, &tenant.schema_name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    let user = sqlx::query_as::<_, User>(
        "SELECT u.*, r.permission_type, r.permissions as role_permissions
         FROM users u
         JOIN roles r ON u.role_id = r.id
         WHERE u.id = $1 AND u.status = true"
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
    .ok_or_else(|| Redirect::to("/login").into_response())?;

    // Check route-level permission
    let route_name = extract_route_name(&req);
    if let Some(required_permission) = route_permission_map().get(route_name.as_str()) {
        if !has_permission(&user, required_permission) {
            return Err(StatusCode::FORBIDDEN.into_response());
        }
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
```

### Data Scoping in Queries

Every handler that lists records must respect `view_permission`. This is applied at the query level:

```rust
// src/handlers/leads.rs
pub async fn index(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Query(params): Query<LeadFilterParams>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = db.reader().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    // Get authorized user IDs for data scoping
    let authorized_ids = get_authorized_user_ids(&mut conn, &user).await?;

    let (leads, pagination) = query_leads(&mut conn, &params, authorized_ids.as_deref()).await?;
    // ...
}

/// Query leads with view-permission scoping.
async fn query_leads(
    conn: &mut PgConnection,
    params: &LeadFilterParams,
    authorized_user_ids: Option<&[i64]>,
) -> Result<(Vec<LeadRow>, Pagination), sqlx::Error> {
    let mut query = String::from(
        "SELECT l.*, p.name as person_name, o.name as org_name
         FROM leads l
         LEFT JOIN persons p ON l.person_id = p.id
         LEFT JOIN organizations o ON l.organization_id = o.id
         WHERE 1=1"
    );

    // Apply view permission scoping
    if let Some(user_ids) = authorized_user_ids {
        // user_ids is &[i64], bind as parameter
        query.push_str(" AND l.user_id = ANY($1)");
    }

    // ... add filters, sorting, pagination
}
```

**Every DataGrid-style listing must apply this pattern:** leads, persons, organizations, products, activities, quotes.

### Permission-Gated UI

Templates conditionally show elements based on permissions:

```html
{# templates/pages/leads/index.html #}
{% extends "layouts/authenticated.html" %}

{% block content %}
<div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-semibold">Leads</h1>

    {% if user.has_permission("leads.create") %}
    <a href="/admin/leads/create" class="btn btn-primary">Create Lead</a>
    {% endif %}
</div>

{% call data_table(leads, pagination, filters) %}
{% endblock %}
```

```html
{# templates/components/data_table_actions.html #}
{% if user.has_permission("leads.edit") %}
<a href="/admin/leads/{{ lead.id }}/edit" class="text-gray-500 hover:text-gray-700">Edit</a>
{% endif %}

{% if user.has_permission("leads.delete") %}
<form method="POST" action="/admin/leads/{{ lead.id }}/delete">
    <input type="hidden" name="_csrf" value="{{ csrf_token }}">
    <button type="submit" class="text-red-500 hover:text-red-700">Delete</button>
</form>
{% endif %}
```

Sidebar navigation is also gated:

```html
{# templates/partials/sidebar.html #}
<nav>
    {% if user.has_permission("dashboard") %}
    <a href="/admin/dashboard">Dashboard</a>
    {% endif %}

    {% if user.has_permission("leads") %}
    <a href="/admin/leads">Leads</a>
    {% endif %}

    {% if user.has_permission("contacts") %}
    <a href="/admin/contacts/persons">Contacts</a>
    {% endif %}

    {% if user.has_permission("settings") %}
    <a href="/admin/settings">Settings</a>
    {% endif %}
</nav>
```

### User Model with Permission Helper

```rust
// src/models/user.rs

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub status: bool,
    pub role_id: i64,
    pub view_permission: String,        // "global", "group", "individual"
    pub image: Option<String>,

    // Joined from roles table
    pub permission_type: String,        // "all" or "custom"
    pub role_permissions: serde_json::Value,  // JSON array of permission keys
}

impl User {
    /// Check if this user has a specific permission.
    /// Used in templates and handlers.
    pub fn has_permission(&self, permission: &str) -> bool {
        if self.permission_type == "all" {
            return true;
        }

        self.role_permissions
            .as_array()
            .map(|perms| perms.iter().any(|p| p.as_str() == Some(permission)))
            .unwrap_or(false)
    }
}
```

### Role Management Handlers

```rust
// src/handlers/settings/roles.rs

pub async fn store(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Form(input): Form<CreateRoleInput>,
) -> Result<impl IntoResponse, AppError> {
    validate_csrf(&session, &input.csrf).await?;

    let permissions = if input.permission_type == "all" {
        serde_json::json!([])
    } else {
        serde_json::json!(input.permissions)  // Vec<String> from checkbox tree
    };

    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    sqlx::query(
        "INSERT INTO roles (name, description, permission_type, permissions)
         VALUES ($1, $2, $3, $4)"
    )
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.permission_type)
    .bind(&permissions)
    .execute(&mut *conn)
    .await?;

    session.insert("flash", FlashMessage::success("Role created")).await?;
    Ok(Redirect::to("/admin/settings/roles"))
}

pub async fn destroy(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    Path(role_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    // Prevent deleting a role that has users assigned
    let user_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE role_id = $1"
    )
    .bind(role_id)
    .fetch_one(&mut *conn)
    .await?;

    if user_count > 0 {
        return Err(AppError::Validation(vec![
            ("role".into(), "Cannot delete role with assigned users".into())
        ]));
    }

    // Prevent deleting the last role
    let role_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles")
        .fetch_one(&mut *conn)
        .await?;

    if role_count <= 1 {
        return Err(AppError::Validation(vec![
            ("role".into(), "Cannot delete the last role".into())
        ]));
    }

    // Prevent deleting current user's own role
    if user.role_id == role_id {
        return Err(AppError::Validation(vec![
            ("role".into(), "Cannot delete your own role".into())
        ]));
    }

    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(role_id)
        .execute(&mut *conn)
        .await?;

    Ok(Redirect::to("/admin/settings/roles"))
}
```

### User Management Handler (Assigning Roles, Groups, View Permission)

```rust
// src/handlers/settings/users.rs

#[derive(Deserialize, Validate)]
pub struct CreateUserInput {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub role_id: i64,
    pub view_permission: String,       // "global", "group", "individual"
    pub groups: Vec<i64>,              // Group IDs
    pub status: Option<bool>,
    #[serde(rename = "_csrf")]
    pub csrf: String,
}

pub async fn store(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Form(input): Form<CreateUserInput>,
) -> Result<impl IntoResponse, AppError> {
    validate_csrf(&session, &input.csrf).await?;
    input.validate().map_err(|e| AppError::Validation(extract_errors(e)))?;

    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    let password_hash = hash_password(&input.password)?;

    let new_user_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (name, email, password_hash, role_id, view_permission, status)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id"
    )
    .bind(&input.name)
    .bind(&input.email)
    .bind(&password_hash)
    .bind(input.role_id)
    .bind(&input.view_permission)
    .bind(input.status.unwrap_or(true))
    .fetch_one(&mut *conn)
    .await?;

    // Sync group memberships
    for group_id in &input.groups {
        sqlx::query(
            "INSERT INTO user_groups (user_id, group_id) VALUES ($1, $2)"
        )
        .bind(new_user_id)
        .bind(group_id)
        .execute(&mut *conn)
        .await?;
    }

    session.insert("flash", FlashMessage::success("User created")).await?;
    Ok(Redirect::to("/admin/settings/users"))
}
```

### Account Self-Edit Protection

Users cannot change their own role or view_permission — this prevents privilege escalation:

```rust
// src/handlers/account.rs

pub async fn update(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Form(input): Form<UpdateAccountInput>,
) -> Result<impl IntoResponse, AppError> {
    // Only allow updating name, email, password, image
    // NEVER allow role_id or view_permission changes via self-edit
    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    let mut query = String::from("UPDATE users SET name = $1, email = $2");
    // ... update password only if provided, update image only if provided
    // Explicitly DO NOT include role_id or view_permission

    Ok(Redirect::to("/admin/account"))
}
```

---

## Tenant Provisioning

When a new tenant signs up, create their schema and seed default data:

```rust
// src/db/provision.rs

pub async fn provision_tenant(
    db: &Database,
    company_name: &str,
    subdomain: &str,
) -> Result<Company, anyhow::Error> {
    let schema_name = format!("tenant_{subdomain}");

    // Register in main.companies
    let company = sqlx::query_as::<_, Company>(
        "INSERT INTO main.companies (name, domain, schema_name, created_at)
         VALUES ($1, $2, $3, NOW())
         RETURNING *"
    )
    .bind(company_name)
    .bind(format!("{subdomain}.headspace.local"))
    .bind(&schema_name)
    .fetch_one(db.writer())
    .await?;

    // Create schema and run all tenant migrations
    migrate_tenant(db.writer(), &schema_name).await?;

    // Seed default data
    seed_tenant(db.writer(), &schema_name).await?;

    tracing::info!(
        company_id = company.id,
        schema = %schema_name,
        "Tenant provisioned"
    );

    Ok(company)
}

async fn seed_tenant(pool: &PgPool, schema: &str) -> Result<(), anyhow::Error> {
    let mut conn = pool.acquire().await?;
    sqlx::query(&format!("SET search_path TO {schema}, main"))
        .execute(&mut *conn)
        .await?;

    // Default Administrator role (permission_type = 'all')
    sqlx::query(
        "INSERT INTO roles (name, description, permission_type)
         VALUES ('Administrator', 'Full access to all CRM features', 'all')"
    )
    .execute(&mut *conn)
    .await?;

    // Default group
    sqlx::query(
        "INSERT INTO groups (name, description) VALUES ('Sales', 'Default sales team')"
    )
    .execute(&mut *conn)
    .await?;

    // Default pipeline
    sqlx::query(
        "INSERT INTO pipelines (name, is_default, created_at)
         VALUES ('Default Pipeline', true, NOW())"
    )
    .execute(&mut *conn)
    .await?;

    let pipeline_id: i64 = sqlx::query_scalar("SELECT id FROM pipelines LIMIT 1")
        .fetch_one(&mut *conn)
        .await?;

    // Default stages
    let stages = ["New", "Contacted", "Qualified", "Proposal", "Negotiation", "Won", "Lost"];
    for (i, name) in stages.iter().enumerate() {
        sqlx::query(
            "INSERT INTO pipeline_stages (pipeline_id, name, sort_order, created_at)
             VALUES ($1, $2, $3, NOW())"
        )
        .bind(pipeline_id)
        .bind(name)
        .bind(i as i32)
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}
```

**Deletion (GDPR compliance):**

```sql
-- One command removes all tenant data. No surgical DELETE across dozens of tables.
DROP SCHEMA tenant_departing CASCADE;
DELETE FROM main.companies WHERE schema_name = 'tenant_departing';
```

---

## Error Handling

```rust
// src/error.rs
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Tenant not found")]
    TenantNotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation failed")]
    Validation(Vec<(String, String)>),  // (field, message) pairs

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            AppError::TenantNotFound => (StatusCode::NOT_FOUND, "Tenant not found").into_response(),
            AppError::Unauthorized => Redirect::to("/login").into_response(),
            AppError::Forbidden(msg) => {
                // Render a 403 page or redirect with flash message
                (StatusCode::FORBIDDEN, msg).into_response()
            }
            AppError::Validation(errors) => {
                // Re-render form with errors (handled per-handler)
                (StatusCode::UNPROCESSABLE_ENTITY, "Validation failed").into_response()
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
        }
    }
}
```

---

## Askama Template Pattern

### Template Struct (Rust side)

```rust
// src/views/leads.rs
use askama::Template;

#[derive(Template)]
#[template(path = "pages/leads/index.html")]
pub struct LeadIndexTemplate {
    pub user: User,
    pub tenant: Tenant,
    pub leads: Vec<LeadRow>,
    pub pagination: Pagination,
    pub filters: LeadFilters,
    pub csrf_token: String,
    pub flash: Option<FlashMessage>,
}

#[derive(Template)]
#[template(path = "pages/leads/create.html")]
pub struct LeadCreateTemplate {
    pub user: User,
    pub tenant: Tenant,
    pub pipelines: Vec<Pipeline>,
    pub sources: Vec<LeadSource>,
    pub types: Vec<LeadType>,
    pub persons: Vec<PersonOption>,  // For dropdown
    pub csrf_token: String,
    pub errors: Vec<(String, String)>,
}
```

### Template File (HTML side)

```html
{# templates/pages/leads/index.html #}
{% extends "layouts/authenticated.html" %}

{% block title %}Leads{% endblock %}

{% block content %}
<div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-semibold text-gray-900 dark:text-white">Leads</h1>

    {% if user.has_permission("leads.create") %}
    <a href="/admin/leads/create" class="btn btn-primary">Create Lead</a>
    {% endif %}
</div>

{% include "components/flash.html" %}

{% call data_table(leads, pagination, filters) %}

{% endblock %}
```

### Handler

```rust
// src/handlers/leads.rs
pub async fn index(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Query(params): Query<LeadFilterParams>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = db.reader().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    let (leads, pagination) = query_leads(&mut conn, &params).await?;
    let csrf_token = get_csrf_token(&session).await?;
    let flash = session.remove::<FlashMessage>("flash").await?;

    Ok(LeadIndexTemplate {
        user,
        tenant,
        leads,
        pagination,
        filters: params.into(),
        csrf_token,
        flash,
    })
}

pub async fn store(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    session: Session,
    Form(input): Form<CreateLeadInput>,
) -> Result<impl IntoResponse, AppError> {
    // Validate CSRF
    validate_csrf(&session, &input.csrf).await?;

    // Validate input
    input.validate()
        .map_err(|e| AppError::Validation(extract_errors(e)))?;

    // Insert
    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    let lead = sqlx::query_as::<_, Lead>(
        "INSERT INTO leads (title, person_id, organization_id, pipeline_id, stage_id, lead_value, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING *"
    )
    .bind(&input.title)
    .bind(input.person_id)
    .bind(input.organization_id)
    .bind(input.pipeline_id)
    .bind(input.stage_id)
    .bind(input.lead_value)
    .bind(user.id)
    .fetch_one(&mut *conn)
    .await?;

    session.insert("flash", FlashMessage::success("Lead created successfully")).await?;
    Ok(Redirect::to(&format!("/admin/leads/{}", lead.id)))
}
```

---

## Router Setup

```rust
// src/main.rs
use axum::{Router, middleware};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize
    let config = Config::from_env()?;
    tracing_subscriber::fmt::init();
    let db = Database::connect(&config.database_writer_url, &config.database_reader_url).await?;

    // Run all migrations (main schema + all tenants, idempotent)
    migrate::run_all_migrations(db.writer()).await?;

    // Session store
    let session_store = PostgresStore::new(db.writer().clone());
    session_store.migrate().await?;
    let session_layer = SessionManagerLayer::new(session_store)
        .with_same_site(SameSite::Lax)
        .with_http_only(true);

    // Routes
    let auth_routes = Router::new()
        .route("/login", get(handlers::auth::login_form).post(handlers::auth::login))
        .route("/logout", post(handlers::auth::logout));

    let admin_routes = Router::new()
        .route("/dashboard", get(handlers::dashboard::index))
        .route("/leads", get(handlers::leads::index))
        .route("/leads/create", get(handlers::leads::create).post(handlers::leads::store))
        .route("/leads/:id", get(handlers::leads::show))
        .route("/leads/:id/edit", get(handlers::leads::edit).post(handlers::leads::update))
        .route("/leads/:id/delete", post(handlers::leads::destroy))
        // ... persons, organizations, products, activities, settings
        .layer(middleware::from_fn(middleware::auth::require_auth));

    let app = Router::new()
        .nest("/admin", admin_routes)
        .merge(auth_routes)
        .layer(middleware::from_fn(middleware::tenant::resolve_tenant))
        .layer(session_layer)
        .layer(Extension(db))
        .layer(Extension(config.clone()))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::services::ServeDir::new("static"));

    let addr = format!("{}:{}", config.app_host, config.app_port);
    tracing::info!("Starting server on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## Validation Pattern

```rust
// src/handlers/leads.rs
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateLeadInput {
    #[validate(length(min = 1, message = "Title is required"))]
    pub title: String,

    pub person_id: Option<i64>,
    pub organization_id: Option<i64>,

    #[validate(range(min = 1, message = "Pipeline is required"))]
    pub pipeline_id: i64,

    #[validate(range(min = 1, message = "Stage is required"))]
    pub stage_id: i64,

    #[validate(range(min = 0, message = "Value must be positive"))]
    pub lead_value: Option<f64>,

    #[serde(rename = "_csrf")]
    pub csrf: String,
}
```

On validation failure, the handler re-renders the form template with errors pre-populated:

```rust
if let Err(validation_errors) = input.validate() {
    let errors = extract_errors(validation_errors);
    return Ok(LeadCreateTemplate {
        errors,
        // ... re-populate form fields from input
    }.into_response());
}
```

---

## What Phase 1 Delivers

| Feature | Status |
|---------|--------|
| Multi-tenant isolation (schema-per-tenant) | Complete |
| Authentication (login, logout, sessions) | Complete |
| Password hashing (argon2) | Complete |
| CSRF protection | Complete |
| **RBAC — Roles with permission_type (all/custom)** | Complete |
| **RBAC — ACL permission tree (47 keys, dot notation)** | Complete |
| **RBAC — Groups (teams for data scoping)** | Complete |
| **RBAC — View permission (global/group/individual data visibility)** | Complete |
| **RBAC — Permission-gated middleware (route → permission mapping)** | Complete |
| **RBAC — Permission-gated UI (sidebar, action buttons, create buttons)** | Complete |
| **RBAC — Account self-edit protection (can't change own role)** | Complete |
| **Super admin panel (tenant CRUD, agent management, super roles)** | Complete |
| Lead CRUD (list, create, edit, delete) | Complete |
| Person CRUD | Complete |
| Organization CRUD | Complete |
| Product CRUD | Complete |
| Activity CRUD | Complete |
| Pipeline/stage configuration | Complete |
| User management (assign role, groups, view_permission) | Complete |
| Role management (create, permission tree, edit, delete with guards) | Complete |
| Group management (CRUD, user assignment) | Complete |
| Dashboard (server-rendered stats) | Complete |
| DataGrid (server-rendered, paginated, filterable, view-scoped) | Complete |
| Flash messages | Complete |
| Tailwind CSS styling | Complete |
| Responsive layout | Complete |
| Dark mode (CSS-only, `prefers-color-scheme`) | Complete |
| Request tracing/logging | Complete |

**Not in Phase 1:** Kanban board, calendar, charts, email, file uploads, data import/export. These require Vue.js components (Phase 2) or background tasks (Phase 3).
