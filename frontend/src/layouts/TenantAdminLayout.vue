<template>
  <v-app>
    <!-- Header -->
    <v-app-bar flat :height="62" color="surface" border="b" class="px-2">
      <template #prepend>
        <a href="/admin/dashboard" class="d-flex align-center text-decoration-none ml-2">
          <svg width="32" height="32" viewBox="0 0 32 32" class="mr-2">
            <rect width="32" height="32" rx="6" fill="#6366F1"/>
            <text x="16" y="23" text-anchor="middle" fill="white" font-size="20" font-weight="700" font-family="Inter, sans-serif">H</text>
          </svg>
          <span class="text-h6 font-weight-bold text-on-surface">{{ companyName }}</span>
        </a>
      </template>

      <v-spacer />

      <!-- Global Search -->
      <div class="global-search-wrapper mr-2" style="position: relative; max-width: 300px; width: 100%;">
        <v-text-field
          v-model="searchQuery"
          density="compact"
          hide-details
          placeholder="Search..."
          prepend-inner-icon="mdi-magnify"
          clearable
          variant="outlined"
          class="search-field"
          @input="onSearchInput"
          @focus="showResults = true"
          @blur="hideResultsDelayed"
        />
        <v-card
          v-if="showResults && searchResults.length"
          class="search-results-card"
          elevation="8"
        >
          <v-list density="compact">
            <v-list-item
              v-for="result in searchResults"
              :key="`${result.entity_type}-${result.id}`"
              :href="result.url"
              class="search-result-item"
            >
              <template #prepend>
                <v-icon size="small" :color="entityColor(result.entity_type)">
                  {{ entityIcon(result.entity_type) }}
                </v-icon>
              </template>
              <v-list-item-title class="text-body-2">{{ result.title }}</v-list-item-title>
              <v-list-item-subtitle class="text-caption">
                {{ result.entity_type }}
                <span v-if="result.subtitle"> - {{ result.subtitle }}</span>
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </v-card>
      </div>

      <!-- Quick Create -->
      <v-menu location="bottom end">
        <template #activator="{ props }">
          <v-btn v-bind="props" icon variant="tonal" color="primary" size="small" class="mr-1">
            <v-icon>mdi-plus</v-icon>
          </v-btn>
        </template>
        <v-list density="compact" min-width="180">
          <v-list-item prepend-icon="mdi-filter-variant" title="Lead" href="/admin/leads/create" />
          <v-list-item prepend-icon="mdi-account" title="Person" href="/admin/contacts/persons/create" />
          <v-list-item prepend-icon="mdi-domain" title="Organization" href="/admin/contacts/organizations/create" />
          <v-list-item prepend-icon="mdi-calendar-clock" title="Activity" href="/admin/activities/create" />
          <v-list-item prepend-icon="mdi-package-variant-closed" title="Product" href="/admin/products/create" />
          <v-list-item prepend-icon="mdi-file-document-outline" title="Quote" href="/admin/quotes/create" />
        </v-list>
      </v-menu>

      <!-- Theme toggle -->
      <v-btn
        icon
        variant="text"
        @click="toggleTheme"
      >
        <v-icon>{{ currentTheme === 'light' ? 'mdi-weather-sunny' : currentTheme === 'sunset' ? 'mdi-weather-sunset' : 'mdi-weather-night' }}</v-icon>
      </v-btn>

      <!-- Profile dropdown -->
      <v-menu location="bottom end">
        <template #activator="{ props }">
          <v-btn v-bind="props" variant="text" class="text-none">
            <v-avatar color="primary" size="36" class="mr-2">
              <span class="text-white font-weight-medium">{{ adminInitial }}</span>
            </v-avatar>
            <span class="text-body-2 text-on-surface d-none d-sm-inline">{{ adminName }}</span>
            <v-icon end size="small">mdi-chevron-down</v-icon>
          </v-btn>
        </template>
        <v-list density="compact" min-width="180">
          <v-list-item
            prepend-icon="mdi-account-circle"
            title="My Account"
            href="/admin/account"
          />
          <v-list-item
            prepend-icon="mdi-logout"
            title="Sign Out"
            @click="logout"
          />
        </v-list>
      </v-menu>
    </v-app-bar>

    <!-- Sidebar -->
    <v-navigation-drawer
      permanent
      :rail="!sidebarExpanded"
      :rail-width="70"
      :width="200"
      color="surface"
      border="r"
      @mouseenter="sidebarExpanded = true"
      @mouseleave="sidebarExpanded = false"
    >
      <v-list density="compact" nav class="mt-2 sidebar-list" :opened="openGroups">
        <v-list-item
          prepend-icon="mdi-view-dashboard"
          title="Dashboard"
          href="/admin/dashboard"
          :active="isActive('/admin/dashboard')"
          :class="{ 'active-item': isActive('/admin/dashboard') }"
          rounded="lg"
        />

        <v-list-item
          prepend-icon="mdi-filter-variant"
          title="Leads"
          href="/admin/leads"
          :active="isActive('/admin/leads')"
          :class="{ 'active-item': isActive('/admin/leads') }"
          rounded="lg"
        />

        <v-list-item
          prepend-icon="mdi-file-document-outline"
          title="Quotes"
          href="/admin/quotes"
          :active="isActive('/admin/quotes')"
          :class="{ 'active-item': isActive('/admin/quotes') }"
          rounded="lg"
        />

        <v-list-item
          prepend-icon="mdi-email-outline"
          title="Mail"
          href="/admin/mail"
          :active="currentPath.startsWith('/admin/mail')"
          rounded="lg"
        />

        <v-list-item
          prepend-icon="mdi-calendar-clock"
          title="Activities"
          href="/admin/activities"
          :active="isActive('/admin/activities')"
          :class="{ 'active-item': isActive('/admin/activities') }"
          rounded="lg"
        />

        <v-list-group value="contacts">
          <template #activator="{ props }">
            <v-list-item
              v-bind="props"
              prepend-icon="mdi-account-multiple"
              title="Contacts"
              rounded="lg"
            />
          </template>
          <v-list-item
            title="Persons"
            href="/admin/contacts/persons"
            :active="isActive('/admin/contacts/persons')"
            :class="{ 'active-item': isActive('/admin/contacts/persons') }"
            rounded="lg"
          />
          <v-list-item
            title="Organizations"
            href="/admin/contacts/organizations"
            :active="isActive('/admin/contacts/organizations')"
            :class="{ 'active-item': isActive('/admin/contacts/organizations') }"
            rounded="lg"
          />
        </v-list-group>

        <v-list-item
          prepend-icon="mdi-package-variant-closed"
          title="Products"
          href="/admin/products"
          :active="isActive('/admin/products')"
          :class="{ 'active-item': isActive('/admin/products') }"
          rounded="lg"
        />

        <v-list-item
          prepend-icon="mdi-cog"
          title="Settings"
          href="/admin/settings"
          :active="isActive('/admin/settings')"
          :class="{ 'active-item': isActive('/admin/settings') }"
          rounded="lg"
        />
      </v-list>
    </v-navigation-drawer>

    <!-- Main content -->
    <v-main class="bg-background">
      <v-container fluid class="px-4 pb-6 pt-4">
        <v-alert
          v-if="flash"
          type="success"
          closable
          variant="tonal"
          class="mb-4"
          @click:close="flash = ''"
        >
          {{ flash }}
        </v-alert>
        <v-breadcrumbs
          v-if="breadcrumbs.length > 1"
          :items="breadcrumbs"
          density="compact"
          class="px-0 pt-0 pb-3 text-body-2"
        >
          <template #divider>
            <v-icon size="x-small">mdi-chevron-right</v-icon>
          </template>
        </v-breadcrumbs>
        <slot />
      </v-container>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useTheme } from "vuetify";
