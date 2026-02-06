<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Roles</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/super/settings/roles/create"
      >
        Create Role
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.roles"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.permission_type="{ item }">
        <v-chip :color="item.permission_type === 'all' ? 'primary' : 'default'" size="small">
          {{ item.permission_type === "all" ? "All" : "Custom" }}
        </v-chip>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/super/settings/roles/${item.id}/edit`"
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
        <v-card-title>Delete Role</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingRole?.name }}"?
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
import { useRolesStore, type Role } from "@/stores/super/roles";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("settings.roles.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("settings.roles.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("settings.roles.delete") || data.permission_type === "all");

const store = useRolesStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Description", key: "description" },
  { title: "Type", key: "permission_type", width: "120px" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingRole = ref<Role | null>(null);
const deleting = ref(false);

function confirmDelete(role: Role) {
  deletingRole.value = role;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingRole.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingRole.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
