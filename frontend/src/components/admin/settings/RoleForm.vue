<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Role" : "Create Role" }}</h1>
    <v-card max-width="700">
      <v-card-text>
        <v-alert v-if="error" type="error" density="compact" class="mb-4">
          {{ error }}
        </v-alert>
        <v-form @submit.prevent="submit">
          <v-text-field v-model="form.name" label="Name" required />
          <v-textarea v-model="form.description" label="Description" rows="2" class="mt-2" />
          <v-radio-group v-model="form.permission_type" label="Permission Type" class="mt-2">
            <v-radio label="All Permissions" value="all" />
            <v-radio label="Custom Permissions" value="custom" />
          </v-radio-group>

          <div v-if="form.permission_type === 'custom'" class="mt-2">
            <div v-for="node in acl" :key="node.key" :style="{ paddingLeft: node.depth * 24 + 'px' }">
              <v-checkbox
                v-model="form.permissions"
                :value="node.key"
                :label="node.name"
                density="compact"
                hide-details
              />
            </div>
          </div>

          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="loading">
              {{ isEdit ? "Update" : "Create" }}
            </v-btn>
            <v-btn href="/admin/settings/roles" variant="outlined">Cancel</v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { useRolesStore } from "@/stores/admin/roles";

const data = window.__INITIAL_DATA__ || {};

const store = useRolesStore();
store.hydrate(data);
const acl = data.acl || [];
const isEdit = !!data.role;
const error = ref("");
const loading = ref(false);

const form = reactive({
  name: data.role?.name || "",
  description: data.role?.description || "",
  permission_type: data.role?.permission_type || "custom",
  permissions: data.role?.permissions || [],
});

async function submit() {
  error.value = "";
  loading.value = true;
  try {
    if (isEdit) {
      await store.update(data.role.id, form);
    } else {
      await store.create(form);
    }
    window.location.href = "/admin/settings/roles";
  } catch (e: any) {
    error.value = e.message || "Failed to save";
  } finally {
    loading.value = false;
  }
}
</script>
