# Headspace CRM 

A multi-tenant CRM built in Rust. Ships as a single binary that serves server-rendered HTML (Askama), interactive Vue.js components, and a JSON API. Multi-tenancy uses PostgreSQL schema-per-tenant isolation.

~39k lines of Rust, ~11k lines of Vue/TypeScript.

## Live Demo

Try the app without installing anything:

**[Live Demo](https://4bf8-38-62-45-51.ngrok-free.app/admin/login)**

| Email | Password |
|-------|----------|
| `admin@demo.headspace.local` | `admin123` |

> Note: This is a demo instance running on ngrok. The URL may change — check back here for the latest link.

## Features

**CRM Core**
- Leads with kanban board, pipeline stages, won/lost workflow, rotten lead tracking
- Contacts (persons & organizations) with detail pages, linked leads/activities
- Products with decimal pricing, search, catalog management
- Quotes with line-item builder, discount/tax, PDF generation
- Activities (calls, meetings, notes, lunches) with calendar view, file attachments, participants
- Tags across all entities (attach/detach)
- Dashboard with 8 stat cards, 4 charts (Chart.js), top products/persons, date filtering

**Email**
- SMTP outbound (compose, send, drafts, reply threading)
- IMAP inbound sync (background polling every 5 min, manual sync, dedup by message_id, attachment storage)
- Folder management (inbox, sent, drafts, trash)

**Settings & Admin**
- Custom attributes system (text, number, select, date, etc.)
- Pipeline & stage management with lead migration on delete
- User roles with granular ACL permissions
- Groups, sources, types, tags management
- Email templates, warehouses, web forms
- Tenant configuration (currency, date format, timezone, locale, brand color, SMTP/IMAP)

**Data**
- CSV import/export for leads, persons, organizations, products
- Mass delete operations on all major entities
- Global search across leads, persons, products, quotes, organizations
- Server-side pagination with search and sort

**Multi-Tenancy**
- Schema-per-tenant PostgreSQL isolation
- Subdomain-based tenant resolution
- Super admin panel for tenant, agent, and role management
- Per-tenant migrations and seeding

**Security**
- Server-side ACL bouncer on all API endpoints
- View permission filtering (own/group/all scoping)
- Server-side validation on all payloads
- CSRF protection
- Argon2 password hashing

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2024) |
| Runtime | Tokio |
| Web | Axum 0.8 + Tower |
| Templates | Askama 0.15 |
| Database | PostgreSQL + sqlx 0.8 (no ORM, compile-time checked SQL) |
| Auth | tower-sessions + argon2 |
| Frontend | Vue.js 3 + Vuetify 3 + Pinia + Vite |
| Email | lettre (SMTP) + async-imap (IMAP) + mail-parser |
| PDF | genpdfi |

## Prerequisites

- Rust (stable, edition 2024)
- PostgreSQL 16+
- Node.js 18+ and npm (for frontend build)

## Getting Started

```bash
# Clone the repo
git clone https://github.com/AKCodeWorks/headspace.git
cd headspace

# Create the database
createdb headspace

# Configure environment
cp .env.example .env   # Edit as needed

# Add local domains to /etc/hosts
echo "127.0.0.1 headspace.local" | sudo tee -a /etc/hosts
echo "127.0.0.1 demo.headspace.local" | sudo tee -a /etc/hosts

# Build frontend
cd frontend && npm install && npm run build && cd ..

# Run (migrations and seeding happen automatically on startup)
cargo run
```

**Default credentials** (created on first run):

| Panel | URL | Email | Password |
|-------|-----|-------|----------|
| Super admin | `headspace.local:8000/super/login` | `admin@headspace.local` | `admin123` |
| Tenant admin | `demo.headspace.local:8000/admin/login` | `admin@demo.headspace.local` | `admin123` |

## Docker

Run everything with one command — no Rust toolchain or PostgreSQL install needed:

```bash
cp .env.example .env  # Edit as needed
docker compose up --build -d
```

The app will be available at `http://localhost`. To expose it publicly via ngrok:

```bash
ngrok http 80
```

Set `FALLBACK_TENANT=demo` in `.env` so the ngrok URL resolves to the demo tenant automatically.

