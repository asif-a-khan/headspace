<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5 font-weight-bold">Leads</h1>
      <v-spacer />
      <v-btn
        variant="outlined"
        class="mr-2"
        href="/admin/leads"
      >
        <v-icon start>mdi-view-column</v-icon>
        Kanban
      </v-btn>
      <v-btn
        variant="outlined"
        class="mr-2"
        prepend-icon="mdi-download"
        href="/admin/api/leads/export"
      >
        Export
      </v-btn>
      <v-btn
        v-if="canCreate"
        variant="outlined"
        class="mr-2"
        prepend-icon="mdi-upload"
        @click="importDialog = true"
      >
        Import
      </v-btn>
      <v-btn
        v-if="canDelete && selectedIds.length"
        color="error"
        variant="outlined"
        class="mr-2"
        prepend-icon="mdi-delete-sweep"
        @click="massDeleteDialog = true"
      >
        Delete ({{ selectedIds.length }})
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
      <v-card-text class="d-flex ga-3 align-center">
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
          @update:model-value="onFilterChange"
        />
        <v-text-field
          v-model="searchQuery"
          density="compact"
          hide-details
          placeholder="Search leads..."
          prepend-inner-icon="mdi-magnify"
          clearable
          variant="outlined"
          style="max-width: 280px"
          @update:model-value="onSearchChange"
        />
      </v-card-text>
    </v-card>

    <v-data-table-server
      v-model="selectedIds"
      :headers="headers"
      :items="store.leads"
      :items-length="store.meta.total"
      :loading="store.loading"
      :page="currentPage"
      :items-per-page="itemsPerPage"
      item-value="id"
      :show-select="canDelete"
      @update:options="onTableUpdate"
    >
      <template #item.title="{ item }">
        <a :href="`/admin/leads/${item.id}`" class="text-decoration-none font-weight-medium">
          {{ item.title }}
        </a>
      </template>
      <template #item.lead_value="{ item }">
        {{ item.lead_value ? `$${Number(item.lead_value).toLocaleString()}` : '-' }}
      </template>
      <template #item.status="{ item }">
        <div class="d-flex align-center ga-1">
          <v-chip v-if="item.status === null" color="info" size="small" variant="tonal">Open</v-chip>
          <v-chip v-else-if="item.status" color="success" size="small" variant="tonal">Won</v-chip>
          <v-chip v-else color="error" size="small" variant="tonal">Lost</v-chip>
          <v-tooltip v-if="item.rotten_days && item.rotten_days > 0" location="top">
            <template #activator="{ props }">
              <v-icon v-bind="props" color="error" size="small">mdi-alert-circle</v-icon>
            </template>
            Rotten for {{ item.rotten_days }} day{{ item.rotten_days === 1 ? '' : 's' }}
          </v-tooltip>
        </div>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          icon="mdi-eye"
          size="small"
          variant="text"
          :href="`/admin/leads/${item.id}`"
        />
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
    </v-data-table-server>

    <v-dialog v-model="importDialog" max-width="500">
      <v-card>
        <v-card-title>Import Leads from CSV</v-card-title>
        <v-card-text>
          <p class="text-body-2 mb-3">
            Upload a CSV file with columns: <strong>title</strong>, description, lead_value, contact_person, source, lead_type, pipeline, stage.
          </p>
          <v-file-input
            v-model="importFile"
            accept=".csv"
            label="Select CSV file"
            prepend-icon="mdi-file-delimited"
            variant="outlined"
            density="compact"
          />
          <v-alert v-if="importResult" :type="importResult.errors?.length ? 'warning' : 'success'" density="compact" class="mt-2">
            {{ importResult.message }}
            <div v-if="importResult.errors?.length" class="mt-1">
              <div v-for="(err, i) in importResult.errors.slice(0, 5)" :key="i" class="text-caption">{{ err }}</div>
              <div v-if="importResult.errors.length > 5" class="text-caption">...and {{ importResult.errors.length - 5 }} more</div>
            </div>
          </v-alert>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="importDialog = false">Close</v-btn>
          <v-btn color="primary" @click="doImport" :loading="importing" :disabled="!importFile?.length">Import</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

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

    <v-dialog v-model="massDeleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Selected Leads</v-card-title>
        <v-card-text>
          Are you sure you want to delete {{ selectedIds.length }} selected lead(s)? This action cannot be undone.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="massDeleteDialog = false">Cancel</v-btn>
          <v-btn color="error" @click="doMassDelete" :loading="massDeleting">Delete All</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useLeadsStore, type Lead } from "@/stores/admin/leads";
