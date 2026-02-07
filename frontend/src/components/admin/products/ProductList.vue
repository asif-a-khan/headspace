<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Products</h1>
      <v-spacer />
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
      :headers="headers"
      :items="store.products"
      :loading="store.loading"
      item-value="id"
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useProductsStore, type Product } from "@/stores/admin/products";

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
</script>
