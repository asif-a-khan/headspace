<template>
  <div>
    <h1 class="text-h5 font-weight-bold mb-6">Settings</h1>

    <div v-for="(category, idx) in categories" :key="category.title" :class="{ 'mt-8': idx > 0 }">
      <div class="mb-3">
        <h2 class="text-h6">{{ category.title }}</h2>
        <p class="text-body-2 text-medium-emphasis">{{ category.description }}</p>
      </div>
      <v-row>
        <v-col v-for="card in category.cards" :key="card.href" cols="12" md="4">
          <v-card hover :href="card.href">
            <v-card-item>
              <template #prepend>
                <v-icon :icon="card.icon" size="40" color="primary" />
              </template>
              <v-card-title>{{ card.title }}</v-card-title>
              <v-card-subtitle>{{ card.subtitle }}</v-card-subtitle>
            </v-card-item>
          </v-card>
        </v-col>
      </v-row>
    </div>
  </div>
</template>

<script setup lang="ts">
interface SettingsCard {
  title: string;
  subtitle: string;
  icon: string;
  href: string;
}

interface SettingsCategory {
  title: string;
  description: string;
  cards: SettingsCard[];
}

const categories: SettingsCategory[] = [
  {
    title: "User Management",
    description: "Manage users and their permissions",
    cards: [
      { title: "Groups", subtitle: "Manage user groups for data scoping", icon: "mdi-account-multiple-check", href: "/admin/settings/groups" },
      { title: "Roles", subtitle: "Manage roles and permissions", icon: "mdi-shield-lock", href: "/admin/settings/roles" },
      { title: "Users", subtitle: "Manage team members", icon: "mdi-account-group", href: "/admin/settings/users" },
    ],
  },
  {
    title: "Lead Management",
    description: "Manage lead-related settings",
    cards: [
      { title: "Pipelines", subtitle: "Configure sales pipelines and stages", icon: "mdi-pipe", href: "/admin/settings/pipelines" },
      { title: "Sources", subtitle: "Manage lead sources", icon: "mdi-source-branch", href: "/admin/settings/sources" },
      { title: "Types", subtitle: "Manage lead types", icon: "mdi-tag-multiple", href: "/admin/settings/types" },
      { title: "Web Forms", subtitle: "Embeddable forms for lead capture", icon: "mdi-form-select", href: "/admin/settings/web-forms" },
    ],
  },
  {
    title: "Automation",
    description: "Templates and automation tools",
    cards: [
      { title: "Attributes", subtitle: "Manage custom fields for entities", icon: "mdi-format-list-bulleted-type", href: "/admin/settings/attributes" },
      { title: "Email Templates", subtitle: "Reusable email templates", icon: "mdi-email-edit-outline", href: "/admin/settings/email-templates" },
    ],
  },
  {
    title: "Warehouse",
    description: "Manage warehouses and locations",
    cards: [
      { title: "Warehouses", subtitle: "Manage warehouse locations and contacts", icon: "mdi-warehouse", href: "/admin/settings/warehouses" },
    ],
  },
  {
    title: "Other Settings",
    description: "Tags, configuration and more",
    cards: [
      { title: "Tags", subtitle: "Manage tags for categorizing records", icon: "mdi-tag-multiple-outline", href: "/admin/settings/tags" },
      { title: "Configuration", subtitle: "Currency, date format, email, appearance", icon: "mdi-cog", href: "/admin/settings/configuration" },
    ],
  },
];
</script>
