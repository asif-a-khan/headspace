<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Warehouses</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/settings/warehouses/create"
      >
        Create Warehouse
      </v-btn>
    </div>

    <v-card>
      <v-data-table
        :headers="headers"
        :items="store.warehouses"
        :loading="store.loading"
        hover
      >
        <template #item.actions="{ item }">
          <v-btn
            icon="mdi-pencil"
            size="small"
            variant="text"
            :href="`/admin/settings/warehouses/${item.id}/edit`"
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
    </v-card>

    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Warehouse</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deleteTarget?.name }}"?
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" :loading="deleting" @click="doDelete">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="errorSnackbar" color="error" :timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useWarehousesStore, type Warehouse } from "@/stores/admin/warehouses";

const data = window.__INITIAL_DATA__ || {};
const store = useWarehousesStore();
store.hydrate(data);

const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("settings.warehouses.create") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("settings.warehouses.delete") || data.permission_type === "all");

const headers = [
  { title: "Name", key: "name" },
  { title: "Description", key: "description" },
  { title: "Contact", key: "contact_name" },
  { title: "Actions", key: "actions", sortable: false, align: "end" as const },
];

const deleteDialog = ref(false);
const deleting = ref(false);
const deleteTarget = ref<Warehouse | null>(null);
const errorSnackbar = ref(false);
const errorMessage = ref("");

function confirmDelete(item: Warehouse) {
  deleteTarget.value = item;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deleteTarget.value) return;
  deleting.value = true;
  try {
    await store.remove(deleteTarget.value.id);
    deleteDialog.value = false;
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to delete warehouse.";
    errorSnackbar.value = true;
  } finally {
    deleting.value = false;
  }
}
</script>
