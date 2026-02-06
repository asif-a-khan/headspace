# Headspace CRM

A multi-tenant CRM SaaS built in Rust. Single binary serves everything — server-rendered HTML pages via Askama templates, interactive Vue.js components for complex UI, and a JSON API. Multi-tenancy is achieved through PostgreSQL schema-per-tenant isolation.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2024) |
| Runtime | Tokio |
| Web | Axum 0.8 + Tower |
| Templates | Askama 0.15 (compile-time, Jinja2 syntax) |
| Database | PostgreSQL + sqlx 0.8 (no ORM) |
| Auth | tower-sessions + argon2 |
| Frontend | Tailwind CSS, Vue.js 3 + Vuetify 3 + Pinia (Phase 2) |
| Build | Vite (Phase 2) |

## Documentation

### Architecture

| Document | Description |
|----------|-------------|
| [Tech Stack](docs/STACK.md) | Complete crate list with justifications for every dependency |
| [Design Pattern](docs/architecture/DESIGN_PATTERN.md) | Hybrid Askama + Vue.js architecture — how server-rendered shells and interactive components work together |

### Implementation Phases

| Phase | Document | Description |
|-------|----------|-------------|
| 1 | [Foundation](docs/architecture/IMPLEMENTATION_PHASE_1.md) | Axum server, Askama templates, authentication, multi-tenant isolation, full CRUD for all entities |
| 2 | [Interactivity](docs/architecture/IMPLEMENTATION_PHASE_2.md) | Vue.js 3 + Vuetify + Pinia integration, JSON API routes, kanban boards, calendars, dashboards, data tables |
| 3 | [Communication](docs/architecture/IMPLEMENTATION_PHASE_3.md) | IMAP/SMTP email integration, background task processing, file uploads, data import/export |

### Database Scaling

| Phase | Document | Description |
|-------|----------|-------------|
| 1 | [Single VPS](docs/db_design/PHASE_1.md) | Co-located Rust + PostgreSQL on one server (0–50 tenants) |
| 2 | [Managed DB](docs/db_design/PHASE_2.md) | Separated app server and managed PostgreSQL (50–500 tenants) |
| 3 | [Scaled DB](docs/db_design/PHASE_3.md) | Read replicas, connection pooling, load balancing (500+ tenants) |

### Reference

| Document | Description |
|----------|-------------|
| [System Overview](docs/PREV/PREV_OVERVIEW.md) | Entity relationships and business flows for the CRM domain |
| [Architecture Analysis](docs/PREV/PREV_ARCHITECTURE.md) | Exhaustive analysis of the original system — database schema, routing, auth, business logic, frontend |

## Project Structure

```
headspace/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # App bootstrap (config, DB, server)
│   ├── api/                  # JSON API handlers (consumed by Vue components)
│   ├── auth/                 # Password hashing, ACL definitions, bouncer
│   ├── config/               # Environment configuration
│   ├── db/                   # Connection pools, migrations, tenant resolution
│   ├── error/                # AppError enum, IntoResponse impl
│   ├── handlers/             # HTTP request handlers (HTML responses)
│   ├── middleware/            # Auth guards, CSRF, tenant resolution
│   ├── models/               # Domain structs (sqlx::FromRow)
│   ├── routes/               # Route definitions grouped by middleware
│   └── views/                # Askama template structs
├── templates/                # Askama HTML files (layouts, components, pages)
├── migrations/
│   ├── main/                 # Shared schema (tenants, super admin, sessions)
│   └── tenant/               # Per-tenant schema (leads, contacts, etc.)
├── static/                   # CSS, JS, images
├── Cargo.toml
└── .env
```

## Prerequisites

- Rust (stable, edition 2024)
- PostgreSQL 16+

## Getting Started

```bash
# Clone and enter the project
cd headspace

# Create the database
createdb headspace

# Configure environment
cp .env.example .env   # Edit as needed

# Run (migrations execute automatically on startup)
cargo run
```

The server starts on `http://0.0.0.0:8000` by default. Configuration is read from `.env`:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_WRITER_URL` | — | PostgreSQL connection string (write pool) |
| `DATABASE_READER_URL` | — | PostgreSQL connection string (read pool) |
| `APP_HOST` | `0.0.0.0` | Bind address |
| `APP_PORT` | `8000` | Bind port |
| `SESSION_SECRET` | — | 64-char hex string for session signing |
| `PRIMARY_DOMAIN` | — | Primary domain for super admin panel (e.g. `headspace.local`) |

## Multi-Tenancy

Each tenant gets its own PostgreSQL schema (`tenant_{domain}`). The `main` schema holds shared tables: tenant registry, super admin users, roles, and sessions. Tenant resolution happens via subdomain — `acme.headspace.local` resolves to schema `tenant_acme`.

The super admin panel runs on the primary domain (`headspace.local/super/`) and manages tenants, agents, and roles. Tenant CRM panels run on subdomains.

## License

Proprietary.
