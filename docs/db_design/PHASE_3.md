# Phase 3: Scaled Database with Replicas

## Overview

Multiple application servers behind a load balancer. Database split into primary (writes) and read replicas. Connection pooler (PgBouncer) manages database connections efficiently. Designed for high availability and horizontal scaling.

**Target:** 500+ tenants, thousands of concurrent users.
**Estimated cost:** ~$150-300/mo

---

## Architecture

```
                    ┌───────────────┐
                    │  DNS Wildcard  │
                    │ *.headspace.io │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │ Load Balancer  │
                    │ (Caddy/nginx)  │
                    └───┬───────┬───┘
                        │       │
              ┌─────────▼─┐ ┌──▼──────────┐
              │  App Srv 1 │ │  App Srv 2   │
              │  (Rust)    │ │  (Rust)      │
              └─────┬──────┘ └──────┬───────┘
                    │               │
                    └───────┬───────┘
                            │
                    ┌───────▼───────┐
                    │  PgBouncer    │
                    │  (Pooler)     │
                    └───┬───────┬───┘
                        │       │
              ┌─────────▼─┐ ┌──▼──────────┐
              │  Primary   │ │ Read Replica │
              │  (writes)  │ │ (reads)      │
              │            │─│ async repl.  │
              └────────────┘ └─────────────┘
```

## What Changes From Phase 2

| Aspect | Phase 2 | Phase 3 |
|--------|---------|---------|
| App servers | 1 | 2+ behind load balancer |
| DB connections | App → DB direct | App → PgBouncer → DB |
| Read path | Primary handles all | Replica handles reads |
| Write path | Primary | Primary (unchanged) |
| Availability | Single points of failure | Redundant at every layer |
| Failover | Manual | Automated (managed DB) |

## Database Connection Configuration

```bash
# Separate endpoints for reads and writes
DATABASE_WRITER_URL=postgres://headspace:pass@pgbouncer.host:6432/headspace_write?sslmode=require
DATABASE_READER_URL=postgres://headspace:pass@pgbouncer.host:6432/headspace_read?sslmode=require
```

PgBouncer routes `headspace_write` to primary and `headspace_read` to replica.

## Read/Write Split Implementation

The logical separation built in Phase 1 now becomes physical:

```rust
pub struct Database {
    writer: PgPool,  // → PgBouncer → Primary
    reader: PgPool,  // → PgBouncer → Replica
}

// Repository methods already use the correct pool
async fn get_leads(db: &Database, tenant: &Tenant) -> Result<Vec<Lead>> {
    let mut conn = db.reader().acquire().await?;
    set_tenant(&mut conn, tenant).await?;
    sqlx::query_as("SELECT * FROM leads")
        .fetch_all(&mut *conn)
        .await
}

async fn create_lead(db: &Database, tenant: &Tenant, data: NewLead) -> Result<Lead> {
    let mut conn = db.writer().acquire().await?;
    set_tenant(&mut conn, tenant).await?;
    sqlx::query_as("INSERT INTO leads (...) VALUES (...) RETURNING *")
        .fetch_one(&mut *conn)
        .await
}
```

## Replication Lag Handling

Async replication means reads from the replica may be 50-200ms behind the primary. This creates a read-your-own-writes problem:

```
User creates lead (write → primary)
  → 100ms replication lag
User redirected to lead list (read → replica)
  → New lead not visible yet
```

### Solution: Read-After-Write Routing

```rust
/// Middleware that checks for recent writes
/// If a write just happened, route reads to primary temporarily
async fn read_after_write_middleware(
    db: &Database,
    req: Request,
    next: Next,
) -> Response {
    // Check if this request follows a recent write
    let use_primary = req.cookie("_raw")  // "read-after-write" flag
        .map(|c| c.value() == "1")
        .unwrap_or(false);

    if use_primary {
        // Override reader to use writer for this request
        req.extensions_mut().insert(ReadFromPrimary);
    }

    next.run(req).await
}

/// Set the cookie after any write operation
async fn create_lead(...) -> Response {
    let lead = insert_lead(db.writer(), ...).await?;

    Response::redirect("/admin/leads")
        .with_cookie(
            Cookie::build("_raw", "1")
                .max_age(Duration::seconds(2))  // 2 seconds, covers replication lag
                .path("/")
                .finish()
        )
}
```

### What Reads Go Where

| Operation | Pool | Reason |
|-----------|------|--------|
| List pages (leads, contacts) | Reader | High volume, tolerates lag |
| Dashboard statistics | Reader | Aggregate queries, tolerates lag |
| Search/autocomplete | Reader | Read-only |
| Detail view after create | Writer | Read-your-own-writes |
| Detail view (normal) | Reader | Standard read |
| Form submission | Writer | Write operation |
| Data export | Reader | Heavy read, offload from primary |
| Migration scripts | Writer | Schema modifications |

