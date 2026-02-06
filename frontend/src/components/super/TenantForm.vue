<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Tenant" : "Create Tenant" }}</h1>
    <v-card max-width="700">
      <v-card-text>
        <v-alert v-if="error" type="error" density="compact" class="mb-4">
          {{ error }}
        </v-alert>
        <v-form @submit.prevent="submit">
          <v-text-field v-model="form.name" label="Name" required />
          <v-text-field
            v-model="form.domain"
            label="Domain (subdomain slug)"
            required
            :disabled="isEdit"
            hint="e.g. 'acme' → acme.headspace.local"
            persistent-hint
            class="mt-2"
          />
          <v-text-field v-model="form.email" label="Email" type="email" class="mt-2" />
          <v-text-field v-model="form.cname" label="Custom Domain (CNAME)" class="mt-2" />
          <v-textarea v-model="form.description" label="Description" rows="3" class="mt-2" />
          <v-switch v-model="form.is_active" label="Active" class="mt-2" />
          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="loading">
              {{ isEdit ? "Update" : "Create" }}
            </v-btn>
            <v-btn href="/super/tenants" variant="text">Cancel</v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { useTenantsStore } from "@/stores/super/tenants";

const data = window.__INITIAL_DATA__ || {};
const store = useTenantsStore();
const isEdit = !!data.tenant;
const error = ref("");
const loading = ref(false);

const form = reactive({
  name: data.tenant?.name || "",
  domain: data.tenant?.domain || "",
  email: data.tenant?.email || "",
  cname: data.tenant?.cname || "",
  description: data.tenant?.description || "",
  is_active: data.tenant?.is_active ?? true,
});

async function submit() {
  error.value = "";
  loading.value = true;
  try {
    if (isEdit) {
      await store.update(data.tenant.id, form);
    } else {
      await store.create(form);
    }
    window.location.href = "/super/tenants";
  } catch (e: any) {
    error.value = e.message || "Failed to save";
  } finally {
    loading.value = false;
  }
}
</script>
