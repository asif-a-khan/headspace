<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Agent" : "Create Agent" }}</h1>
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
          <v-switch v-model="form.status" label="Active" class="mt-2" />
          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="loading">
              {{ isEdit ? "Update" : "Create" }}
            </v-btn>
            <v-btn href="/super/settings/agents" variant="text">Cancel</v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { useAgentsStore } from "@/stores/super/agents";

const data = window.__INITIAL_DATA__ || {};
const store = useAgentsStore();
store.hydrate(data);
const roles = data.roles || [];
const isEdit = !!data.agent;
const error = ref("");
const loading = ref(false);

const form = reactive({
  first_name: data.agent?.first_name || "",
  last_name: data.agent?.last_name || "",
  email: data.agent?.email || "",
  password: "",
  password_confirmation: "",
  status: data.agent?.status ?? true,
  role_id: data.agent?.role_id || null,
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
      await store.update(data.agent.id, form);
    } else {
      await store.create(form);
    }
    window.location.href = "/super/settings/agents";
  } catch (e: any) {
    error.value = e.message || "Failed to save";
  } finally {
    loading.value = false;
  }
}
</script>
