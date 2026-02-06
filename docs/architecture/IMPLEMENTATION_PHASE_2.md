# Implementation Phase 2: Interactivity

## Goal

Add Vue.js 3 + Vuetify + Pinia to server-rendered pages. Introduce JSON API routes consumed by Vue components. Build interactive features: kanban board, activity calendar, dashboard charts, quote builder, data tables with sorting/filtering/pagination, inline editing, and tag management.

**Prerequisite:** Phase 1 complete (server-rendered CRUD, auth, templates).

**After this phase:** The CRM feels like a modern application. Leads can be dragged between pipeline stages. Activities appear on a calendar. Dashboard shows interactive charts. Data tables support sorting, filtering, and inline editing — all powered by Vuetify components within Askama page shells.

---

## Frontend Architecture

### How Askama + Vue Work Together

This is the same pattern as Laravel Blade + Vue:

1. **Askama renders the page shell** — layout, sidebar, header, breadcrumbs, and a `<div id="app">` mount point
2. **Askama embeds initial data** — `<script>window.__INITIAL_DATA__ = {{ data|json }}</script>`
3. **Vue mounts into the shell** — picks up initial data via Pinia, renders interactive content with Vuetify components
4. **Subsequent interactions** — Vue calls JSON API routes, Pinia stores update, Vuetify re-renders

### File Structure

```
frontend/                          # Vue source (NOT served directly)
├── package.json
├── vite.config.ts
├── tsconfig.json
├── src/
│   ├── main.ts                    # Vue app bootstrap + Vuetify + Pinia setup
│   ├── api/
│   │   └── client.ts              # Fetch wrapper (CSRF, error handling)
│   ├── components/
│   │   ├── leads/
│   │   │   ├── LeadDataTable.vue  # Vuetify data table for lead list
│   │   │   ├── KanbanBoard.vue    # Drag-and-drop pipeline view
│   │   │   ├── LeadDetail.vue     # Lead detail with inline editing
│   │   │   └── LeadForm.vue       # Create/edit lead form
│   │   ├── activities/
│   │   │   ├── ActivityCalendar.vue
│   │   │   └── ActivityForm.vue
│   │   ├── dashboard/
│   │   │   ├── DashboardCharts.vue
│   │   │   └── StatCards.vue
│   │   ├── contacts/
│   │   │   ├── PersonDataTable.vue
│   │   │   └── OrganizationDataTable.vue
│   │   ├── products/
│   │   │   └── ProductDataTable.vue
│   │   ├── quotes/
│   │   │   └── QuoteBuilder.vue
│   │   ├── settings/
│   │   │   ├── UserManagement.vue
│   │   │   ├── RoleEditor.vue
│   │   │   └── PipelineConfig.vue
│   │   └── shared/
│   │       ├── TagManager.vue     # Reusable tag autocomplete + CRUD
│   │       ├── InlineEditor.vue   # Edit-in-place for attributes
│   │       └── FileUpload.vue     # Drag-and-drop file upload
│   ├── stores/
│   │   ├── leads.ts               # Lead list + kanban state
│   │   ├── activities.ts          # Activity calendar state
│   │   ├── dashboard.ts           # Dashboard stats
│   │   └── auth.ts                # Current user + permissions
│   └── composables/
│       ├── useApi.ts              # API client composable
│       ├── usePermissions.ts      # Permission checking (mirrors Rust ACL)
│       └── useNotifications.ts    # Toast notifications
├── static/
│   └── vendor/
│       └── sortable.min.js        # SortableJS (used by KanbanBoard)
```

```
static/                            # Served by Axum (tower-http ServeDir)
├── dist/                          # Vite build output (gitignored)
│   ├── assets/
│   │   ├── app-[hash].js          # Vue + Vuetify + Pinia bundle
│   │   └── app-[hash].css         # Vuetify + Tailwind styles
│   └── manifest.json              # Asset manifest for cache busting
├── css/
│   └── app.css                    # Compiled Tailwind (for shell styling)
└── images/
    └── favicon.svg
```

---

## Vite Configuration

```typescript
// frontend/vite.config.ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vuetify from 'vite-plugin-vuetify'

export default defineConfig({
  plugins: [
    vue(),
    vuetify({ autoImport: true }),
  ],
  build: {
    outDir: '../static/dist',
    emptyOutDir: true,
    manifest: true,
    rollupOptions: {
      input: 'src/main.ts',
    },
  },
  server: {
    // Dev proxy: forward API calls to Rust backend
    proxy: {
      '/api': 'http://localhost:8000',
    },
  },
})
```

