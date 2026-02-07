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
import AdminSettingsIndex from "./components/admin/settings/SettingsIndex.vue";
import AdminUserList from "./components/admin/settings/UserList.vue";
import AdminUserForm from "./components/admin/settings/UserForm.vue";
import AdminRoleList from "./components/admin/settings/RoleList.vue";
import AdminRoleForm from "./components/admin/settings/RoleForm.vue";
import AdminGroupList from "./components/admin/settings/GroupList.vue";
import AdminGroupForm from "./components/admin/settings/GroupForm.vue";
import AdminAttributeList from "./components/admin/settings/AttributeList.vue";
import AdminAttributeForm from "./components/admin/settings/AttributeForm.vue";
import AdminPipelineList from "./components/admin/settings/PipelineList.vue";
import AdminPipelineForm from "./components/admin/settings/PipelineForm.vue";
import AdminSourceList from "./components/admin/settings/SourceList.vue";
import AdminTypeList from "./components/admin/settings/TypeList.vue";
import AdminQuoteList from "./components/admin/quotes/QuoteList.vue";
import AdminQuoteBuilder from "./components/admin/quotes/QuoteBuilder.vue";
import AdminActivityList from "./components/admin/activities/ActivityList.vue";
import AdminActivityForm from "./components/admin/activities/ActivityForm.vue";
import AdminProductList from "./components/admin/products/ProductList.vue";
import AdminProductForm from "./components/admin/products/ProductForm.vue";
import AdminLeadList from "./components/admin/leads/LeadList.vue";
import AdminLeadForm from "./components/admin/leads/LeadForm.vue";
import AdminLeadKanban from "./components/admin/leads/LeadKanban.vue";
import AdminPersonList from "./components/admin/contacts/PersonList.vue";
import AdminPersonForm from "./components/admin/contacts/PersonForm.vue";
import AdminOrganizationList from "./components/admin/contacts/OrganizationList.vue";
import AdminOrganizationForm from "./components/admin/contacts/OrganizationForm.vue";

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
  "admin-settings": { layout: TenantAdminLayout, component: AdminSettingsIndex },
  "admin-user-list": { layout: TenantAdminLayout, component: AdminUserList },
  "admin-user-form": { layout: TenantAdminLayout, component: AdminUserForm },
  "admin-role-list": { layout: TenantAdminLayout, component: AdminRoleList },
  "admin-role-form": { layout: TenantAdminLayout, component: AdminRoleForm },
  "admin-group-list": { layout: TenantAdminLayout, component: AdminGroupList },
  "admin-group-form": { layout: TenantAdminLayout, component: AdminGroupForm },
  "admin-attribute-list": { layout: TenantAdminLayout, component: AdminAttributeList },
  "admin-attribute-form": { layout: TenantAdminLayout, component: AdminAttributeForm },
  "admin-pipeline-list": { layout: TenantAdminLayout, component: AdminPipelineList },
  "admin-pipeline-form": { layout: TenantAdminLayout, component: AdminPipelineForm },
  "admin-source-list": { layout: TenantAdminLayout, component: AdminSourceList },
  "admin-type-list": { layout: TenantAdminLayout, component: AdminTypeList },
  "admin-quote-list": { layout: TenantAdminLayout, component: AdminQuoteList },
  "admin-quote-form": { layout: TenantAdminLayout, component: AdminQuoteBuilder },
  "admin-activity-list": { layout: TenantAdminLayout, component: AdminActivityList },
  "admin-activity-form": { layout: TenantAdminLayout, component: AdminActivityForm },
  "admin-product-list": { layout: TenantAdminLayout, component: AdminProductList },
  "admin-product-form": { layout: TenantAdminLayout, component: AdminProductForm },
  "admin-lead-list": { layout: TenantAdminLayout, component: AdminLeadList },
  "admin-lead-form": { layout: TenantAdminLayout, component: AdminLeadForm },
  "admin-lead-kanban": { layout: TenantAdminLayout, component: AdminLeadKanban },
  "admin-person-list": { layout: TenantAdminLayout, component: AdminPersonList },
  "admin-person-form": { layout: TenantAdminLayout, component: AdminPersonForm },
  "admin-organization-list": { layout: TenantAdminLayout, component: AdminOrganizationList },
  "admin-organization-form": { layout: TenantAdminLayout, component: AdminOrganizationForm },
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
