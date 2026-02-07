<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Leads</h1>
      <v-spacer />
      <v-btn
        variant="outlined"
        class="mr-2"
        href="/admin/leads/kanban"
      >
        <v-icon start>mdi-view-column</v-icon>
        Kanban
      </v-btn>
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/leads/create"
      >
        Create Lead
      </v-btn>
    </div>

    <v-card class="mb-4">
      <v-card-text class="d-flex ga-3">
        <v-select
          v-model="pipelineFilter"
          :items="pipelineItems"
          item-title="name"
          item-value="id"
          label="Pipeline"
          density="compact"
          clearable
          hide-details
          style="max-width: 240px"
          @update:model-value="filterByPipeline"
        />
      </v-card-text>
    </v-card>

    <v-data-table
      :headers="headers"
      :items="store.leads"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.title="{ item }">
        <a :href="`/admin/leads/${item.id}/edit`" class="text-decoration-none font-weight-medium">
          {{ item.title }}
        </a>
      </template>
      <template #item.lead_value="{ item }">
        {{ item.lead_value ? `$${Number(item.lead_value).toLocaleString()}` : '-' }}
      </template>
      <template #item.status="{ item }">
        <v-chip v-if="item.status === null" color="info" size="small">Open</v-chip>
        <v-chip v-else-if="item.status" color="success" size="small">Won</v-chip>
        <v-chip v-else color="error" size="small">Lost</v-chip>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/leads/${item.id}/edit`"
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
        <v-card-title>Delete Lead</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingLead?.title }}"?
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
import { useLeadsStore, type Lead } from "@/stores/admin/leads";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("leads.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("leads.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("leads.delete") || data.permission_type === "all");

const store = useLeadsStore();
store.hydrate(data);

const pipelineItems = computed(() => data.pipelines || []);
const pipelineFilter = ref<number | null>(null);

function filterByPipeline(pipelineId: number | null) {
  if (pipelineId) {
    store.fetchAll(pipelineId);
  } else {
    store.fetchAll();
  }
}

const headers = [
  { title: "ID", key: "id", width: "70px" },
  { title: "Title", key: "title" },
  { title: "Value", key: "lead_value", width: "120px" },
  { title: "Contact", key: "person_name" },
  { title: "Stage", key: "stage_name" },
  { title: "Source", key: "source_name" },
  { title: "Status", key: "status", width: "100px" },
  { title: "Assigned", key: "user_name" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const deleteDialog = ref(false);
const deletingLead = ref<Lead | null>(null);
const deleting = ref(false);

function confirmDelete(lead: Lead) {
  deletingLead.value = lead;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingLead.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingLead.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
