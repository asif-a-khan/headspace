<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Attributes</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/settings/attributes/create"
      >
        Create Attribute
      </v-btn>
    </div>

    <v-tabs v-model="activeTab" class="mb-4">
      <v-tab v-for="et in entityTypes" :key="et.value" :value="et.value">
        {{ et.label }}
      </v-tab>
    </v-tabs>

    <v-data-table
      :headers="headers"
      :items="filteredAttributes"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.type="{ item }">
        <v-chip size="small">{{ item.type }}</v-chip>
      </template>
      <template #item.is_required="{ item }">
        <v-icon v-if="item.is_required" color="success" size="small">mdi-check</v-icon>
      </template>
      <template #item.is_user_defined="{ item }">
        <v-chip :color="item.is_user_defined ? 'default' : 'info'" size="small">
          {{ item.is_user_defined ? "Custom" : "System" }}
        </v-chip>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit && item.is_user_defined"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/settings/attributes/${item.id}/edit`"
        />
        <v-btn
          v-if="canDelete && item.is_user_defined"
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
        <v-card-title>Delete Attribute</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingAttr?.name }}"?
          This will also remove all stored values for this attribute.
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
import { useAttributesStore, type Attribute } from "@/stores/admin/attributes";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("settings.attributes.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("settings.attributes.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("settings.attributes.delete") || data.permission_type === "all");

const entityTypes = [
  { label: "Leads", value: "leads" },
  { label: "Persons", value: "persons" },
  { label: "Organizations", value: "organizations" },
  { label: "Products", value: "products" },
  { label: "Quotes", value: "quotes" },
];

const activeTab = ref("leads");

const store = useAttributesStore();
store.hydrate(data);

const filteredAttributes = computed(() =>
  store.attributes.filter((a) => a.entity_type === activeTab.value)
);

const headers = [
  { title: "Code", key: "code" },
  { title: "Name", key: "name" },
  { title: "Type", key: "type", width: "120px" },
  { title: "Required", key: "is_required", width: "100px" },
  { title: "Source", key: "is_user_defined", width: "100px" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingAttr = ref<Attribute | null>(null);
const deleting = ref(false);

function confirmDelete(attr: Attribute) {
  deletingAttr.value = attr;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingAttr.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingAttr.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
