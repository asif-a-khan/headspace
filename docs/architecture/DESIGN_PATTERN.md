# Headspace CRM - Hybrid Architecture Design Pattern

## Overview

Headspace uses a **hybrid server-rendered + Vue.js interactive** architecture built on **Axum** (Rust). The backend serves HTML page shells using **Askama** compile-time templates. Interactive content within those shells is powered by **Vue.js 3** with **Vuetify** components and **Pinia** state management — similar to how Laravel uses Blade templates with Vue.

Single binary. Single deployment. No separate frontend application.

**Full tech stack:** See `../STACK.md`
**Implementation details:** See `IMPLEMENTATION_PHASE_1.md`, `IMPLEMENTATION_PHASE_2.md`, `IMPLEMENTATION_PHASE_3.md`

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────┐
│                  Rust Binary (Axum)                        │
│                                                            │
│  ┌────────────────────┐  ┌───────────────────────────┐   │
│  │   HTML Routes       │  │   JSON API Routes          │   │
│  │   (Askama templates)│  │   (serde JSON)             │   │
│  │                     │  │                            │   │
│  │  GET /admin/*       │  │  GET  /api/leads           │   │
│  │   → Askama renders  │  │  POST /api/leads           │   │
│  │     page shell with │  │  PATCH /api/leads/{id}     │   │
│  │     Vue mount points│  │  GET  /api/dashboard/stats │   │
│  │                     │  │                            │   │
│  │  Serves:            │  │  Consumed by:              │   │
│  │  - Page layouts     │  │  - Vue components          │   │
│  │  - Navigation       │  │  - Pinia stores            │   │
│  │  - Initial data     │  │  - Vuetify data tables     │   │
│  │  - Auth forms       │  │  - Dashboard charts        │   │
│  └────────────────────┘  └───────────────────────────┘   │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  tower Middleware Stack                               │ │
│  │  Tenant resolution → Auth → Tracing → CSRF → Handler │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  Background Tasks (tokio::spawn)                      │ │
│  │  IMAP email sync │ Scheduled jobs │ Data imports       │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  Static Assets (tower-http ServeDir)                  │ │
│  │  Vite-built CSS + Vue bundles + Vuetify + images      │ │
│  └──────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

---

## Rendering Strategy

### Two-Layer Rendering

Every authenticated page follows the same pattern:

1. **Askama renders the page shell** — layout, sidebar, header, breadcrumbs, and a Vue mount point with embedded initial data
2. **Vue mounts into the shell** — picks up the initial data from `window.__INITIAL_DATA__`, renders the interactive content using Vuetify components

This is the same pattern Laravel uses with Blade + Vue. The server handles auth, routing, and the page frame. Vue handles the interactive content.

### Page Types

| Page | Server (Askama) | Client (Vue) |
|------|----------------|--------------|
| Login / Password Reset | Full page, no Vue | None |
| Dashboard | Shell + stat cards | Charts, activity feed |
| Lead list (table) | Shell + breadcrumbs | Vuetify data table (sort, filter, paginate) |
| Lead list (kanban) | Shell + breadcrumbs | Kanban board (drag-and-drop) |
| Lead detail | Shell + sections | Inline editing, tag manager, activity timeline |
| Person / Org list | Shell + breadcrumbs | Vuetify data table |
| Person / Org detail | Shell + sections | Inline editing, tag manager |
| Product list / detail | Shell + breadcrumbs | Vuetify data table, forms |
| Activity calendar | Shell + breadcrumbs | Calendar view (month/week/day) |
| Quote builder | Shell + breadcrumbs | Dynamic line items, calculations |
| Settings pages | Shell + breadcrumbs | Vuetify forms, permission tree |
| User management | Shell + breadcrumbs | Vuetify data table, role assignment |

### Data Flow

```
1. Browser requests GET /admin/leads
2. tower middleware stack:
   - Resolve tenant from subdomain (set search_path)
   - Authenticate user (tower-sessions + sqlx session store)
   - Request tracing (tracing + tower-http)
3. Handler queries leads via sqlx (compile-time checked SQL)
4. Askama renders page shell:
   - Layout (header, sidebar, content area)
   - <div id="app"></div> mount point
   - <script>window.__INITIAL_DATA__ = {{ data|json }}</script>
   - <script> tags for Vue bundle
5. Browser receives HTML (fast first paint — shell visible immediately)
6. Vue initializes:
   - Pinia store hydrates from __INITIAL_DATA__ (no extra API call)
   - Vuetify data table renders with leads
   - Page fully interactive
7. Subsequent interactions (sort, filter, paginate, edit):
   - Vue calls JSON API routes (/api/leads?page=2&sort=title)
   - Pinia store updates
   - Vuetify components re-render reactively
```

**First paint:** ~50-100ms (server-rendered shell, navigation visible)
**Fully interactive:** ~300-600ms (Vue + Vuetify initialize, initial data already available)

---

## Mirror Routes Pattern

Every page that displays data has two routes sharing the same query logic:

| HTML Route | API Route | Shared Query |
|-----------|-----------|-------------|
| `GET /admin/leads` | `GET /api/leads` | `query_leads()` |
| `GET /admin/leads/{id}` | `GET /api/leads/{id}` | `query_lead_detail()` |
| `GET /admin/dashboard` | `GET /api/dashboard/stats` | `query_dashboard_stats()` |

```rust
// HTML route — returns Askama page shell with embedded initial data
pub async fn index(/* extractors */) -> Result<impl IntoResponse, AppError> {
    let leads = query_leads(&mut conn, &params).await?;
    Ok(LeadIndexTemplate { leads_json: serde_json::to_string(&leads)?, /* ... */ })
}

