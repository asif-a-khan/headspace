<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Quotes</h1>
      <v-spacer />
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
        href="/admin/quotes/create"
      >
        Create Quote
      </v-btn>
    </div>

    <v-data-table
      v-model="selectedIds"
      :headers="headers"
      :items="store.quotes"
      :loading="store.loading"
      item-value="id"
      :show-select="canDelete"
    >
      <template #item.subject="{ item }">
        <a :href="`/admin/quotes/${item.id}/edit`" class="text-decoration-none font-weight-medium">
          {{ item.subject }}
        </a>
      </template>
      <template #item.grand_total="{ item }">
        {{ item.grand_total ? `$${Number(item.grand_total).toLocaleString(undefined, { minimumFractionDigits: 2 })}` : '-' }}
      </template>
      <template #item.expired_at="{ item }">
        {{ item.expired_at ? new Date(item.expired_at).toLocaleDateString() : '-' }}
      </template>
      <template #item.actions="{ item }">
        <v-btn
          icon="mdi-file-pdf-box"
          size="small"
          variant="text"
          color="error"
          :href="`/admin/api/quotes/${item.id}/pdf`"
          target="_blank"
          title="Download PDF"
        />
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/quotes/${item.id}/edit`"
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
        <v-card-title>Delete Quote</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingQuote?.subject }}"?
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
        <v-card-title>Delete Selected Quotes</v-card-title>
        <v-card-text>
          Are you sure you want to delete {{ selectedIds.length }} selected quote(s)? This action cannot be undone.
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
import { useQuotesStore, type Quote } from "@/stores/admin/quotes";
import { post } from "@/api/client";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("quotes.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("quotes.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("quotes.delete") || data.permission_type === "all");

const store = useQuotesStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "70px" },
  { title: "Subject", key: "subject" },
  { title: "Contact", key: "person_name" },
  { title: "Total", key: "grand_total", width: "140px" },
  { title: "Expires", key: "expired_at", width: "120px" },
  { title: "Assigned", key: "user_name" },
  { title: "Actions", key: "actions", sortable: false, width: "160px" },
];

const deleteDialog = ref(false);
const deletingQuote = ref<Quote | null>(null);
const deleting = ref(false);

function confirmDelete(quote: Quote) {
  deletingQuote.value = quote;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingQuote.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingQuote.value.id);
    deleteDialog.value = false;
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
    await post("/admin/api/quotes/mass-delete", { ids: selectedIds.value });
    massDeleteDialog.value = false;
    selectedIds.value = [];
    store.fetchAll();
  } finally {
    massDeleting.value = false;
  }
}
</script>
