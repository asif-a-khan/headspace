import { createApp, h, type Component } from "vue";
import { createPinia } from "pinia";
import vuetify from "./plugins/vuetify";

// Layouts (eagerly loaded — shared across all pages)
import SuperAdminLayout from "./layouts/SuperAdminLayout.vue";
import TenantAdminLayout from "./layouts/TenantAdminLayout.vue";
import AnonymousLayout from "./layouts/AnonymousLayout.vue";

type LazyComponent = () => Promise<{ default: Component }>;

const pages: Record<string, { layout: Component; component: LazyComponent }> = {
  // Super admin pages
  login: { layout: AnonymousLayout, component: () => import("./components/super/LoginForm.vue") },
  "tenant-list": { layout: SuperAdminLayout, component: () => import("./components/super/TenantList.vue") },
  "tenant-form": { layout: SuperAdminLayout, component: () => import("./components/super/TenantForm.vue") },
  "agent-list": { layout: SuperAdminLayout, component: () => import("./components/super/AgentList.vue") },
  "agent-form": { layout: SuperAdminLayout, component: () => import("./components/super/AgentForm.vue") },
  "role-list": { layout: SuperAdminLayout, component: () => import("./components/super/RoleList.vue") },
  "role-form": { layout: SuperAdminLayout, component: () => import("./components/super/RoleForm.vue") },
  "account-form": { layout: SuperAdminLayout, component: () => import("./components/super/AccountForm.vue") },
  settings: { layout: SuperAdminLayout, component: () => import("./components/super/SettingsIndex.vue") },
  // Tenant admin pages
  "admin-login": { layout: AnonymousLayout, component: () => import("./components/admin/LoginForm.vue") },
  "admin-dashboard": { layout: TenantAdminLayout, component: () => import("./components/admin/Dashboard.vue") },
  "admin-settings": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/SettingsIndex.vue") },
  "admin-user-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/UserList.vue") },
  "admin-user-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/UserForm.vue") },
  "admin-role-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/RoleList.vue") },
  "admin-role-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/RoleForm.vue") },
  "admin-group-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/GroupList.vue") },
  "admin-group-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/GroupForm.vue") },
  "admin-attribute-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/AttributeList.vue") },
  "admin-attribute-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/AttributeForm.vue") },
  "admin-pipeline-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/PipelineList.vue") },
  "admin-pipeline-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/PipelineForm.vue") },
  "admin-source-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/SourceList.vue") },
  "admin-type-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/TypeList.vue") },
  "admin-tag-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/TagList.vue") },
  "admin-configuration": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/ConfigurationForm.vue") },
  "admin-email-template-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/EmailTemplateList.vue") },
  "admin-email-template-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/EmailTemplateForm.vue") },
  "admin-warehouse-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/WarehouseList.vue") },
  "admin-warehouse-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/WarehouseForm.vue") },
  "admin-web-form-list": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/WebFormList.vue") },
  "admin-web-form-form": { layout: TenantAdminLayout, component: () => import("./components/admin/settings/WebFormForm.vue") },
  "admin-quote-list": { layout: TenantAdminLayout, component: () => import("./components/admin/quotes/QuoteList.vue") },
  "admin-quote-form": { layout: TenantAdminLayout, component: () => import("./components/admin/quotes/QuoteBuilder.vue") },
  "admin-activity-list": { layout: TenantAdminLayout, component: () => import("./components/admin/activities/ActivityList.vue") },
  "admin-activity-form": { layout: TenantAdminLayout, component: () => import("./components/admin/activities/ActivityForm.vue") },
  "admin-product-list": { layout: TenantAdminLayout, component: () => import("./components/admin/products/ProductList.vue") },
  "admin-product-form": { layout: TenantAdminLayout, component: () => import("./components/admin/products/ProductForm.vue") },
  "admin-lead-list": { layout: TenantAdminLayout, component: () => import("./components/admin/leads/LeadList.vue") },
  "admin-lead-form": { layout: TenantAdminLayout, component: () => import("./components/admin/leads/LeadForm.vue") },
  "admin-lead-kanban": { layout: TenantAdminLayout, component: () => import("./components/admin/leads/LeadKanban.vue") },
  "admin-lead-detail": { layout: TenantAdminLayout, component: () => import("./components/admin/leads/LeadDetail.vue") },
  "admin-person-list": { layout: TenantAdminLayout, component: () => import("./components/admin/contacts/PersonList.vue") },
  "admin-person-form": { layout: TenantAdminLayout, component: () => import("./components/admin/contacts/PersonForm.vue") },
  "admin-person-detail": { layout: TenantAdminLayout, component: () => import("./components/admin/contacts/PersonDetail.vue") },
  "admin-organization-list": { layout: TenantAdminLayout, component: () => import("./components/admin/contacts/OrganizationList.vue") },
  "admin-organization-form": { layout: TenantAdminLayout, component: () => import("./components/admin/contacts/OrganizationForm.vue") },
  "admin-account": { layout: TenantAdminLayout, component: () => import("./components/admin/AccountForm.vue") },
  "admin-email": { layout: TenantAdminLayout, component: () => import("./components/admin/EmailClient.vue") },
};

// Mount after all assets loaded (no blank page flash)
window.addEventListener("load", async () => {
  const pageName = window.__PAGE__ || "login";
  const page = pages[pageName];

  if (!page) {
    console.error(`Unknown page: ${pageName}`);
    return;
  }

  // Dynamically load only the page component needed
  const mod = await page.component();
  const PageComponent = mod.default;

  const app = createApp({
    render() {
      return h(page.layout, null, {
        default: () => h(PageComponent),
      });
    },
  });

  app.use(createPinia());
  app.use(vuetify);
  app.mount("#app");
});