import { post } from "@/api/client";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("leads.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("leads.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("leads.delete") || data.permission_type === "all");

const store = useLeadsStore();
store.hydrate(data);

const pipelineItems = computed(() => data.pipelines || []);
const pipelineFilter = ref<number | null>(null);
const searchQuery = ref("");
const currentPage = ref(1);
const itemsPerPage = ref(15);
const sortField = ref<string | undefined>(undefined);
const sortDir = ref<string | undefined>(undefined);

let searchTimer: ReturnType<typeof setTimeout> | null = null;

function loadData() {
  store.fetchAll({
    pipelineId: pipelineFilter.value || undefined,
    page: currentPage.value,
    perPage: itemsPerPage.value,
    search: searchQuery.value || undefined,
    sortField: sortField.value,
    sortDir: sortDir.value,
  });
}

function onFilterChange() {
  currentPage.value = 1;
  loadData();
}

function onSearchChange() {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = setTimeout(() => {
    currentPage.value = 1;
    loadData();
  }, 300);
}

function onTableUpdate(opts: { page: number; itemsPerPage: number; sortBy: Array<{ key: string; order: string }> }) {
  currentPage.value = opts.page;
  itemsPerPage.value = opts.itemsPerPage;
  if (opts.sortBy?.length) {
    sortField.value = opts.sortBy[0].key;
    sortDir.value = opts.sortBy[0].order;
  } else {
    sortField.value = undefined;
    sortDir.value = undefined;
  }
  loadData();
}

const headers = [
  { title: "ID", key: "id", width: "70px" },
  { title: "Title", key: "title" },
  { title: "Value", key: "lead_value", width: "120px" },
  { title: "Contact", key: "person_name", sortable: false },
  { title: "Stage", key: "stage_name", sortable: false },
  { title: "Source", key: "source_name", sortable: false },
  { title: "Status", key: "status", width: "100px", sortable: false },
  { title: "Assigned", key: "user_name", sortable: false },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

const importDialog = ref(false);
const importFile = ref<File[] | null>(null);
const importing = ref(false);
const importResult = ref<{ message: string; imported: number; errors: string[] } | null>(null);

async function doImport() {
  if (!importFile.value?.length) return;
  importing.value = true;
  importResult.value = null;
  try {
    const formData = new FormData();
    formData.append("file", importFile.value[0]);
    const csrfMeta = document.querySelector('meta[name="csrf-token"]');
    const resp = await fetch("/admin/api/leads/import", {
      method: "POST",
      headers: { "X-CSRF-TOKEN": csrfMeta?.getAttribute("content") || "" },
      body: formData,
    });
    importResult.value = await resp.json();
    if (importResult.value && importResult.value.imported > 0) {
      loadData();
    }
  } catch {
    importResult.value = { message: "Upload failed.", imported: 0, errors: [] };
  } finally {
    importing.value = false;
  }
}

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
    loadData();
  } finally {
    deleting.value = false;
  }
}

const selectedIds = ref<number[]>([]);
const massDeleteDialog = ref(false);
const massDeleting = ref(false);

async function doMassDelete() {
  if (!selectedIds.value.length) return;
  massDeleting.value = true;
  try {
    await post("/admin/api/leads/mass-delete", { ids: selectedIds.value });
    massDeleteDialog.value = false;
    selectedIds.value = [];
    loadData();
  } finally {
    massDeleting.value = false;
  }
}
</script>
