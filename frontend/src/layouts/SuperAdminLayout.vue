<template>
  <v-app>
    <v-navigation-drawer permanent>
      <v-list-item
        :title="adminName"
        subtitle="Super Admin"
        prepend-icon="mdi-shield-crown"
        class="mb-2"
      />
      <v-divider />
      <v-list density="compact" nav>
        <v-list-item
          prepend-icon="mdi-domain"
          title="Tenants"
          href="/super/tenants"
          :active="currentPath.startsWith('/super/tenants')"
        />
        <v-list-item
          prepend-icon="mdi-cog"
          title="Settings"
          href="/super/settings"
          :active="currentPath.startsWith('/super/settings')"
        />
        <v-list-item
          prepend-icon="mdi-account-circle"
          title="My Account"
          href="/super/account"
          :active="currentPath === '/super/account'"
        />
      </v-list>
    </v-navigation-drawer>

    <v-app-bar flat density="compact">
      <v-app-bar-title>Headspace</v-app-bar-title>
      <template #append>
        <v-btn
          icon="mdi-logout"
          @click="logout"
          title="Logout"
        />
      </template>
    </v-app-bar>

    <v-main>
      <v-container fluid>
        <v-alert
          v-if="flash"
          type="success"
          closable
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

const data = window.__INITIAL_DATA__ || {};
const adminName = computed(() => data.admin_name || "Admin");
const flash = ref(data.flash || "");
const currentPath = window.location.pathname;

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
