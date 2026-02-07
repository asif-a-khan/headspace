<template>
  <div>
    <h1 class="text-h5 mb-4">Settings</h1>
    <v-row>
      <v-col cols="12" md="4">
        <v-card hover href="/admin/settings/users">
          <v-card-item>
            <template #prepend>
              <v-icon icon="mdi-account-group" size="40" color="primary" />
            </template>
            <v-card-title>Users</v-card-title>
            <v-card-subtitle>Manage team members</v-card-subtitle>
          </v-card-item>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card hover href="/admin/settings/roles">
          <v-card-item>
            <template #prepend>
              <v-icon icon="mdi-shield-lock" size="40" color="primary" />
            </template>
            <v-card-title>Roles</v-card-title>
            <v-card-subtitle>Manage roles and permissions</v-card-subtitle>
          </v-card-item>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card hover href="/admin/settings/groups">
          <v-card-item>
            <template #prepend>
              <v-icon icon="mdi-account-multiple-check" size="40" color="primary" />
            </template>
            <v-card-title>Groups</v-card-title>
            <v-card-subtitle>Manage user groups for data scoping</v-card-subtitle>
          </v-card-item>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>
