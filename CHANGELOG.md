# Changelog

All notable changes to Headspace CRM are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1-alpha] - 2026-03-16

### Added

- Multi-stage Dockerfile (node frontend build → rust release build → slim debian runtime)
- Docker Compose setup with Postgres 17, nginx reverse proxy, and app service
- `FALLBACK_TENANT` config option for running behind ngrok/tunnels without subdomains
- `.dockerignore` to keep build context lean
- nginx reverse proxy config with websocket support and 50MB upload limit
- Live demo link in README with credentials
- Docker quickstart section in README

## [0.1.0-alpha] - 2026-03-15

Initial alpha release. Full-featured multi-tenant CRM with email integration.

### Added

**Infrastructure**
- Axum 0.8 web server with Tokio runtime
- PostgreSQL schema-per-tenant multi-tenancy
- Subdomain-based tenant resolution middleware
- Session-based authentication with argon2 password hashing
- CSRF protection on all mutating requests
- Hybrid Askama + Vue.js architecture (server-rendered shells + interactive components)
- Vite build pipeline with code-splitting (dynamic imports)
- Auto-running migrations and seeding on startup

**Super Admin Panel**
- Login and session management
- Tenant CRUD (create, edit, delete with schema provisioning)
- Agent (super admin user) management
- Role management with permission assignment
- Account self-edit

**CRM - Contacts**
- Persons CRUD with dynamic email/phone fields
- Organizations CRUD
- Person detail page (contact info, linked leads, activities, tags)
- Organization detail page
- Person and organization search endpoints

**CRM - Leads**
- Lead CRUD with pipeline/stage assignment
- Kanban board with drag-and-drop stage movement (SortableJS)
- Lead detail page (stage bar, products, quotes, activities, tags, contact)
- Won/lost workflow with status, lost reason, and closed date
- Lead-product linking (add/remove products to leads)
- Lead-quote linking (link/unlink quotes)
- Rotten lead computation and indicators (list + kanban)
- Server-side paginated list view with search and sort

**CRM - Activities**
- Activities CRUD (calls, meetings, notes, emails, lunches)
- Calendar view (monthly grid with activity dots)
- Activity participants (users + persons)
- File attachments (multipart upload, download, delete)
- Activity timeline on lead/person detail pages

**CRM - Products & Quotes**
- Product catalog with rust_decimal pricing
- Quote builder with dynamic line items
- Per-item discount and tax percentages
- Billing/shipping address fields
- Quote PDF generation (genpdfi with NotoSans fonts)

**CRM - Pipeline & Settings**
- Pipeline and stage management with lead migration on stage delete
- Custom attributes system (text, number, select, date, boolean, etc.)
- User roles with granular ACL permissions
- Groups, lead sources, lead types, tags management
- Email templates CRUD
- Warehouses CRUD
- Web forms (public-facing lead capture forms)
- Tenant configuration (currency, date format, timezone, locale, brand color)

**CRM - Dashboard**
- 8 stat cards (total leads, won, lost, open, revenue, etc.)
- 4 charts: pipeline funnel, leads by source, revenue by month, lead status
- Top products and top persons tables
- Date range filtering

**Email**
- SMTP outbound via lettre (compose, send, save as draft, reply)
- IMAP inbound sync via async-imap (background 5-min polling + manual sync)
- Email threading (in_reply_to / parent_id)
- Folder management (inbox, sent, drafts, trash)
- Message deduplication by message_id
- Attachment parsing and storage from IMAP messages
- IMAP enable toggle in configuration

**Data Operations**
- CSV export for leads, persons, organizations, products
- CSV import with header mapping and FK lookup by name
- Mass delete on leads, persons, organizations, products, quotes, activities
- Global search across all major entities

**Security**
- Server-side ACL bouncer on all 15+ API endpoint groups
- View permission filtering (own/group/all record scoping)
- Server-side validation (validator crate) on all payload structs
- Pipeline stage sync with lead migration protections
- Last-role and own-role delete protections
- UNIQUE constraints on group and organization names

**UX**
- Light/dark theme support
- Breadcrumb navigation
- Quick creation button (header "+" menu for all entity types)
- Vue bundle code-splitting (~354KB main chunk)
