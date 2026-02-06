# Phase 2: Separated Managed Database

## Overview

Application and database are separated. App runs on a VPS or container. Database runs on a managed PostgreSQL service with automated backups, point-in-time recovery, and monitoring.

**Target:** 50-500 tenants, active growth, revenue flowing.
**Estimated cost:** ~$50-70/mo

---

## Architecture

```
┌──────────────────┐         ┌──────────────────┐
│   App Server     │   VPC   │   Managed DB      │
│   (VPS/Container)│         │   (Neon/Supabase/ │
│                  │         │    RDS/Cloud SQL)  │
│   ┌────────┐    │  ~1ms   │   ┌──────────┐   │
│   │  Rust  │────┼─────────┼──▶│ Postgres │   │
│   │  App   │    │   TLS   │   │          │   │
│   └────────┘    │         │   └──────────┘   │
│                  │         │                   │
│   ┌────────┐    │         │   Automated:      │
│   │ Caddy  │    │         │   - Backups       │
│   └────────┘    │         │   - PITR          │
│                  │         │   - Monitoring    │
└──────────────────┘         └──────────────────┘
     Scale: CPU                  Scale: RAM/IO
     Horizontally                Vertically
```

## What Changes From Phase 1

| Aspect | Phase 1 | Phase 2 |
|--------|---------|---------|
| Connection | Unix socket (~0ms) | TCP over VPC (~1ms) |
| Connection string | `postgres:///headspace?host=/var/run/postgresql` | `postgres://user:pass@db.host.com:5432/headspace?sslmode=require` |
| Backups | Manual pg_dump cron | Automated by provider |
| Recovery | Restore from dump | Point-in-time recovery (click a button) |
| Scaling DB | Resize entire VPS | Resize database independently |
| Monitoring | DIY | Provider dashboard + alerts |

## Database Connection

```bash
DATABASE_WRITER_URL=postgres://headspace:password@db-primary.host.com:5432/headspace?sslmode=require
DATABASE_READER_URL=postgres://headspace:password@db-primary.host.com:5432/headspace?sslmode=require
```

Both point to the same instance in Phase 2. Reader URL exists for Phase 3 readiness.

## Connection Pooling

Network connections are more expensive than Unix sockets. Pool configuration becomes more important:

```rust
let pool = PgPoolOptions::new()
    .max_connections(10)
    .min_connections(2)          // Keep warm connections ready
    .acquire_timeout(Duration::from_secs(5))
    .idle_timeout(Duration::from_mins(10))
    .connect(&database_url)
    .await?;
```

**TLS is mandatory.** Managed databases require encrypted connections. The connection string includes `sslmode=require`.

## Multi-Tenancy Model

Identical to Phase 1. Schema-per-tenant with `SET search_path`:

```
headspace (database on managed instance)
├── main
├── tenant_asif
├── tenant_bob
└── ...hundreds of tenant schemas
```

No application code changes.

## Managed Database Provider Comparison

| Provider | Cheapest Tier | Free Tier | PITR | Read Replicas |
|----------|-------------|-----------|------|---------------|
| Neon | $19/mo | Yes (0.5GB) | Yes | Yes (branching) |
| Supabase | $25/mo | Yes (500MB) | Yes | Planned |
| AWS RDS | ~$15/mo (db.t3.micro) | No | Yes | Yes ($15+ each) |
| Google Cloud SQL | ~$10/mo (db-f1-micro) | No | Yes | Yes |
| DigitalOcean | $15/mo | No | Yes | Yes ($15+ each) |

**Recommendation for this stage:** Neon or Supabase. Both offer generous free tiers to start with, serverless scaling, and are PostgreSQL-native.

## Backup and Recovery

**Automated backups (handled by provider):**
- Daily full backups
- Continuous WAL archiving
- Point-in-time recovery to any second in retention window

**Per-tenant backup (still works):**
```bash
pg_dump -h db.host.com -n tenant_asif headspace > tenant_asif.sql
```

**Per-tenant restore:**
```bash
# Drop and recreate schema
psql -h db.host.com -c "DROP SCHEMA tenant_asif CASCADE;" headspace
pg_restore -h db.host.com -n tenant_asif headspace < tenant_asif.sql
```

## Migrations

Same `run_all_migrations()` entry point as Phase 1, but over the network. Main schema migrations run first, then all tenant migrations. With hundreds of tenants, parallelize the tenant step:

```rust
pub async fn run_all_migrations(pool: &PgPool) -> Result<(), anyhow::Error> {
    // Step 1-2: Bootstrap + main schema (same as Phase 1)
    sqlx::query("CREATE SCHEMA IF NOT EXISTS main").execute(pool).await?;
    let mut conn = pool.acquire().await?;
    sqlx::query("SET search_path TO main").execute(&mut *conn).await?;
    sqlx::migrate!("migrations/main").run(&mut *conn).await?;

    // Step 3: Tenant migrations — parallelized for hundreds of tenants
    let tenants: Vec<String> = sqlx::query_scalar(
        "SELECT schema_name FROM main.companies WHERE schema_name IS NOT NULL"
    )
    .fetch_all(pool).await?;

    let semaphore = Arc::new(Semaphore::new(5)); // 5 concurrent migrations
    let mut tasks = Vec::new();

    for schema in tenants {
        let permit = semaphore.clone().acquire_owned().await?;
        let pool = pool.clone();
        tasks.push(tokio::spawn(async move {
            let result = migrate_tenant(&pool, &schema).await;
            drop(permit);
            result
        }));
    }

    for task in tasks {
        task.await??;
    }
    Ok(())
}
```

Each tenant has its own `_sqlx_migrations` table tracking applied migrations. If the server crashes mid-migration, restart picks up where it left off — already-migrated tenants are skipped, the failed tenant retries.

## Latency Considerations

Network adds ~1ms per database round-trip. For a page making N sequential queries:

| Queries per page | Added latency | Impact |
|-----------------|--------------|--------|
| 5 | 5ms | Imperceptible |
| 20 | 20ms | Barely noticeable |
| 50 | 50ms | Optimize query count |

**Mitigation:** Batch queries where possible. Use JOINs instead of N+1 queries. This is good practice regardless.

## Monitoring

Managed database provides:
- Connection count and utilization
- Query performance (slow query log)
- Storage usage and growth rate
- Replication lag (when replicas added)
- CPU and memory utilization

Application-side:
- Track query timing per request
- Alert on p99 latency spikes
- Monitor connection pool saturation

## Scaling Limits

A managed PostgreSQL instance with 4GB RAM handles:
- Hundreds of tenant schemas
- Millions of rows per tenant
- Hundreds of concurrent connections (through pooling)

**When to move to Phase 3:**
- Read queries dominate and you want to offload them
- Write latency suffers because reads saturate the primary
- You need multiple app servers for redundancy/availability
- Single-instance vertical scaling hits cost/performance ceiling

## Transition to Phase 3

**Effort:** 2-3 days of development work.

Changes required:
1. Add read replica (usually one click in managed provider)
2. Add PgBouncer or connection pooler
3. Split `DATABASE_READER_URL` to point at replica
4. Handle replication lag for read-your-own-writes pattern
5. Add load balancer in front of multiple app servers