### Build Commands

```bash
# Development (with HMR)
cd frontend && npm run dev    # Vite dev server on :5173
cargo run                      # Axum server on :8000

# Production build
cd frontend && npm run build  # Outputs to static/dist/
cargo build --release         # Single binary serves everything
```

---

## Vue App Bootstrap

```typescript
// frontend/src/main.ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createVuetify } from 'vuetify'
import 'vuetify/styles'

// Mount Vue into the server-rendered shell
const mountEl = document.getElementById('app')
if (!mountEl) {
  // Page doesn't have a Vue mount point (e.g., login page)
  // This is fine — not all pages use Vue
} else {
  const app = createApp({
    // Dynamic component based on data attribute
    template: `<component :is="pageComponent" />`,
    computed: {
      pageComponent() {
        return mountEl?.dataset.component || 'div'
      }
    }
  })

  const pinia = createPinia()

  // Hydrate Pinia from server-embedded data
  if (window.__INITIAL_DATA__) {
    pinia.state.value = window.__INITIAL_DATA__
  }

  const vuetify = createVuetify({
    theme: {
      defaultTheme: 'light',
      themes: {
        light: {
          colors: {
            primary: '#6366f1',   // Indigo — matches Headspace brand
            secondary: '#8b5cf6',
          }
        }
      }
    }
  })

  app.use(pinia)
  app.use(vuetify)
  app.mount('#app')
}
```

---

## JSON API Layer

Vue components consume JSON APIs. These routes share the same middleware stack (tenant resolution, auth, **permission checking**) as HTML routes but return `serde` JSON instead of Askama templates.

The same RBAC system from Phase 1 applies:
- **Route permissions:** The auth middleware maps each API route to an ACL key and checks the user's role
- **Data scoping:** API handlers call `get_authorized_user_ids()` to filter results by `view_permission`
- **Permission-gated components:** The server-rendered page only includes Vue mount points for features the user has permission to use

### API Router

```rust
// Added to routes/mod.rs
let api_routes = Router::new()
    .route("/api/leads", get(api::leads::index).post(api::leads::store))
    .route("/api/leads/{id}", get(api::leads::show).put(api::leads::update))
    .route("/api/leads/{id}/stage", patch(api::leads::update_stage))
    .route("/api/activities", get(api::activities::index).post(api::activities::store))
    .route("/api/activities/{id}", put(api::activities::update).delete(api::activities::destroy))
    .route("/api/dashboard/stats", get(api::dashboard::stats))
    .route("/api/quotes/{id}/items", get(api::quotes::items).post(api::quotes::add_item))
    .route("/api/quotes/{id}/items/{item_id}", put(api::quotes::update_item).delete(api::quotes::remove_item))
    .route("/api/tags", get(api::tags::search))
    .route("/api/tags/{entity}/{id}", post(api::tags::attach).delete(api::tags::detach))
    .route("/api/persons/search", get(api::persons::search))
    .route("/api/organizations/search", get(api::organizations::search))
    .layer(middleware::from_fn(middleware::auth::require_auth));
```

### API Response Pattern

```rust
// src/api/leads.rs
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct LeadListResponse {
    pub leads: Vec<LeadRow>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

pub async fn index(
    Extension(user): Extension<User>,
    Extension(tenant): Extension<Tenant>,
    Extension(db): Extension<Database>,
    Query(params): Query<LeadApiParams>,
) -> Result<Json<LeadListResponse>, AppError> {
    let mut conn = db.reader().acquire().await?;
    set_tenant(&mut conn, &tenant.schema_name).await?;

    // Data scoping: filter by view_permission
    let authorized_ids = get_authorized_user_ids(&mut conn, &user).await?;
    let (leads, total) = query_leads(&mut conn, &params, authorized_ids.as_deref()).await?;

    Ok(Json(LeadListResponse {
        leads,
        total,
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(25),
    }))
}
```

### Mirror Route Pattern

HTML and API handlers share the same query function:

