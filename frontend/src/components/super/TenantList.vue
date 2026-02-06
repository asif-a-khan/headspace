<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Tenants</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/super/tenants/create"
      >
        Create Tenant
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.tenants"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.is_active="{ item }">
        <v-chip :color="item.is_active ? 'success' : 'error'" size="small">
          {{ item.is_active ? "Active" : "Inactive" }}
        </v-chip>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/super/tenants/${item.id}/edit`"
        />
        <v-btn
          v-if="canDelete"
          icon="mdi-delete"
          size="small"
          variant="text"
          color="error"
          @click="confirmDelete(item)"
        />
      </template>
    </v-data-table>

    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Tenant</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingTenant?.name }}"?
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" @click="doDelete" :loading="deleting">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useTenantsStore, type Tenant } from "@/stores/super/tenants";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("tenants.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("tenants.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("tenants.delete") || data.permission_type === "all");

const store = useTenantsStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Domain", key: "domain" },
  { title: "Email", key: "email" },
  { title: "Status", key: "is_active", width: "100px" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingTenant = ref<Tenant | null>(null);
const deleting = ref(false);

function confirmDelete(tenant: Tenant) {
  deletingTenant.value = tenant;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingTenant.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingTenant.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
