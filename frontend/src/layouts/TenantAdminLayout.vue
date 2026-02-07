<template>
  <v-app>
    <!-- Header -->
    <v-app-bar flat :height="60" color="surface" border="b" class="px-2">
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

      <!-- Theme toggle -->
      <v-btn
        icon
        variant="text"
        @click="toggleTheme"
      >
        <v-icon>{{ isDark ? 'mdi-weather-sunny' : 'mdi-weather-night' }}</v-icon>
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
      :width="220"
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
          disabled
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
      <v-container fluid class="pa-6">
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
        <slot />
      </v-container>
    </v-main>
  </v-app>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useTheme } from "vuetify";

const theme = useTheme();
const data = (window as any).__INITIAL_DATA__ || {};
const adminName = computed(() => data.admin_name || "Admin");
const adminInitial = computed(() => (data.admin_name || "A").charAt(0).toUpperCase());
const companyName = computed(() => data.company_name || "Headspace");
const flash = ref(data.flash || "");
const currentPath = window.location.pathname;
const sidebarExpanded = ref(true);

const isDark = computed(() => theme.global.current.value.dark);

const openGroups = computed(() => {
  const groups: string[] = [];
  if (currentPath.startsWith("/admin/contacts")) groups.push("contacts");
  return groups;
});

function isActive(path: string): boolean {
  return currentPath.startsWith(path);
}

function toggleTheme() {
  const next = isDark.value ? "light" : "dark";
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
.active-item {
  background-color: rgb(var(--v-theme-primary)) !important;
  color: white !important;
}
.active-item :deep(.v-icon) {
  color: white !important;
}
.active-item :deep(.v-list-item-title) {
  color: white !important;
}

.sidebar-list :deep(.v-list-group__items .v-list-item) {
  padding-inline-start: 16px !important;
}
</style>