// API route — returns JSON for Vue to consume on subsequent requests
pub async fn api_index(/* extractors */) -> Result<Json<LeadListResponse>, AppError> {
    let leads = query_leads(&mut conn, &params).await?;
    Ok(Json(leads))
}
```

**Performance:** No significant overhead. The DB query is ~95% of the cost. Serializing to JSON vs rendering Askama is negligible. The initial page load embeds data, so there's no double request.

---

## Why This Architecture

### vs Full SPA (React/Vue CLI app)

| Concern | Full SPA | Hybrid (Askama + Vue) |
|---------|----------|----------------------|
| First paint | Blank → spinner → content | Instant shell, Vue fills content |
| Complexity | Two apps (API + SPA) | One app, one deployment |
| Type safety | Types defined twice (Rust + TS) | Rust structs → JSON → Vue |
| Auth | Token management, CORS | Session cookies, same origin |
| Multi-tenant routing | Subdomain CORS headaches | Handled by server, transparent |
| SEO/SSR | Needs SSR setup | Server renders shell natively |
| Build tooling | Full Node.js toolchain | Vite for assets only, cargo for server |

### vs Fully Server-Rendered (No JS)

Kanban boards, data tables with inline editing, calendars, and quote builders genuinely need client-side interactivity. Drag-and-drop, real-time filtering, and smooth animations are not achievable with server-rendered HTML alone. Vue + Vuetify provides a rich component library (data tables, date pickers, dialogs, autocomplete) that would take months to build from scratch with vanilla JS.

### vs Vanilla JS Islands (Previous Approach)

| Concern | Vanilla JS Islands | Vue + Vuetify |
|---------|-------------------|---------------|
| Development speed | Build every component from scratch | Vuetify provides 80+ ready components |
| Data tables | Custom DOM manipulation | `v-data-table` with sort, filter, paginate |
| Forms | Manual validation + rendering | Vuetify form components with built-in validation |
| State management | Ad-hoc per-island | Pinia stores, shared across components |
| Consistency | Manual design system | Vuetify's Material Design system |
| Bundle size | ~50KB | ~300KB (Vue + Vuetify + Pinia) |

The trade-off is bundle size. But for a CRM with 15+ interactive pages, using a component framework is dramatically faster to develop and maintain than hand-rolling every data table, form, dialog, and autocomplete.

---

## Design Principles

1. **Server-rendered shell.** Every page loads with layout, navigation, and breadcrumbs visible immediately. Vue enhances the content area.
2. **Initial data embedding.** First page load includes data in `window.__INITIAL_DATA__` — no extra API call. Subsequent interactions use JSON API routes.
3. **One deployment.** Single Rust binary serves HTML shells, JSON APIs, and static assets (Vite-built Vue bundles).
4. **Type-safe backend.** Askama checks templates at compile time. sqlx checks SQL at compile time. Struct field changes cause compile errors.
5. **Shared auth.** Same session cookie for HTML pages and API calls. No token management.
6. **Vuetify for UI.** Material Design components for consistent, accessible UI. No custom design system to maintain.
7. **Pinia for state.** Centralized stores hydrated from server data. Shared across Vue components on the same page.

---

## Implementation Phases

| Phase | Focus | Document |
|-------|-------|----------|
| 1 | Foundation — Axum, Askama, auth, CRUD, database layer | `IMPLEMENTATION_PHASE_1.md` |
| 2 | Interactivity — Vue + Vuetify components, API routes, Pinia stores, Vite build | `IMPLEMENTATION_PHASE_2.md` |
| 3 | Communication — Email (IMAP/SMTP), background jobs, file storage | `IMPLEMENTATION_PHASE_3.md` |

Database scaling phases are documented separately in `../db_design/PHASE_1.md` through `PHASE_3.md`.
