<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Organizations</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/contacts/organizations/create"
      >
        Create Organization
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.organizations"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/contacts/organizations/${item.id}/edit`"
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
        <v-card-title>Delete Organization</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingOrg?.name }}"?
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
import { useOrganizationsStore, type Organization } from "@/stores/admin/organizations";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("contacts.organizations.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("contacts.organizations.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("contacts.organizations.delete") || data.permission_type === "all");

const breadcrumbs = [
  { title: "Contacts", disabled: true },
  { title: "Organizations", disabled: true },
];

const store = useOrganizationsStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Assigned To", key: "user_name" },
  { title: "Created", key: "created_at", width: "180px" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingOrg = ref<Organization | null>(null);
const deleting = ref(false);

function confirmDelete(org: Organization) {
  deletingOrg.value = org;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingOrg.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingOrg.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
