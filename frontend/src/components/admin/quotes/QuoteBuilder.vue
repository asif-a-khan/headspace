<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Quote" : "Create Quote" }}</h1>

    <v-card class="mb-4">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.subject"
            label="Subject"
            :rules="[rules.required]"
            class="mb-3"
          />

          <v-textarea
            v-model="form.description"
            label="Description"
            rows="2"
            class="mb-3"
          />

          <div class="d-flex ga-3 mb-3">
            <v-select
              v-model="form.person_id"
              :items="personItems"
              item-title="name"
              item-value="id"
              label="Contact Person"
              clearable
            />
            <v-select
              v-model="form.user_id"
              :items="userItems"
              item-title="label"
              item-value="id"
              label="Assigned To"
              clearable
            />
            <v-text-field
              v-model="form.expired_at"
              label="Expiry Date"
              type="date"
            />
          </div>
        </v-form>
      </v-card-text>
    </v-card>

    <!-- Line Items -->
    <v-card class="mb-4">
      <v-card-title class="d-flex align-center">
        Line Items
        <v-spacer />
        <v-btn size="small" variant="tonal" prepend-icon="mdi-plus" @click="addItem">
          Add Item
        </v-btn>
      </v-card-title>
      <v-card-text>
        <v-table density="compact">
          <thead>
            <tr>
              <th>Product</th>
              <th style="width: 80px">Qty</th>
              <th style="width: 120px">Price</th>
              <th style="width: 120px">Total</th>
              <th style="width: 50px"></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, i) in activeItems" :key="i">
              <td>
                <v-autocomplete
                  v-model="item.product_id"
                  :items="productItems"
                  item-title="label"
                  item-value="id"
                  density="compact"
                  hide-details
                  clearable
                  @update:model-value="onProductSelect(item)"
                />
              </td>
              <td>
                <v-text-field
                  v-model.number="item.quantity"
                  type="number"
                  density="compact"
                  hide-details
                  min="1"
                  @update:model-value="calcItemTotal(item)"
                />
              </td>
              <td>
                <v-text-field
                  v-model.number="item.price"
                  type="number"
                  step="0.01"
                  density="compact"
                  hide-details
                  prefix="$"
                  @update:model-value="calcItemTotal(item)"
                />
              </td>
              <td class="font-weight-medium">
                ${{ item.total.toFixed(2) }}
              </td>
              <td>
                <v-btn icon="mdi-close" size="x-small" variant="text" @click="removeItem(i)" />
              </td>
            </tr>
            <tr v-if="!activeItems.length">
              <td colspan="5" class="text-center text-medium-emphasis pa-4">
                No items yet. Click "Add Item" to start.
              </td>
            </tr>
          </tbody>
        </v-table>

        <div class="d-flex justify-end mt-4">
          <div style="width: 300px">
            <div class="d-flex justify-space-between mb-1">
              <span>Sub Total:</span>
              <strong>${{ subTotal.toFixed(2) }}</strong>
            </div>
            <div class="d-flex justify-space-between mb-1 align-center">
              <span>Adjustment:</span>
              <v-text-field
                v-model.number="form.adjustment_amount"
                type="number"
                step="0.01"
                density="compact"
                hide-details
                prefix="$"
                style="max-width: 130px"
              />
            </div>
            <v-divider class="my-2" />
            <div class="d-flex justify-space-between text-h6">
              <span>Grand Total:</span>
              <strong>${{ grandTotal.toFixed(2) }}</strong>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>

    <div class="d-flex">
      <v-btn href="/admin/quotes" variant="text">Cancel</v-btn>
      <v-spacer />
      <v-btn color="primary" :loading="saving" @click="submit">
        {{ isEdit ? "Update" : "Create" }}
      </v-btn>
    </div>

    <v-snackbar v-model="errorSnackbar" color="error" :timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { useQuotesStore, type QuoteItem } from "@/stores/admin/quotes";

const data = window.__INITIAL_DATA__ || {};
const store = useQuotesStore();
const isEdit = computed(() => !!data.quote);

const breadcrumbs = computed(() => [
  { title: "Quotes", href: "/admin/quotes" },
  { title: isEdit.value ? "Edit" : "Create", disabled: true },
]);

const personItems = computed(() => data.persons || []);
const productItems = computed(() =>
  (data.products || []).map((p: any) => ({
    id: p.id,
    label: `${p.sku} - ${p.name}`,
    price: Number(p.price),
  }))
);
const userItems = computed(() =>
  (data.users || []).map((u: any) => ({
    id: u.id,
    label: `${u.first_name} ${u.last_name}`,
  }))
);

const quote = data.quote;
const form = reactive({
  subject: quote?.subject || "",
  description: quote?.description || "",
  person_id: quote?.person_id || null,
  user_id: quote?.user_id || null,
  expired_at: quote?.expired_at ? quote.expired_at.split("T")[0] : "",
  adjustment_amount: Number(quote?.adjustment_amount || 0),
});

const items = ref<QuoteItem[]>(
  (data.items || []).map((item: any) => ({
    id: item.id,
    sku: item.sku,
    name: item.name,
    quantity: item.quantity,
    price: Number(item.price),
    discount_percent: Number(item.discount_percent || 0),
    discount_amount: Number(item.discount_amount || 0),
    tax_percent: Number(item.tax_percent || 0),
    tax_amount: Number(item.tax_amount || 0),
    total: Number(item.total),
    product_id: item.product_id,
  }))
);

const activeItems = computed(() => items.value.filter((i) => !i.is_delete));

function addItem() {
  items.value.push({
    sku: null,
    name: null,
    quantity: 1,
    price: 0,
    discount_percent: 0,
    discount_amount: 0,
    tax_percent: 0,
    tax_amount: 0,
    total: 0,
    product_id: null,
  });
}

function removeItem(index: number) {
  const active = activeItems.value;
  const item = active[index];
  if (item.id) {
    item.is_delete = true;
  } else {
    const realIndex = items.value.indexOf(item);
    if (realIndex >= 0) items.value.splice(realIndex, 1);
  }
}

function onProductSelect(item: QuoteItem) {
  const product = productItems.value.find((p: any) => p.id === item.product_id);
  if (product) {
    item.price = product.price;
    item.name = product.label;
    calcItemTotal(item);
  }
}

function calcItemTotal(item: QuoteItem) {
  item.total = (item.quantity || 0) * (item.price || 0);
}

const subTotal = computed(() => activeItems.value.reduce((sum, i) => sum + i.total, 0));
const grandTotal = computed(() => subTotal.value + (form.adjustment_amount || 0));

const rules = {
  required: (v: any) => !!v || "Required",
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
      subject: form.subject,
      description: form.description || null,
      person_id: form.person_id || null,
      user_id: form.user_id || null,
      expired_at: form.expired_at ? new Date(form.expired_at).toISOString() : null,
      sub_total: subTotal.value,
      grand_total: grandTotal.value,
      adjustment_amount: form.adjustment_amount || 0,
      discount_percent: 0,
      discount_amount: 0,
      tax_amount: 0,
      items: items.value,
    };

    if (isEdit.value) {
      await store.update(quote.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/quotes";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
