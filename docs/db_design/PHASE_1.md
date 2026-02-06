# Phase 1: Single VPS Deployment

## Overview

Everything runs on a single VPS. Rust binary and PostgreSQL co-located on the same machine. Optimized for simplicity, low cost, and fast iteration during product-market fit.

**Target:** 0-50 tenants, proving the product.
**Estimated cost:** ~$10-20/mo

---

## Architecture

```
┌──────────────────────────────────┐
│          Single VPS              │
│      (Hetzner CX22, $5-15)      │
│                                  │
│   ┌────────┐    ┌──────────┐    │
│   │  Rust  │    │ Postgres │    │
│   │  App   │────│ (local)  │    │
│   │        │unix│          │    │
│   └────────┘sock└──────────┘    │
│                                  │
│   ┌────────────────────────┐    │
│   │  Caddy (reverse proxy) │    │
│   │  TLS + wildcard certs  │    │
│   └────────────────────────┘    │
│                                  │
│   Shared CPU, RAM, Disk          │
└──────────────────────────────────┘
```

## Database Connection

```
DATABASE_URL=postgres:///headspace?host=/var/run/postgresql
```

Unix socket connection. Zero network latency between app and database.

## Multi-Tenancy Model

Schema-per-tenant within a single PostgreSQL instance:

```
headspace (database)
├── main                (shared: countries, states, super_admins, companies)
├── tenant_asif         (schema: leads, persons, quotes, users, ...)
├── tenant_bob          (schema: leads, persons, quotes, users, ...)
└── tenant_carol        (schema: leads, persons, quotes, users, ...)
```

Each request sets the search_path on its connection:

```rust
sqlx::query("SET search_path TO $1, main")
    .bind(&tenant.schema_name)
    .execute(&mut *conn)
    .await?;
```

## Connection Pooling

Single pool, shared across all tenants:

```rust
let pool = PgPoolOptions::new()
    .max_connections(10)  // (CPU cores * 2) + 1
    .connect(&database_url)
    .await?;
```

**Critical:** search_path MUST be set on every connection checkout. Connections are shared across tenants in the pool.

## Read/Write Separation (Logical, Not Physical)

Even though both point to the same database in Phase 1, separate them in code to make Phase 3 trivial:

```rust
pub struct Database {
    writer: PgPool,
    reader: PgPool,  // Same connection string as writer in Phase 1
}

impl Database {
    pub fn reader(&self) -> &PgPool { &self.reader }
    pub fn writer(&self) -> &PgPool { &self.writer }
}
```

## Backup Strategy

Automated daily pg_dump to S3-compatible object storage:

```bash
# Cron job (daily at 2am)
pg_dump -Fc headspace | \
  aws s3 cp - s3://headspace-backups/$(date +%Y-%m-%d).dump \
  --endpoint-url https://s3.hetzner.com
```

Retention: 30 daily backups. Cost: ~$0.50/mo for a small database.

**Per-tenant backup** (enabled by schema-per-tenant):

```bash
pg_dump -n tenant_asif headspace > tenant_asif_backup.sql
```

## Tenant Lifecycle

**Creation:**
```rust
// Creates schema, runs all tenant migrations, seeds default data
provision_tenant(&db, "Acme Corp", "acme").await?;
```

This calls `migrate_tenant()` internally, which:
1. `CREATE SCHEMA IF NOT EXISTS tenant_acme`
2. `SET search_path TO tenant_acme` (so `_sqlx_migrations` tracker lands in tenant schema)
3. Runs all `migrations/tenant/*.sql` files
4. Seeds default pipeline, stages, and other initial data

**Deletion (GDPR compliance):**
```sql
DROP SCHEMA tenant_departing CASCADE;
DELETE FROM main.companies WHERE schema_name = 'tenant_departing';
-- One command drops all tenant data. No surgical DELETE across dozens of tables.
```

## Migrations

Single entry point on every startup — `run_all_migrations()`:

```
Server starts → run_all_migrations()
  1. CREATE SCHEMA IF NOT EXISTS main     ← bootstrap (no-op after first run)
  2. Run migrations/main/*.sql            ← tracked by main._sqlx_migrations
  3. For each tenant:                     ← tracked by {tenant}._sqlx_migrations
     └── Run migrations/tenant/*.sql
```

Each schema has its own `_sqlx_migrations` table. `sqlx::migrate!().run()` checks what's already applied and only runs what's new. Write a migration once, it applies to all tenants automatically.

```rust
// Called on every startup — idempotent, safe to run repeatedly
pub async fn run_all_migrations(pool: &PgPool) -> Result<(), anyhow::Error> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS main").execute(pool).await?;

    // Main schema migrations
    let mut conn = pool.acquire().await?;
    sqlx::query("SET search_path TO main").execute(&mut *conn).await?;
    sqlx::migrate!("migrations/main").run(&mut *conn).await?;

    // All tenant migrations (sequential, completes in seconds with <50 tenants)
    let tenants: Vec<String> = sqlx::query_scalar(
        "SELECT schema_name FROM main.companies WHERE schema_name IS NOT NULL"
    )
    .fetch_all(pool).await?;

    for schema in &tenants {
        migrate_tenant(pool, schema).await?;
    }
    Ok(())
}
```

With <50 tenants, the full migration run completes in seconds. See `db_design/PHASE_2.md` for parallelized migrations at scale.

## Monitoring

Minimal but sufficient:
- PostgreSQL `pg_stat_activity` for connection counts
- Disk usage alerts (df)
- Application health endpoint (`GET /health`)
- Log aggregation to stdout (journald captures it)

## Scaling Limits

This setup handles more than you'd expect:
- PostgreSQL on a 4GB VPS comfortably manages millions of rows
- Rust binary uses ~10-20MB RAM, leaving most memory for Postgres
- 50 concurrent users with sub-100ms response times

**When to move to Phase 2:**
- Database size exceeds 50% of disk
- You want automated point-in-time recovery
- You need to resize database independently of app
- You have paying customers and can't afford downtime for VPS maintenance

## Transition to Phase 2

**Effort:** Config change + 30 minutes of downtime.

```bash
# 1. Put app in maintenance mode
# 2. Final backup
pg_dump -Fc headspace > final_backup.dump
# 3. Restore to managed instance
pg_restore -h managed-db.host.com -d headspace final_backup.dump
# 4. Update DATABASE_URL in config
# 5. Restart app
# 6. Exit maintenance mode
```

Zero application code changes required.
