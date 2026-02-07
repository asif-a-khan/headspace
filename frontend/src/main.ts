import { createApp, h, type Component } from "vue";
import { createPinia } from "pinia";
import vuetify from "./plugins/vuetify";

// Layouts
import SuperAdminLayout from "./layouts/SuperAdminLayout.vue";
import TenantAdminLayout from "./layouts/TenantAdminLayout.vue";
import AnonymousLayout from "./layouts/AnonymousLayout.vue";

// Super admin page components
import LoginForm from "./components/super/LoginForm.vue";
import TenantList from "./components/super/TenantList.vue";
import TenantForm from "./components/super/TenantForm.vue";
import AgentList from "./components/super/AgentList.vue";
import AgentForm from "./components/super/AgentForm.vue";
import RoleList from "./components/super/RoleList.vue";
import RoleForm from "./components/super/RoleForm.vue";
import AccountForm from "./components/super/AccountForm.vue";
import SettingsIndex from "./components/super/SettingsIndex.vue";

// Tenant admin page components
import AdminLoginForm from "./components/admin/LoginForm.vue";
import AdminDashboard from "./components/admin/Dashboard.vue";

const pages: Record<string, { layout: Component; component: Component }> = {
  // Super admin pages
  login: { layout: AnonymousLayout, component: LoginForm },
  "tenant-list": { layout: SuperAdminLayout, component: TenantList },
  "tenant-form": { layout: SuperAdminLayout, component: TenantForm },
  "agent-list": { layout: SuperAdminLayout, component: AgentList },
  "agent-form": { layout: SuperAdminLayout, component: AgentForm },
  "role-list": { layout: SuperAdminLayout, component: RoleList },
  "role-form": { layout: SuperAdminLayout, component: RoleForm },
  "account-form": { layout: SuperAdminLayout, component: AccountForm },
  settings: { layout: SuperAdminLayout, component: SettingsIndex },
  // Tenant admin pages
  "admin-login": { layout: AnonymousLayout, component: AdminLoginForm },
  "admin-dashboard": { layout: TenantAdminLayout, component: AdminDashboard },
};

// Mount after all assets loaded (no blank page flash)
window.addEventListener("load", () => {
  const pageName = window.__PAGE__ || "login";
  const page = pages[pageName];

  if (!page) {
    console.error(`Unknown page: ${pageName}`);
    return;
  }

  const app = createApp({
    render() {
      return h(page.layout, null, {
        default: () => h(page.component),
      });
    },
  });

  app.use(createPinia());
  app.use(vuetify);
  app.mount("#app");
});
