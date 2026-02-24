//! Background IMAP sync scheduler.
//!
//! Spawns a tokio task that periodically syncs all tenant IMAP mailboxes.

use sqlx::PgPool;
use tokio::task::JoinHandle;

/// Spawn the IMAP sync background loop.
///
/// Waits 30 seconds on startup, then runs `sync_all_tenants()` every 5 minutes.
/// Returns a JoinHandle that can be used to abort the loop if needed.
pub fn spawn_imap_sync_loop(pool: PgPool) -> JoinHandle<()> {
    tokio::spawn(async move {
        // Initial delay — let the app fully start before first sync
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        loop {
            tracing::debug!("IMAP sync: starting sync cycle");
            super::sync::sync_all_tenants(&pool).await;
            tracing::debug!("IMAP sync: cycle complete, sleeping 5 minutes");
            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
        }
    })
}
