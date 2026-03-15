# Contributing to Headspace CRM

Thanks for your interest in contributing! This guide covers the development workflow, code conventions, and how to submit changes.

## Getting Set Up

### Prerequisites

- Rust stable (edition 2024)
- PostgreSQL 16+
- Node.js 18+ and npm
- [just](https://github.com/casey/just) (optional, for task runner)

### Local Development

```bash
# Clone and enter
git clone https://github.com/AKCodeWorks/headspace.git
cd headspace

# Create database
createdb headspace

# Configure environment
cp .env.example .env

# Add local domains
echo "127.0.0.1 headspace.local" | sudo tee -a /etc/hosts
echo "127.0.0.1 demo.headspace.local" | sudo tee -a /etc/hosts

# Install frontend deps and build
cd frontend && npm install && npm run build && cd ..

# Run (auto-migrates and seeds)
cargo run
```

### Watch Mode

```bash
# Requires cargo-watch: cargo install cargo-watch
just dev
```

This runs `cargo watch` and `vite watch` in parallel for hot-reloading.

## Code Organization

### Backend (Rust)

| Directory | Purpose |
|-----------|---------|
| `src/api/` | JSON API handlers — return `Json<>` responses, consumed by Vue |
| `src/handlers/` | Page handlers — return Askama HTML responses with `__INITIAL_DATA__` |
| `src/middleware/` | Auth guards (`require_super_admin`, `require_tenant_admin`), CSRF, tenant resolution |
| `src/db/guard.rs` | `TenantGuard` — safe tenant-scoped DB connection with automatic cleanup |
| `src/auth/bouncer.rs` | ACL permission checking |
| `src/routes/mod.rs` | All route definitions in one place |

### Frontend (Vue)

| Directory | Purpose |
|-----------|---------|
| `frontend/src/components/admin/` | Tenant admin page components |
| `frontend/src/components/admin/settings/` | Settings page components |
| `frontend/src/components/admin/shared/` | Reusable components |
| `frontend/src/components/super/` | Super admin page components |
| `frontend/src/main.ts` | App entry point, component registration, Vuetify setup |

### Templates (Askama)

| Directory | Purpose |
|-----------|---------|
| `templates/layouts/` | Base layouts (super admin, tenant admin) |
| `templates/pages/` | Page templates that extend layouts |

## Conventions

### Rust

- **No ORM** — raw SQL via sqlx with `query_as::<_, StructName>()`. Keep queries close to handlers.
- **TenantGuard** — always use `TenantGuard::acquire()` for tenant DB access. Always call `.release()` when done.
- **ACL** — every API handler must call `bouncer(&user, "permission.key")` before any logic.
- **Validation** — use the `validator` crate on payload structs. Call `validate_payload()` after bouncer.
- **Error responses** — return `(StatusCode, Json(...)).into_response()`. Use `internal_error()` helper for 500s.
- **Naming** — snake_case for Rust, `type` column uses `#[sqlx(rename = "type")]` since it's reserved.

### Vue / Frontend

- **Vuetify 3** — use Vuetify components exclusively (v-btn, v-card, v-text-field, etc.).
- **Code-splitting** — page components use dynamic imports in `main.ts`: `() => import("./path/Component.vue")`.
- **Data hydration** — read `window.__INITIAL_DATA__` for server-provided data, fetch API for subsequent updates.
- **CSRF** — include `X-CSRF-TOKEN` header from `<meta name="csrf-token">` on all mutating requests.
- **No Pinia stores for simple pages** — use local `ref`/`reactive` state. Pinia is for cross-component state only.

### Database

- **Migrations** — add new files in `migrations/tenant/` with the naming pattern `YYYYMMDDHHMMSS_description.sql`. Use `IF NOT EXISTS` / `IF EXISTS` for idempotency.
- **Schema isolation** — never hardcode schema names. The `TenantGuard` sets `search_path` automatically.
- **Config** — tenant settings use the `tenant_config` key-value table. Add new keys with `ON CONFLICT DO NOTHING` in migrations.

## Pull Request Process

1. **Fork and branch** — create a feature branch from `main`.
2. **Keep it focused** — one feature or fix per PR. Small PRs are reviewed faster.
3. **Verify builds** — run `just check` (or `cargo build && cd frontend && npm run build`) before pushing.
4. **Write a clear description** — explain what changed and why. Include screenshots for UI changes.
5. **No generated files** — don't commit `static/dist/`, `target/`, or `node_modules/`.

### Commit Messages

Use concise, descriptive messages:

```
Add IMAP inbound email sync with background polling
Fix pipeline stage deletion not migrating leads
Update dashboard chart colors for dark mode
```

## Architecture Notes

### Adding a New Entity

1. **Migration** — create `migrations/tenant/NNNN_create_things.sql`
2. **API handler** — `src/api/tenant_admin/things.rs` (list, show, store, update, destroy)
3. **Page handler** — `src/handlers/tenant_admin/things.rs` (index, create, edit)
4. **Routes** — add HTML + API routes in `src/routes/mod.rs`
5. **Vue component** — `frontend/src/components/admin/ThingList.vue`, `ThingForm.vue`
6. **Template** — `templates/pages/tenant_admin/things/*.html`
7. **Register** — add dynamic import in `frontend/src/main.ts`

### The TenantGuard Pattern

```rust
let mut guard = TenantGuard::acquire(db.reader(), &company.schema_name).await?;
let items = guard.fetch_all(sqlx::query_as::<_, Item>("SELECT * FROM items")).await?;
let _ = guard.release().await;
```

Always release the guard. If you don't, the connection is detached (closed) on drop to prevent returning a connection with a stale `search_path` to the pool.

### HRTB / Send Gotcha

`sqlx::Migrator::run()` and `PoolConnection` held across `.await` produce `!Send` futures. Axum handlers require `Send`. Use the `detach()` pattern for background tasks:

```rust
let pool_conn = pool.acquire().await?;
let mut conn = pool_conn.detach(); // Owned PgConnection, no HRTB issues
```

## Questions?

Open an issue on GitHub for questions, bugs, or feature requests.