## Configuration

Environment variables (`.env`):

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_WRITER_URL` | required | PostgreSQL connection string (write pool) |
| `DATABASE_READER_URL` | required | PostgreSQL connection string (read pool) |
| `APP_HOST` | `0.0.0.0` | Bind address |
| `APP_PORT` | `8000` | Bind port |
| `SESSION_SECRET` | required | Secret string for session signing |
| `PRIMARY_DOMAIN` | required | Primary domain for super admin panel (e.g. `headspace.local`) |
| `FALLBACK_TENANT` | none | Fallback tenant domain when subdomain can't be resolved (for ngrok/tunnels) |
| `RUST_LOG` | `headspace=debug` | Log level filter |

## Development

The project includes a [`justfile`](https://github.com/casey/just) for common tasks:

```bash
just start          # Build frontend + run app
just dev            # Watch mode (cargo watch + vite watch)
just check          # Verify both Rust and frontend compile
just reset-db       # Drop and recreate the database
just build-frontend # Build Vue/Vite frontend only
```

## Project Structure

```
headspace/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # App bootstrap (config, DB, sessions, IMAP scheduler)
│   ├── api/                 # JSON API handlers (consumed by Vue components)
│   ├── auth/                # Password hashing, ACL bouncer
│   ├── config/              # Environment configuration
│   ├── db/                  # Connection pools, migrations, tenant guard, seeding
│   ├── error/               # AppError enum
│   ├── handlers/            # Page handlers (Askama HTML responses)
│   ├── imap/                # IMAP sync (connect, parse, sync, scheduler)
│   ├── middleware/          # Auth guards, CSRF, tenant resolution
│   ├── models/              # Domain structs (sqlx::FromRow)
│   ├── routes/              # All route definitions
│   └── views/               # Askama template structs
├── frontend/                # Vue.js 3 + Vuetify source
│   └── src/components/      # Vue SFCs (admin pages, settings, shared)
├── templates/               # Askama HTML files (layouts + pages)
├── migrations/
│   ├── main/                # Shared schema (tenants, super admin)
│   └── tenant/              # Per-tenant schema (36 migrations)
├── fonts/                   # NotoSans TTF for PDF generation
├── static/                  # Favicon, built frontend (dist/)
├── Cargo.toml
├── justfile
└── .env.example
```

## Architecture

**Hybrid Askama + Vue.js** (like Laravel Blade + Vue):

- Askama renders page shells (layout, sidebar, header, breadcrumbs, Vue mount point)
- Vue.js + Vuetify handles interactive content within those shells
- Pinia stores hydrate from `window.__INITIAL_DATA__` embedded by Askama
- Mirror routes: HTML routes serve the shell, JSON API routes serve data
- Vite builds Vue into `static/dist/` with code-splitting (dynamic imports)

**Multi-tenancy**: Each tenant gets a PostgreSQL schema (`tenant_{domain}`). The `main` schema holds the tenant registry, super admin users, and sessions. Tenant resolution happens in middleware via subdomain matching.

## Documentation

| Document | Description |
|----------|-------------|
| [Tech Stack](docs/STACK.md) | Complete crate list with justifications |
| [Design Pattern](docs/architecture/DESIGN_PATTERN.md) | Hybrid Askama + Vue.js architecture |
| [Phase 1: Foundation](docs/architecture/IMPLEMENTATION_PHASE_1.md) | Axum, Askama, auth, multi-tenancy, CRUD |
| [Phase 2: Interactivity](docs/architecture/IMPLEMENTATION_PHASE_2.md) | Vue + Vuetify + Pinia, kanban, dashboards |
| [Phase 3: Communication](docs/architecture/IMPLEMENTATION_PHASE_3.md) | Email, background jobs, file uploads, CSV |
| [DB Phase 1](docs/db_design/PHASE_1.md) | Single VPS deployment (0-50 tenants) |
| [DB Phase 2](docs/db_design/PHASE_2.md) | Separated managed DB (50-500 tenants) |
| [DB Phase 3](docs/db_design/PHASE_3.md) | Scaled DB with replicas (500+ tenants) |

## License

MIT
