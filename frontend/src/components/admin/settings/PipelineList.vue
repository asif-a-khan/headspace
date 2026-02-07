<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Pipelines</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/settings/pipelines/create"
      >
        Create Pipeline
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.pipelines"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.is_default="{ item }">
        <v-chip v-if="item.is_default" color="primary" size="small">Default</v-chip>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/settings/pipelines/${item.id}/edit`"
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
        <v-card-title>Delete Pipeline</v-card-title>
        <v-card-text>Are you sure you want to delete "{{ deletingItem?.name }}"?</v-card-text>
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
import { usePipelinesStore, type Pipeline } from "@/stores/admin/pipelines";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("settings.pipelines.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("settings.pipelines.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("settings.pipelines.delete") || data.permission_type === "all");

const breadcrumbs = [
  { title: "Settings", href: "/admin/settings" },
  { title: "Pipelines", disabled: true },
];

const store = usePipelinesStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Default", key: "is_default", width: "100px" },
  { title: "Rotten Days", key: "rotten_days", width: "120px" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingItem = ref<Pipeline | null>(null);
const deleting = ref(false);

function confirmDelete(item: Pipeline) {
  deletingItem.value = item;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingItem.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingItem.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
