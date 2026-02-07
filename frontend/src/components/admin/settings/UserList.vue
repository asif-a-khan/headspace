<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Users</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/settings/users/create"
      >
        Create User
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.users"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.name="{ item }">
        {{ item.first_name }} {{ item.last_name }}
      </template>
      <template #item.status="{ item }">
        <v-chip :color="item.status ? 'success' : 'error'" size="small">
          {{ item.status ? "Active" : "Inactive" }}
        </v-chip>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/settings/users/${item.id}/edit`"
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
        <v-card-title>Delete User</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingUser?.first_name }} {{ deletingUser?.last_name }}"?
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
import { useUsersStore, type User } from "@/stores/admin/users";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("settings.users.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("settings.users.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("settings.users.delete") || data.permission_type === "all");

const breadcrumbs = [
  { title: "Settings", href: "/admin/settings" },
  { title: "Users", disabled: true },
];

const store = useUsersStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Email", key: "email" },
  { title: "Status", key: "status", width: "100px" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingUser = ref<User | null>(null);
const deleting = ref(false);

function confirmDelete(user: User) {
  deletingUser.value = user;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingUser.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingUser.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
