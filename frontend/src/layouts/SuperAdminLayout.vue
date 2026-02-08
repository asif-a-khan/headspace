<template>
  <v-app>
    <!-- Header -->
    <v-app-bar flat :height="62" color="surface" border="b" class="px-2">
      <template #prepend>
        <a href="/super/tenants" class="d-flex align-center text-decoration-none ml-2">
          <svg width="32" height="32" viewBox="0 0 32 32" class="mr-2">
            <rect width="32" height="32" rx="6" fill="#6366F1"/>
            <text x="16" y="23" text-anchor="middle" fill="white" font-size="20" font-weight="700" font-family="Inter, sans-serif">H</text>
          </svg>
          <span class="text-h6 font-weight-bold text-on-surface">Headspace</span>
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
            prepend-icon="mdi-account-circle"
            title="My Account"
            href="/super/account"
          />
          <v-divider />
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
        <!-- Tenants -->
        <v-list-item
          prepend-icon="mdi-domain"
          title="Tenants"
          href="/super/tenants"
          :active="isActive('/super/tenants')"
          :class="{ 'active-item': isActive('/super/tenants') }"
          rounded="lg"
        />

        <!-- Settings group with submenu -->
        <v-list-group value="settings">
          <template #activator="{ props }">
            <v-list-item
              v-bind="props"
              prepend-icon="mdi-cog"
              title="Settings"
              :active="isSettingsActive && !isActive('/super/settings/agents') && !isActive('/super/settings/roles')"
              :class="{ 'active-item': isSettingsActive && !isActive('/super/settings/agents') && !isActive('/super/settings/roles') }"
              rounded="lg"
            />
          </template>
          <v-list-item
            prepend-icon="mdi-account-group"
            title="Agents"
            href="/super/settings/agents"
            :active="isActive('/super/settings/agents')"
            :class="{ 'active-item': isActive('/super/settings/agents') }"
            rounded="lg"
          />
          <v-list-item
            prepend-icon="mdi-shield-lock"
            title="Roles"
            href="/super/settings/roles"
            :active="isActive('/super/settings/roles')"
            :class="{ 'active-item': isActive('/super/settings/roles') }"
            rounded="lg"
          />
        </v-list-group>
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

const theme = useTheme();
const data = (window as any).__INITIAL_DATA__ || {};
const adminName = computed(() => data.admin_name || "Admin");
const adminInitial = computed(() => (data.admin_name || "A").charAt(0).toUpperCase());
const flash = ref(data.flash || "");
const currentPath = window.location.pathname;
const sidebarExpanded = ref(true);

const isDark = computed(() => theme.global.current.value.dark);

// --- Breadcrumbs ---
const superLabelMap: Record<string, string> = {
  super: "",
  tenants: "Tenants",
  settings: "Settings",
  agents: "Agents",
  roles: "Roles",
  account: "My Account",
  create: "Create",
  edit: "Edit",
};

const breadcrumbs = computed(() => {
  const parts = currentPath.replace(/\/$/, "").split("/").filter(Boolean);
  const crumbs: Array<{ title: string; href?: string; disabled?: boolean }> = [
    { title: "Home", href: "/super/tenants" },
  ];
  let path = "";
  for (let i = 0; i < parts.length; i++) {
    const segment = parts[i];
    path += "/" + segment;
    if (segment === "super") continue;
    if (/^\d+$/.test(segment)) continue;
    const label = superLabelMap[segment] || segment.charAt(0).toUpperCase() + segment.slice(1);
    const isLast = i === parts.length - 1;
    crumbs.push({
      title: label,
      href: isLast ? undefined : path,
      disabled: isLast,
    });
  }
  return crumbs;
});

const isSettingsActive = computed(() => currentPath.startsWith("/super/settings"));

const openGroups = computed(() => {
  if (isSettingsActive.value) return ["settings"];
  return [];
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
  await fetch("/super/logout", {
    method: "POST",
    headers: { "X-CSRF-Token": token },
    credentials: "same-origin",
  });
  window.location.href = "/super/login";
}
</script>

