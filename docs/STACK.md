# Headspace CRM - Tech Stack

## Language & Runtime

| Crate | Purpose | Why |
|-------|---------|-----|
| **Rust** (stable, edition 2024) | Application language | Memory safety, performance, single binary deployment |
| **tokio** | Async runtime | Industry standard, powers axum/sqlx/tower ecosystem |

---

## Web Framework & HTTP

| Crate | Purpose | Why |
|-------|---------|-----|
| **axum 0.8** | Web framework | Built on tower by tokio team, best ecosystem integration |
| **tower 0.5** | Middleware framework | Composable middleware (auth, tenant resolution, logging) |
| **tower-http 0.6** | HTTP middleware | CORS, compression, static file serving, request tracing — all in one crate |

**Note:** `hyper` is used internally by axum but is not a direct dependency.

**Axum 0.8 changes:** Path parameters use `{param}` syntax (not `:param`). Static file serving via `nest_service()`.

### tower-http Features Used

```toml
tower-http = { version = "0.6", features = [
    "cors",           # Cross-origin for API routes
    "compression-gzip", # Response compression
    "fs",             # Static file serving (/static/*)
    "trace",          # Request/response tracing
    "limit",          # Request body size limits
] }
```

---

## Templating

| Crate | Purpose | Why |
|-------|---------|-----|
| **askama 0.15** | Template engine | Compile-time checked, Jinja2 syntax, zero-cost rendering |
| **askama_web 0.15** | Axum integration | Templates implement `IntoResponse` via `WebTemplate` derive |

**Note:** `askama_axum` is deprecated as of askama 0.13. Use `askama_web` with the `axum-0.8` feature instead. Derive both `Template` and `WebTemplate` on template structs.

**Role of Askama:** Renders page shells (layout, sidebar, header, breadcrumbs) and embeds initial data for Vue components. Similar to Laravel's Blade templates. Vue handles the interactive content within the shell.

---

## Database

| Crate | Purpose | Why |
|-------|---------|-----|
| **sqlx 0.8** | Database driver + query checker | Compile-time SQL verification, async, built-in migrations, connection pooling |

**Why sqlx over sea-orm:** CRM queries involve complex joins (leads + pipeline + stage + person + organization), aggregate dashboard stats, and schema-per-tenant `SET search_path`. An ORM abstracts away the SQL control we need. sqlx gives compile-time checked raw SQL with zero overhead.

### sqlx Features

```toml
sqlx = { version = "0.8", features = [
    "runtime-tokio",  # Tokio async runtime
    "postgres",       # PostgreSQL driver
    "chrono",         # DateTime type mapping
    "uuid",           # UUID type mapping
    "migrate",        # Built-in migration runner
] }
```

---

## Serialization

| Crate | Purpose | Why |
|-------|---------|-----|
| **serde** | Serialization framework | Industry standard, derive macros for structs |
| **serde_json** | JSON serialization | API responses, initial data embedding, configuration parsing |

---

## Authentication & Sessions

| Crate | Purpose | Why |
|-------|---------|-----|
| **tower-sessions 0.15** | Session management | Tower-native, middleware-based, pluggable backends |
| **tower-sessions-sqlx-store 0.15** | PostgreSQL session store | Sessions in the same database, no Redis dependency in Phase 1 |
| **argon2 0.5** | Password hashing | Current best practice (winner of PHC), resistant to GPU/ASIC attacks |

**Why argon2 over bcrypt:** Argon2id is the modern recommendation (OWASP, PHC winner). It's memory-hard, making brute-force attacks significantly more expensive than bcrypt.

---

## Validation

| Crate | Purpose | Why |
|-------|---------|-----|
| **validator 0.18** | Struct validation | Derive-based (`#[validate(email)]`, `#[validate(length(min = 1))]`), maps to form errors |

---

## Error Handling

| Crate | Purpose | Why |
|-------|---------|-----|
| **thiserror 2** | Domain error types | Derive `Error` for structured errors (NotFound, Unauthorized, ValidationFailed) |
| **anyhow** | Application errors | Startup, migrations, scripts — where error type doesn't matter |

**Pattern:** `thiserror` for domain/HTTP errors that implement `IntoResponse`. `anyhow` for infrastructure code (migrations, CLI commands, startup).

---

## Logging & Observability