import { get } from "@/api/client";

const theme = useTheme();
const data = (window as any).__INITIAL_DATA__ || {};
const adminName = computed(() => data.admin_name || "Admin");
const adminInitial = computed(() => (data.admin_name || "A").charAt(0).toUpperCase());
const companyName = computed(() => data.company_name || "Headspace");
const flash = ref(data.flash || "");
const currentPath = window.location.pathname;
const sidebarExpanded = ref(true);

// --- Global search ---
const searchQuery = ref("");
const searchResults = ref<any[]>([]);
const showResults = ref(false);
let searchTimer: ReturnType<typeof setTimeout> | null = null;

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(doSearch, 300);
}

async function doSearch() {
  const q = searchQuery.value?.trim();
  if (!q || q.length < 2) {
    searchResults.value = [];
    return;
  }
  try {
    const res = await get<{ data: any[] }>(`/admin/api/search?q=${encodeURIComponent(q)}`);
    searchResults.value = res.data;
    showResults.value = true;
  } catch {
    searchResults.value = [];
  }
}

function hideResultsDelayed() {
  setTimeout(() => { showResults.value = false; }, 200);
}

function entityIcon(type: string): string {
  switch (type) {
    case "Lead": return "mdi-filter-variant";
    case "Person": return "mdi-account";
    case "Product": return "mdi-package-variant-closed";
    case "Quote": return "mdi-file-document-outline";
    case "Organization": return "mdi-domain";
    default: return "mdi-magnify";
  }
}

