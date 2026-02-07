<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Lead Sources</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        @click="openDialog()"
      >
        Create Source
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.sources"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          @click="openDialog(item)"
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

    <!-- Create/Edit dialog -->
    <v-dialog v-model="formDialog" max-width="400">
      <v-card>
        <v-card-title>{{ editingItem ? "Edit Source" : "Create Source" }}</v-card-title>
        <v-card-text>
          <v-text-field v-model="formName" label="Name" required autofocus />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="formDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="saveItem" :loading="saving">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Source</v-card-title>
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
import { useSourcesStore, type Source } from "@/stores/admin/sources";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("settings.sources.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("settings.sources.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("settings.sources.delete") || data.permission_type === "all");

const breadcrumbs = [
  { title: "Settings", href: "/admin/settings" },
  { title: "Sources", disabled: true },
];

const store = useSourcesStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

// Form dialog
const formDialog = ref(false);
const formName = ref("");
const editingItem = ref<Source | null>(null);
const saving = ref(false);

function openDialog(item?: Source) {
  editingItem.value = item || null;
  formName.value = item?.name || "";
  formDialog.value = true;
}

async function saveItem() {
  saving.value = true;
  try {
    if (editingItem.value) {
      await store.update(editingItem.value.id, { name: formName.value });
    } else {
      await store.create({ name: formName.value });
    }
    formDialog.value = false;
  } finally {
    saving.value = false;
  }
}

// Delete dialog
const deleteDialog = ref(false);
const deletingItem = ref<Source | null>(null);
const deleting = ref(false);

function confirmDelete(item: Source) {
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
