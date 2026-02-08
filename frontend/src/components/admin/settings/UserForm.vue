<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit User" : "Create User" }}</h1>
    <v-card max-width="700">
      <v-card-text>
        <v-alert v-if="error" type="error" density="compact" class="mb-4">
          {{ error }}
        </v-alert>
        <v-form @submit.prevent="submit">
          <v-text-field v-model="form.first_name" label="First Name" required />
          <v-text-field v-model="form.last_name" label="Last Name" required class="mt-2" />
          <v-text-field v-model="form.email" label="Email" type="email" required class="mt-2" />
          <v-text-field
            v-model="form.password"
            label="Password"
            type="password"
            :required="!isEdit"
            :hint="isEdit ? 'Leave blank to keep current password' : ''"
            persistent-hint
            class="mt-2"
          />
          <v-text-field
            v-model="form.password_confirmation"
            label="Confirm Password"
            type="password"
            :required="!isEdit"
            class="mt-2"
          />
          <v-select
            v-model="form.role_id"
            :items="roles"
            item-title="name"
            item-value="id"
            label="Role"
            required
            class="mt-2"
          />
          <v-select
            v-model="form.view_permission"
            :items="viewPermissionOptions"
            label="View Permission"
            class="mt-2"
          />
          <v-select
            v-model="form.group_ids"
            :items="groups"
            item-title="name"
            item-value="id"
            label="Groups"
            multiple
            chips
            closable-chips
            class="mt-2"
          />
          <v-switch v-model="form.status" label="Active" class="mt-2" />
          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="loading">
              {{ isEdit ? "Update" : "Create" }}
            </v-btn>
            <v-btn href="/admin/settings/users" variant="outlined">Cancel</v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { useUsersStore } from "@/stores/admin/users";

const data = window.__INITIAL_DATA__ || {};

const store = useUsersStore();
const roles = data.roles || [];
const groups = data.groups || [];
const isEdit = !!data.user;
const error = ref("");
const loading = ref(false);

const viewPermissionOptions = [
  { title: "Global", value: "global" },
  { title: "Group", value: "group" },
  { title: "Individual", value: "individual" },
];

const form = reactive({
  first_name: data.user?.first_name || "",
  last_name: data.user?.last_name || "",
  email: data.user?.email || "",
  password: "",
  password_confirmation: "",
  status: data.user?.status ?? true,
  role_id: data.user?.role_id || null,
  view_permission: data.user?.view_permission || "global",
  group_ids: data.group_ids || [],
});

async function submit() {
  if (form.password && form.password !== form.password_confirmation) {
    error.value = "Passwords do not match";
    return;
  }
  error.value = "";
  loading.value = true;
  try {
    if (isEdit) {
      await store.update(data.user.id, form);
    } else {
      await store.create(form);
    }
    window.location.href = "/admin/settings/users";
  } catch (e: any) {
    error.value = e.message || "Failed to save";
  } finally {
    loading.value = false;
  }
}
</script>
