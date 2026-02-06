<template>
  <div>
    <h1 class="text-h5 mb-4">Settings</h1>
    <v-row>
      <v-col cols="12" md="6">
        <v-card hover href="/super/settings/agents">
          <v-card-item>
            <template #prepend>
              <v-icon icon="mdi-account-group" size="40" color="primary" />
            </template>
            <v-card-title>Agents</v-card-title>
            <v-card-subtitle>Manage super admin users</v-card-subtitle>
          </v-card-item>
        </v-card>
      </v-col>
      <v-col cols="12" md="6">
        <v-card hover href="/super/settings/roles">
          <v-card-item>
            <template #prepend>
              <v-icon icon="mdi-shield-lock" size="40" color="primary" />
            </template>
            <v-card-title>Roles</v-card-title>
            <v-card-subtitle>Manage roles and permissions</v-card-subtitle>
          </v-card-item>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>