function entityColor(type: string): string {
  switch (type) {
    case "Lead": return "primary";
    case "Person": return "blue";
    case "Product": return "green";
    case "Quote": return "orange";
    case "Organization": return "purple";
    default: return "grey";
  }
}

const isDark = computed(() => theme.global.current.value.dark);
const currentTheme = computed(() => theme.global.name.value);

// --- Breadcrumbs ---
const labelMap: Record<string, string> = {
  admin: "",
  dashboard: "Dashboard",
  leads: "Leads",
  kanban: "Kanban",
  quotes: "Quotes",
  activities: "Activities",
  contacts: "Contacts",
  persons: "Persons",
  organizations: "Organizations",
  products: "Products",
  settings: "Settings",
  users: "Users",
  roles: "Roles",
  groups: "Groups",
  attributes: "Attributes",
  pipelines: "Pipelines",
  sources: "Sources",
  types: "Types",
  account: "My Account",
  create: "Create",
  edit: "Edit",
};

const breadcrumbs = computed(() => {
  const parts = currentPath.replace(/\/$/, "").split("/").filter(Boolean);
  const crumbs: Array<{ title: string; href?: string; disabled?: boolean }> = [
    { title: "Home", href: "/admin/dashboard" },
  ];
  let path = "";
  for (let i = 0; i < parts.length; i++) {
    const segment = parts[i];
    path += "/" + segment;
    if (segment === "admin") continue;
    // Numeric IDs → skip (they're entity IDs between parent and "edit"/"show")
    if (/^\d+$/.test(segment)) continue;
    const label = labelMap[segment] || segment.charAt(0).toUpperCase() + segment.slice(1);
    const isLast = i === parts.length - 1;
    crumbs.push({
      title: label,
      href: isLast ? undefined : path,
      disabled: isLast,
    });
  }
  return crumbs;
});

const openGroups = computed(() => {
  const groups: string[] = [];
  if (currentPath.startsWith("/admin/contacts")) groups.push("contacts");
  return groups;
});

function isActive(path: string): boolean {
  return currentPath.startsWith(path);
}

function toggleTheme() {
  const cycle = ["light", "sunset", "dark"];
  const idx = cycle.indexOf(currentTheme.value);
  const next = cycle[(idx + 1) % cycle.length];
  theme.global.name.value = next;
  localStorage.setItem("headspace-theme", next);
}

async function logout() {
  const meta = document.querySelector('meta[name="csrf-token"]');
  const token = meta?.getAttribute("content") ?? "";
  await fetch("/admin/logout", {
    method: "POST",
    headers: { "X-CSRF-Token": token },
    credentials: "same-origin",
  });
  window.location.href = "/admin/login";
}
</script>

<style scoped>
.search-results-card {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  z-index: 1000;
  max-height: 400px;
  overflow-y: auto;
  margin-top: 4px;
}
</style>
