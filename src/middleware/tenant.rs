//! Tenant resolution middleware.
//!
//! Extracts the tenant from the request subdomain
//! (e.g., "asif.headspace.local" -> tenant "asif")
//! and injects `Tenant` into request extensions.
