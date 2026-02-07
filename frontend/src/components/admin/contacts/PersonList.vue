<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Persons</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/contacts/persons/create"
      >
        Create Person
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.persons"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.emails="{ item }">
        <span v-if="item.emails && item.emails.length">{{ item.emails[0].value }}</span>
      </template>
      <template #item.contact_numbers="{ item }">
        <span v-if="item.contact_numbers && item.contact_numbers.length">{{ item.contact_numbers[0].value }}</span>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/contacts/persons/${item.id}/edit`"
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
        <v-card-title>Delete Person</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingPerson?.name }}"?
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
import { usePersonsStore, type Person } from "@/stores/admin/persons";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("contacts.persons.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("contacts.persons.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("contacts.persons.delete") || data.permission_type === "all");

const breadcrumbs = [
  { title: "Contacts", disabled: true },
  { title: "Persons", disabled: true },
];

const store = usePersonsStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Email", key: "emails" },
  { title: "Phone", key: "contact_numbers" },
  { title: "Organization", key: "organization_name" },
  { title: "Assigned To", key: "user_name" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingPerson = ref<Person | null>(null);
const deleting = ref(false);

function confirmDelete(person: Person) {
  deletingPerson.value = person;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingPerson.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingPerson.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
