<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Product" : "Create Product" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.sku"
            label="SKU"
            :rules="[rules.required]"
            class="mb-4"
          />

          <v-text-field
            v-model="form.name"
            label="Name"
            :rules="[rules.required]"
            class="mb-4"
          />

          <v-textarea
            v-model="form.description"
            label="Description"
            rows="3"
            class="mb-4"
          />

          <div class="d-flex ga-3 mb-3">
            <v-text-field
              v-model="form.price"
              label="Price"
              type="number"
              step="0.01"
              prefix="$"
              :rules="[rules.required]"
            />
            <v-text-field
              v-model="form.quantity"
              label="Quantity"
              type="number"
              step="1"
            />
          </div>
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/products" variant="outlined">Cancel</v-btn>
        <v-spacer />
        <v-btn color="primary" :loading="saving" @click="submit">
          {{ isEdit ? "Update" : "Create" }}
        </v-btn>
      </v-card-actions>
    </v-card>

    <v-snackbar v-model="errorSnackbar" color="error" :timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { useProductsStore } from "@/stores/admin/products";

const data = window.__INITIAL_DATA__ || {};
const store = useProductsStore();
const isEdit = computed(() => !!data.product);

const product = data.product;
const form = reactive({
  sku: product?.sku || "",
  name: product?.name || "",
  description: product?.description || "",
  price: product?.price || "0",
  quantity: product?.quantity || 0,
});

const rules = {
  required: (v: any) => !!v || v === 0 || "Required",
};

const formRef = ref<any>(null);
const saving = ref(false);
const errorSnackbar = ref(false);
const errorMessage = ref("");

async function submit() {
  const { valid } = await formRef.value?.validate();
  if (!valid) return;

  saving.value = true;
  try {
    const payload = {
      sku: form.sku,
      name: form.name,
      description: form.description || null,
      price: Number(form.price),
      quantity: Number(form.quantity),
    };

    if (isEdit.value) {
      await store.update(product.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/products";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
