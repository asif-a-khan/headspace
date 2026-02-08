<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5 font-weight-bold">Products</h1>
      <v-spacer />
      <v-btn
        variant="outlined"
        class="mr-2"
        prepend-icon="mdi-download"
        href="/admin/api/products/export"
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
        href="/admin/products/create"
      >
        Create Product
      </v-btn>
    </div>

    <v-data-table
      v-model="selectedIds"
      :headers="headers"
      :items="store.products"
      :loading="store.loading"
      item-value="id"
      :show-select="canDelete"
    >
      <template #item.price="{ item }">
        ${{ Number(item.price).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) }}
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/products/${item.id}/edit`"
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

    <v-dialog v-model="importDialog" max-width="500">
      <v-card>
        <v-card-title>Import Products from CSV</v-card-title>
        <v-card-text>
          <p class="text-body-2 mb-3">
            Upload a CSV file with columns: <strong>sku</strong>, <strong>name</strong>, description, price, quantity.
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
        <v-card-title>Delete Product</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deletingProduct?.name }}"?
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
        <v-card-title>Delete Selected Products</v-card-title>
        <v-card-text>
          Are you sure you want to delete {{ selectedIds.length }} selected product(s)? This action cannot be undone.
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
import { useProductsStore, type Product } from "@/stores/admin/products";
import { post } from "@/api/client";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("products.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("products.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("products.delete") || data.permission_type === "all");

const store = useProductsStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "70px" },
  { title: "SKU", key: "sku", width: "140px" },
  { title: "Name", key: "name" },
  { title: "Price", key: "price", width: "120px" },
  { title: "Quantity", key: "quantity", width: "100px" },
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
    const resp = await fetch("/admin/api/products/import", {
      method: "POST",
      headers: { "X-CSRF-TOKEN": csrfMeta?.getAttribute("content") || "" },
      body: formData,
    });
    importResult.value = await resp.json();
    if (importResult.value && importResult.value.imported > 0) {
      store.fetchAll();
    }
  } catch {
    importResult.value = { message: "Upload failed.", imported: 0, errors: [] };
  } finally {
    importing.value = false;
  }
}

const deleteDialog = ref(false);
const deletingProduct = ref<Product | null>(null);
const deleting = ref(false);

function confirmDelete(product: Product) {
  deletingProduct.value = product;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingProduct.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingProduct.value.id);
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
    await post("/admin/api/products/mass-delete", { ids: selectedIds.value });
    massDeleteDialog.value = false;
    selectedIds.value = [];
    store.fetchAll();
  } finally {
    massDeleting.value = false;
  }
}
</script>
