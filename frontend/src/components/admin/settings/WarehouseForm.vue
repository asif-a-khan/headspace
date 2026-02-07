<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Warehouse" : "Create Warehouse" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.name"
            label="Warehouse Name"
            :rules="[rules.required]"
            class="mb-3"
          />
          <v-textarea
            v-model="form.description"
            label="Description"
            rows="3"
            class="mb-3"
          />
          <v-text-field
            v-model="form.contact_name"
            label="Contact Name"
            class="mb-3"
          />

          <div class="text-subtitle-2 mb-2">Contact Emails</div>
          <div v-for="(email, i) in form.contact_emails" :key="i" class="d-flex ga-2 mb-2">
            <v-text-field v-model="email.value" label="Email" density="compact" hide-details />
            <v-text-field v-model="email.label" label="Label" density="compact" hide-details style="max-width: 120px" />
            <v-btn icon="mdi-close" size="small" variant="text" @click="form.contact_emails.splice(i, 1)" />
          </div>
          <v-btn size="small" variant="tonal" class="mb-4" @click="form.contact_emails.push({ value: '', label: '' })">
            Add Email
          </v-btn>

          <div class="text-subtitle-2 mb-2">Contact Numbers</div>
          <div v-for="(phone, i) in form.contact_numbers" :key="i" class="d-flex ga-2 mb-2">
            <v-text-field v-model="phone.value" label="Phone" density="compact" hide-details />
            <v-text-field v-model="phone.label" label="Label" density="compact" hide-details style="max-width: 120px" />
            <v-btn icon="mdi-close" size="small" variant="text" @click="form.contact_numbers.splice(i, 1)" />
          </div>
          <v-btn size="small" variant="tonal" class="mb-4" @click="form.contact_numbers.push({ value: '', label: '' })">
            Add Phone
          </v-btn>

          <div class="text-subtitle-2 mb-2">Address</div>
          <v-text-field v-model="form.address.address" label="Street Address" density="compact" class="mb-2" />
          <div class="d-flex ga-3 mb-3">
            <v-text-field v-model="form.address.city" label="City" density="compact" />
            <v-text-field v-model="form.address.state" label="State" density="compact" />
            <v-text-field v-model="form.address.postcode" label="Postal Code" density="compact" />
            <v-text-field v-model="form.address.country" label="Country" density="compact" />
          </div>
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/settings/warehouses" variant="text">Cancel</v-btn>
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
import { useWarehousesStore } from "@/stores/admin/warehouses";

const data = window.__INITIAL_DATA__ || {};
const store = useWarehousesStore();
const isEdit = computed(() => !!data.warehouse);
const warehouse = data.warehouse;

const form = reactive({
  name: warehouse?.name || "",
  description: warehouse?.description || "",
  contact_name: warehouse?.contact_name || "",
  contact_emails: (warehouse?.contact_emails || []).map((e: any) => ({ ...e })),
  contact_numbers: (warehouse?.contact_numbers || []).map((n: any) => ({ ...n })),
  address: {
    address: warehouse?.contact_address?.address || "",
    city: warehouse?.contact_address?.city || "",
    state: warehouse?.contact_address?.state || "",
    postcode: warehouse?.contact_address?.postcode || "",
    country: warehouse?.contact_address?.country || "",
  },
});

const rules = {
  required: (v: any) => !!v || "Required",
};

const formRef = ref<any>(null);
const saving = ref(false);
const errorSnackbar = ref(false);
const errorMessage = ref("");

function isAddressEmpty(): boolean {
  return !form.address.address && !form.address.city && !form.address.state && !form.address.postcode && !form.address.country;
}

async function submit() {
  const { valid } = await formRef.value?.validate();
  if (!valid) return;

  saving.value = true;
  try {
    const payload = {
      name: form.name,
      description: form.description || null,
      contact_name: form.contact_name || null,
      contact_emails: form.contact_emails.filter((e: any) => e.value),
      contact_numbers: form.contact_numbers.filter((n: any) => n.value),
      contact_address: isAddressEmpty() ? null : { ...form.address },
    };

    if (isEdit.value) {
      await store.update(warehouse.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/settings/warehouses";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
