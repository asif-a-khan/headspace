<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Groups</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/settings/groups/create"
      >
        Create Group
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.groups"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/settings/groups/${item.id}/edit`"
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
        <v-card-title>Delete Group</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingGroup?.name }}"?
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
import { useGroupsStore, type Group } from "@/stores/admin/groups";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("settings.groups.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("settings.groups.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("settings.groups.delete") || data.permission_type === "all");

const breadcrumbs = [
  { title: "Settings", href: "/admin/settings" },
  { title: "Groups", disabled: true },
];

const store = useGroupsStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Description", key: "description" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingGroup = ref<Group | null>(null);
const deleting = ref(false);

function confirmDelete(group: Group) {
  deletingGroup.value = group;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingGroup.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingGroup.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
