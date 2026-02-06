//! Authentication guard middleware.
//!
//! Requires an authenticated session. Redirects to /login if
//! no valid session exists. Injects `User` into request extensions.