| Crate | Purpose | Why |
|-------|---------|-----|
| **tracing** | Structured logging + instrumentation | Spans, structured fields, async-aware |
| **tracing-subscriber** | Log output | Formatting, filtering, stdout/file output |

---

## Email (Phase 3)

| Crate | Purpose | Why |
|-------|---------|-----|
| **lettre** | SMTP email sending | Outbound emails (notifications, contact emails, password resets) |
| **async-imap** | IMAP email reading | Inbound email sync from user mailboxes (async, tokio-compatible) |
| **mail-parser** | Email parsing | MIME decoding, attachment extraction, HTML/text body parsing |

**Architecture:** Background tokio tasks poll connected mailboxes via IMAP. Parsed emails are stored in tenant schema and linked to contacts by email address matching. See `IMPLEMENTATION_PHASE_3.md` for details.

**Not in Cargo.toml yet** — added when Phase 3 begins.

---

## Configuration

| Crate | Purpose | Why |
|-------|---------|-----|
| **dotenvy** | .env file loading | Successor to `dotenv`, actively maintained |

**Pattern:** Load `.env` at startup, read into a typed config struct via `std::env::var()`. No config framework needed — a plain struct is simpler and more explicit.

---

## Utilities

| Crate | Purpose | Why |
|-------|---------|-----|
| **chrono** | Date/time handling | Best sqlx integration, ecosystem standard |
| **uuid** | UUID generation | Primary keys (tenant-safe, no sequential ID leaking between tenants) |
| **rand 0.9** | Secure random generation | CSRF tokens, password reset tokens, API keys |
| **time** | Duration/time types | Required by `tower-sessions` for session expiry configuration |

**Rand 0.9 API changes:** `thread_rng()` renamed to `rng()`, `distributions` module renamed to `distr`, `gen()` renamed to `random()` (due to `gen` being a reserved keyword in Rust 2024 edition).

---

## Frontend (Non-Rust)

| Tool | Purpose | Why |
|------|---------|-----|
| **Vue.js 3** | Interactive UI framework | Component-based, reactive data binding, works inside server-rendered page shells (like Blade + Vue in Laravel) |
| **Vuetify 3** | UI component library | Material Design components (data tables, forms, dialogs, date pickers) — avoids building CRM UI from scratch |
| **Pinia** | State management | Official Vue state management, hydrates from server-embedded `__INITIAL_DATA__`, persists across page interactions |
| **Tailwind CSS** | Base styling + page shell | Utility-first for Askama-rendered shells (sidebar, header, layout). Vuetify handles component styling |
| **Chart.js** | Dashboard charts | Simple charting for stats and reports |
| **SortableJS** | Drag-and-drop | Kanban board, quote line reordering (lightweight, no dependencies) |
| **Vite** | Build tooling | Required by Vuetify, fast HMR in dev, optimized production builds |

**No Node.js in production.** Vite runs at build time only. The output is static files served by the Rust binary.

**Architecture:** Askama renders page shells with Vue mount points and embedded initial data. Vite builds Vue components into optimized bundles served from `/static/`. See `DESIGN_PATTERN.md` for the full rendering strategy.

---

## What's Explicitly Excluded

| Technology | Why Excluded |
|-----------|-------------|
| **Redis** | PostgreSQL handles sessions and job state. Add only if Phase 3 demands it |
| **React/Angular** | Vue.js chosen for Vuetify's CRM-ready component library and familiarity |
| **GraphQL** | Internal API consumed by our own Vue components. REST is simpler and sufficient |
| **OpenAPI/Swagger** | API is not public-facing. Types are shared via Rust structs → JSON → Vue |
| **Docker** (for development) | `cargo run` is the dev server. Docker for deployment only |
| **ORM** | sqlx gives compile-time checked SQL without abstraction overhead |
| **Nuxt/SSR** | Askama handles server rendering. Vue only runs client-side within page shells |

---

## Cargo.toml Summary

```toml
[package]
name = "headspace"
version = "0.1.0"
edition = "2024"

[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
axum = { version = "0.8", features = ["multipart"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "fs", "trace", "limit"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate"] }

# Templates
askama = "0.15"
askama_web = { version = "0.15", features = ["axum-0.8"] }

# Auth & Sessions
tower-sessions = "0.15"
tower-sessions-sqlx-store = { version = "0.15", features = ["postgres"] }
argon2 = "0.5"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Error Handling
thiserror = "2"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Config
dotenvy = "0.15"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
rand = "0.9"
time = "0.3"
```