## PgBouncer Configuration

```ini
[databases]
headspace_write = host=primary.db.host port=5432 dbname=headspace
headspace_read = host=replica.db.host port=5432 dbname=headspace

[pgbouncer]
listen_port = 6432
pool_mode = transaction          ; Release connection after each transaction
max_client_conn = 200            ; App servers can open many connections
default_pool_size = 20           ; Actual connections to Postgres
min_pool_size = 5                ; Keep warm connections
reserve_pool_size = 5            ; Emergency overflow
reserve_pool_timeout = 3         ; Seconds before using reserve
```

**Why transaction mode:** In session mode, the connection is held for the entire client session. In transaction mode, it's released after each transaction, allowing many app connections to share few database connections. Since we SET search_path per query batch (not per session), transaction mode works perfectly.

## Load Balancer Configuration

```
# Caddy example for wildcard subdomain routing
*.headspace.io {
    reverse_proxy app-server-1:8080 app-server-2:8080 {
        health_uri /health
        health_interval 10s
        lb_policy round_robin
    }
}
```

Requirements for stateless app servers:
- No in-memory session state (use DB-backed or cookie sessions)
- No local file storage (use S3-compatible object storage)
- Tenant context derived from request (subdomain), not server state
- All app servers connect to same PgBouncer endpoint

## Connection Pool Sizing

```
                    App connections    PgBouncer    DB connections
App Server 1:      10 connections ─┐
                                   ├─► 200 client  ─► 20 to Primary
App Server 2:      10 connections ─┘    slots         20 to Replica

Total: 20 app connections share 40 actual database connections
PgBouncer handles the multiplexing
```

**Rule of thumb:** Each Postgres connection uses ~5-10MB RAM. A database with 4GB RAM should have no more than ~100 connections. PgBouncer lets hundreds of app connections share those 100.

## Multi-Tenancy at Scale

With 500+ tenant schemas:

**Schema creation** remains fast:
```sql
CREATE SCHEMA tenant_new;
SET search_path TO tenant_new, main;
-- Run table migrations (~20 tables)
-- Complete in <1 second
```

**Migrations** should be parallelized:
```rust
// Run 10 tenant migrations concurrently
let semaphore = Arc::new(Semaphore::new(10));
for tenant in tenants {
    let permit = semaphore.clone().acquire_owned().await?;
    tokio::spawn(async move {
        migrate_tenant(&pool, &tenant).await;
        drop(permit);
    });
}
```

**Monitoring per-tenant query performance:**
```sql
-- Identify slow tenants
SELECT schemaname, count(*), avg(total_exec_time)
FROM pg_stat_statements
GROUP BY schemaname
ORDER BY avg(total_exec_time) DESC;
```

## Failover and High Availability

**Managed database failover:**
- Provider promotes replica to primary automatically on failure
- DNS endpoint updates within 30-60 seconds
- Application reconnects via pool (automatic retry)

**App server failover:**
- Load balancer health checks detect down servers
- Traffic routed to healthy servers
- No state to lose (stateless app)

**Recovery time objectives:**
| Component | Failure | Recovery |
|-----------|---------|----------|
| App server | Process crash | 1-2 seconds (load balancer reroutes) |
| App server | VM dies | 30-60 seconds (LB removes from pool) |
| DB replica | Crashes | No user impact (reads fall back to primary) |
| DB primary | Crashes | 30-60 seconds (managed failover) |

## Cost Breakdown

| Component | Provider | Estimated Cost |
|-----------|----------|---------------|
| App server 1 | Hetzner CX22 | $5/mo |
| App server 2 | Hetzner CX22 | $5/mo |
| Load balancer | Hetzner LB | $6/mo |
| Managed DB (primary) | Neon Pro / RDS | $50-100/mo |
| Read replica | Provider add-on | $30-50/mo |
| Object storage (backups) | S3-compatible | $5/mo |
| **Total** | | **$100-170/mo** |

## Beyond Phase 3

If growth continues beyond what a single primary + replica can handle:

- **Tenant sharding:** Route large tenants to dedicated database instances
- **Citus extension:** Distribute tables across multiple Postgres nodes
- **Read replica scaling:** Add more replicas for read-heavy workloads
- **Geographic distribution:** Deploy app + replica pairs in multiple regions

These are Phase 4+ concerns. Phase 3 handles thousands of tenants and tens of thousands of concurrent users.