```rust
// src/handlers/leads.rs — HTML route (returns Askama shell with embedded data)
pub async fn index(/* extractors */) -> Result<impl IntoResponse, AppError> {
    let (leads, total) = query_leads(&mut conn, &params, authorized_ids.as_deref()).await?;
    let leads_json = serde_json::to_string(&LeadListResponse {
        leads, total, page: 1, per_page: 25,
    })?;
    Ok(LeadIndexTemplate { leads_json, user, tenant, csrf_token })
}

// src/api/leads.rs — API route (returns JSON for Vue)
pub async fn index(/* extractors */) -> Result<Json<LeadListResponse>, AppError> {
    let (leads, total) = query_leads(&mut conn, &params, authorized_ids.as_deref()).await?;
    Ok(Json(LeadListResponse { leads, total, page, per_page }))
}

// Shared query function used by both
async fn query_leads(
    conn: &mut PgConnection,
    params: &LeadApiParams,
    authorized_user_ids: Option<&[i64]>,
) -> Result<(Vec<LeadRow>, i64), sqlx::Error> {
    // ... SQL query with view-permission scoping
}
```

### API Error Format

API routes return JSON errors (not HTML redirects):

```rust
impl AppError {
    fn api_response(&self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            AppError::Validation(errors) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({ "errors": errors })),
                ).into_response();
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

---

## API Client (Vue Side)

```typescript
// frontend/src/api/client.ts

const csrfToken = () =>
  document.querySelector('meta[name="csrf-token"]')?.getAttribute('content')

export async function api<T>(url: string, options: RequestInit = {}): Promise<T> {
  const defaults: RequestInit = {
    headers: {
      'Content-Type': 'application/json',
      'X-CSRF-Token': csrfToken() || '',
    },
    credentials: 'same-origin',
  }

  const response = await fetch(url, { ...defaults, ...options })

  if (!response.ok) {
    const body = await response.json().catch(() => ({}))
    throw new ApiError(response.status, body.error || 'Request failed', body.errors)
  }

  if (response.status === 204) return null as T
  return response.json()
}

export class ApiError extends Error {
  status: number
  fieldErrors?: Record<string, string>[]

  constructor(status: number, message: string, fieldErrors?: Record<string, string>[]) {
    super(message)
    this.status = status
    this.fieldErrors = fieldErrors
  }
}

// Convenience methods
export const get = <T>(url: string) => api<T>(url)
export const post = <T>(url: string, data: unknown) =>
  api<T>(url, { method: 'POST', body: JSON.stringify(data) })
export const put = <T>(url: string, data: unknown) =>
  api<T>(url, { method: 'PUT', body: JSON.stringify(data) })
export const patch = <T>(url: string, data: unknown) =>
  api<T>(url, { method: 'PATCH', body: JSON.stringify(data) })
export const del = <T>(url: string) =>
  api<T>(url, { method: 'DELETE' })
```

---

## Pinia Stores

### Leads Store

```typescript
// frontend/src/stores/leads.ts
import { defineStore } from 'pinia'
import { get, patch, post } from '@/api/client'

interface Lead {
  id: number
  title: string
  person_name: string | null
  organization_name: string | null
  lead_value: number | null
  stage_id: number
  tags: string[]
  created_at: string
}

interface StageColumn {
  id: number
  name: string
  leads: Lead[]
  total_value: number
}

export const useLeadsStore = defineStore('leads', {
  state: () => ({
    leads: [] as Lead[],
    stages: [] as StageColumn[],
    total: 0,
    page: 1,
    perPage: 25,
    loading: false,
    sortBy: 'created_at',
    sortDesc: true,
    filters: {} as Record<string, string>,
  }),

  actions: {
    async fetchLeads(params?: Record<string, string | number>) {
      this.loading = true
      try {
        const query = new URLSearchParams(params as Record<string, string>).toString()
        const data = await get<{ leads: Lead[]; total: number }>(`/api/leads?${query}`)
        this.leads = data.leads
        this.total = data.total
      } finally {
        this.loading = false
      }
    },

    async fetchKanban(pipelineId: number) {
      this.loading = true
      try {
        const data = await get<{ stages: StageColumn[] }>(`/api/leads?pipeline_id=${pipelineId}&view=kanban`)
        this.stages = data.stages
      } finally {
        this.loading = false
      }
    },

    async moveLeadToStage(leadId: number, stageId: number, sortOrder: number) {
      await patch(`/api/leads/${leadId}/stage`, { stage_id: stageId, sort_order: sortOrder })
    },
  },
})
```

### Auth Store

```typescript
// frontend/src/stores/auth.ts
import { defineStore } from 'pinia'

interface User {
  id: number
  name: string
  email: string
  permission_type: string
  permissions: string[]
  view_permission: string
}

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null as User | null,
  }),

  getters: {
    hasPermission: (state) => (permission: string): boolean => {
      if (!state.user) return false
      if (state.user.permission_type === 'all') return true
      return state.user.permissions.includes(permission)
    },
  },
})
```

---

## Vue Components

### Askama Page Shell (Server Side)

```html
{# templates/pages/leads/index.html #}
{% extends "layouts/authenticated.html" %}

{% block title %}Leads{% endblock %}

{% block content %}
<div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-semibold text-gray-900">Leads</h1>
</div>

{# Vue mount point with page component and initial data #}
<div id="app" data-component="LeadIndex"></div>

<script>
  window.__INITIAL_DATA__ = {
    leads: {{ leads_json|safe }},
    auth: {
      user: {{ user_json|safe }}
    }
  }
</script>
{% endblock %}
```

### Lead Data Table (Vuetify)

```vue
<!-- frontend/src/components/leads/LeadDataTable.vue -->
<template>
  <v-card>
    <v-card-title class="d-flex align-center">
      <v-text-field
        v-model="search"
        prepend-inner-icon="mdi-magnify"
        label="Search leads..."
        single-line
        hide-details
        density="compact"
        class="mr-4"
      />
      <v-spacer />
      <v-btn
        v-if="authStore.hasPermission('leads.create')"
        color="primary"
        @click="$emit('create')"
      >
        Create Lead
      </v-btn>
    </v-card-title>

    <v-data-table-server
      v-model:items-per-page="perPage"
      v-model:page="page"
      v-model:sort-by="sortBy"
      :headers="headers"
      :items="leadsStore.leads"
      :items-length="leadsStore.total"
      :loading="leadsStore.loading"
      @update:options="loadLeads"
    >
      <template #item.title="{ item }">
        <a :href="`/admin/leads/${item.id}`" class="text-primary">
          {{ item.title }}
        </a>
      </template>

      <template #item.lead_value="{ item }">
        {{ item.lead_value ? `$${item.lead_value.toLocaleString()}` : '—' }}
      </template>

      <template #item.actions="{ item }">
        <v-btn
          v-if="authStore.hasPermission('leads.edit')"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/leads/${item.id}/edit`"
        />
        <v-btn
          v-if="authStore.hasPermission('leads.delete')"
          icon="mdi-delete"
          size="small"
          variant="text"
          color="error"
          @click="confirmDelete(item)"
        />
      </template>
    </v-data-table-server>
  </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useLeadsStore } from '@/stores/leads'
import { useAuthStore } from '@/stores/auth'

const leadsStore = useLeadsStore()
const authStore = useAuthStore()

const search = ref('')
const page = ref(1)
const perPage = ref(25)
const sortBy = ref([{ key: 'created_at', order: 'desc' }])

const headers = [
  { title: 'Title', key: 'title', sortable: true },
  { title: 'Contact', key: 'person_name', sortable: true },
  { title: 'Organization', key: 'organization_name', sortable: true },
  { title: 'Value', key: 'lead_value', sortable: true },
  { title: 'Stage', key: 'stage_name', sortable: true },
  { title: 'Created', key: 'created_at', sortable: true },
  { title: '', key: 'actions', sortable: false, align: 'end' },
]

async function loadLeads(options: any) {
  await leadsStore.fetchLeads({
    page: options.page,
    per_page: options.itemsPerPage,
    sort: options.sortBy[0]?.key || 'created_at',
    order: options.sortBy[0]?.order || 'desc',
    search: search.value,
  })
}
</script>
```

### Kanban Board

```vue
<!-- frontend/src/components/leads/KanbanBoard.vue -->
<template>
  <div class="d-flex gap-4 overflow-x-auto pb-4">
    <div
      v-for="stage in leadsStore.stages"
      :key="stage.id"
      class="kanban-column"
      style="min-width: 280px; width: 280px;"
    >
      <v-card variant="outlined" class="h-100">
        <v-card-title class="text-subtitle-2 d-flex justify-space-between">
          <span>{{ stage.name }}</span>
          <v-chip size="small">{{ stage.leads.length }}</v-chip>
        </v-card-title>
        <v-card-subtitle class="text-caption">
          ${{ stage.total_value.toLocaleString() }}
        </v-card-subtitle>

        <v-card-text
          :ref="el => setDropZone(el, stage.id)"
          class="kanban-cards pa-2"
          style="min-height: 100px;"
        >
          <v-card
            v-for="lead in stage.leads"
            :key="lead.id"
            :data-lead-id="lead.id"
            class="mb-2 cursor-grab"
            variant="elevated"
            density="compact"
          >
            <v-card-text class="pa-3">
              <a :href="`/admin/leads/${lead.id}`" class="text-body-2 font-weight-medium">
                {{ lead.title }}
              </a>
              <div v-if="lead.person_name" class="text-caption text-grey mt-1">
                {{ lead.person_name }}
              </div>
              <div v-if="lead.lead_value" class="text-caption text-success mt-1 font-weight-medium">
                ${{ lead.lead_value.toLocaleString() }}
              </div>
            </v-card-text>
          </v-card>
        </v-card-text>
      </v-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, nextTick } from 'vue'
import Sortable from 'sortablejs'
import { useLeadsStore } from '@/stores/leads'
import { useNotifications } from '@/composables/useNotifications'

const props = defineProps<{ pipelineId: number }>()
const leadsStore = useLeadsStore()
const { notify } = useNotifications()

const dropZones = new Map<number, HTMLElement>()

function setDropZone(el: HTMLElement | null, stageId: number) {
  if (el) dropZones.set(stageId, el)
}

onMounted(async () => {
  await leadsStore.fetchKanban(props.pipelineId)
  await nextTick()

  // Initialize SortableJS on each column
  for (const [stageId, el] of dropZones) {
    new Sortable(el, {
      group: 'kanban',
      animation: 150,
      ghostClass: 'opacity-30',
      onEnd: async (event) => {
        const leadId = parseInt(event.item.dataset.leadId!)
        const newStageId = parseInt(event.to.closest('[data-stage-id]')?.dataset.stageId || String(stageId))
        try {
          await leadsStore.moveLeadToStage(leadId, newStageId, event.newIndex!)
        } catch {
          notify('Failed to move lead', 'error')
          await leadsStore.fetchKanban(props.pipelineId) // revert
        }
      },
    })
  }
})
</script>
```

### Dashboard Charts

```vue
<!-- frontend/src/components/dashboard/DashboardCharts.vue -->
<template>
  <v-row>
    <v-col cols="12" md="6">
      <v-card>
        <v-card-title class="text-subtitle-1">Leads by Stage</v-card-title>
        <v-card-text>
          <canvas ref="pipelineChart" height="300"></canvas>
        </v-card-text>
      </v-card>
    </v-col>
    <v-col cols="12" md="6">
      <v-card>
        <v-card-title class="text-subtitle-1">Revenue Trend</v-card-title>
        <v-card-text>
          <canvas ref="revenueChart" height="300"></canvas>
        </v-card-text>
      </v-card>
    </v-col>
  </v-row>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Chart from 'chart.js/auto'
import { useDashboardStore } from '@/stores/dashboard'

const dashboardStore = useDashboardStore()
const pipelineChart = ref<HTMLCanvasElement>()
const revenueChart = ref<HTMLCanvasElement>()

onMounted(async () => {
  await dashboardStore.fetchStats()

  if (pipelineChart.value) {
    new Chart(pipelineChart.value, {
      type: 'bar',
      data: {
        labels: dashboardStore.pipelineStats.labels,
        datasets: [{
          label: 'Leads',
          data: dashboardStore.pipelineStats.values,
          backgroundColor: '#6366f1',
        }]
      },
      options: { responsive: true, maintainAspectRatio: false }
    })
  }

  if (revenueChart.value) {
    new Chart(revenueChart.value, {
      type: 'line',
      data: {
        labels: dashboardStore.revenueStats.labels,
        datasets: [{
          label: 'Revenue',
          data: dashboardStore.revenueStats.values,
          borderColor: '#10b981',
          fill: false,
          tension: 0.3,
        }]
      },
      options: { responsive: true, maintainAspectRatio: false }
    })
  }
})
</script>
```

### Tag Manager (Reusable)

```vue
<!-- frontend/src/components/shared/TagManager.vue -->
<template>
  <div class="d-flex flex-wrap align-center ga-1">
    <v-chip
      v-for="tag in tags"
      :key="tag.id"
      closable
      size="small"
      @click:close="removeTag(tag.id)"
    >
      {{ tag.name }}
    </v-chip>

    <v-autocomplete
      v-model="selectedTag"
      :items="suggestions"
      :loading="searching"
      item-title="name"
      item-value="name"
      density="compact"
      variant="plain"
      placeholder="Add tag..."
      hide-details
      no-filter
      style="max-width: 150px;"
      @update:search="searchTags"
      @update:model-value="addTag"
    >
      <template #no-data>
        <v-list-item v-if="searchQuery" @click="addTag(searchQuery)">
          <v-list-item-title>Create "{{ searchQuery }}"</v-list-item-title>
        </v-list-item>
      </template>
    </v-autocomplete>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { get, post, del } from '@/api/client'

const props = defineProps<{
  entity: string   // "lead", "person", etc.
  entityId: number
  initialTags: { id: number; name: string }[]
}>()

const tags = ref([...props.initialTags])
const suggestions = ref<{ name: string }[]>([])
const selectedTag = ref<string | null>(null)
const searching = ref(false)
const searchQuery = ref('')

let debounceTimer: number
async function searchTags(query: string) {
  searchQuery.value = query
  if (query.length < 2) { suggestions.value = []; return }
  clearTimeout(debounceTimer)
  debounceTimer = setTimeout(async () => {
    searching.value = true
    suggestions.value = await get(`/api/tags?search=${encodeURIComponent(query)}`)
    searching.value = false
  }, 200)
}

async function addTag(name: string | null) {
  if (!name) return
  const result = await post<{ id: number; name: string }>(
    `/api/tags/${props.entity}/${props.entityId}`,
    { name }
  )
  tags.value.push(result)
  selectedTag.value = null
}

async function removeTag(tagId: number) {
  await del(`/api/tags/${props.entity}/${props.entityId}?tag_id=${tagId}`)
  tags.value = tags.value.filter(t => t.id !== tagId)
}
</script>
```

---

## Permission-Gated UI (Vue Side)

Permissions are embedded in `__INITIAL_DATA__` and available via the auth store:

```vue
<!-- Conditional rendering based on permissions -->
<v-btn v-if="authStore.hasPermission('leads.create')" color="primary">
  Create Lead
</v-btn>

<!-- Composable for use in script setup -->
<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'
const authStore = useAuthStore()
const canEdit = authStore.hasPermission('leads.edit')
</script>
```

The server-rendered shell can also conditionally include the Vue mount point:

```html
{# Only render kanban mount if user has leads.view permission #}
{% if user.has_permission("leads") %}
<div id="app" data-component="LeadIndex"></div>
<script>window.__INITIAL_DATA__ = { ... }</script>
{% endif %}
```

---

## Build Pipeline

### package.json

```json
{
  "name": "headspace-frontend",
  "private": true,
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "vue": "^3.5",
    "vuetify": "^3.7",
    "pinia": "^2.3",
    "@mdi/font": "^7.4",
    "chart.js": "^4.4",
    "sortablejs": "^1.15"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.2",
    "vite": "^6.1",
    "vite-plugin-vuetify": "^2.0",
    "typescript": "^5.7",
    "vue-tsc": "^2.2",
    "@types/sortablejs": "^1.15"
  }
}
```

### Development Workflow

```bash
# Terminal 1: Rust backend
cargo run

# Terminal 2: Vite dev server (proxies API to Rust)
cd frontend && npm run dev

# For production: build frontend, then run Rust binary
cd frontend && npm run build
cargo build --release
./target/release/headspace  # Serves everything from static/dist/
```

---

## What Phase 2 Delivers

| Feature | Status |
|---------|--------|
| Vue.js 3 + Vuetify 3 + Pinia integration | Complete |
| Vite build pipeline (dev HMR + production builds) | Complete |
| JSON API routes (leads, activities, dashboard, tags, search) | Complete |
| Mirror routes (HTML + JSON sharing query logic) | Complete |
| Initial data embedding (`__INITIAL_DATA__`) | Complete |
| Lead data table (Vuetify `v-data-table-server`, sort, filter, paginate) | Complete |
| Kanban board (SortableJS drag-and-drop between stages) | Complete |
| Activity calendar (month/week navigation) | Complete |
| Dashboard charts (Chart.js within Vue components) | Complete |
| Tag manager (Vuetify autocomplete, CRUD) | Complete |
| Quote builder (dynamic line items, calculations) | Complete |
| Inline attribute editor | Complete |
| Permission-gated Vue components (mirrors Rust ACL) | Complete |
| API error handling (JSON responses) | Complete |
| API client (TypeScript, CSRF, typed responses) | Complete |
| Notification system (Vuetify snackbar) | Complete |

**Not in Phase 2:** Email (IMAP/SMTP), file upload storage, data import/export, background jobs. These are covered in Phase 3.
