<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Organization" : "Create Organization" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.name"
            label="Name"
            :rules="[rules.required]"
            class="mb-4"
          />

          <v-select
            v-model="form.user_id"
            :items="userItems"
            item-title="label"
            item-value="id"
            label="Assigned To"
            clearable
            class="mb-4"
          />

          <!-- Address fields -->
          <div class="text-subtitle-2 mb-1">Address</div>
          <v-text-field
            v-model="form.address.street"
            label="Street"
            density="compact"
            class="mb-2"
          />
          <div class="d-flex gap-2 mb-2">
            <v-text-field
              v-model="form.address.city"
              label="City"
              density="compact"
            />
            <v-text-field
              v-model="form.address.state"
              label="State"
              density="compact"
            />
          </div>
          <div class="d-flex gap-2 mb-3">
            <v-text-field
              v-model="form.address.postcode"
              label="Postcode"
              density="compact"
            />
            <v-text-field
              v-model="form.address.country"
              label="Country"
              density="compact"
            />
          </div>
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/contacts/organizations" variant="outlined">Cancel</v-btn>
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
import { useOrganizationsStore } from "@/stores/admin/organizations";

const data = window.__INITIAL_DATA__ || {};
const store = useOrganizationsStore();
const isEdit = computed(() => !!data.organization);

interface UserOption { id: number; label: string }

const userItems = computed<UserOption[]>(() =>
  (data.users || []).map((u: any) => ({
    id: u.id,
    label: `${u.first_name} ${u.last_name}`,
  }))
);

const org = data.organization;
const addr = org?.address || {};
const form = reactive({
  name: org?.name || "",
  user_id: org?.user_id || null,
  address: {
    street: addr.street || "",
    city: addr.city || "",
    state: addr.state || "",
    postcode: addr.postcode || "",
    country: addr.country || "",
  },
});

const rules = {
  required: (v: string) => !!v || "Required",
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
    const hasAddress = form.address.street || form.address.city || form.address.state || form.address.postcode || form.address.country;
    const payload = {
      name: form.name,
      user_id: form.user_id || null,
      address: hasAddress ? form.address : null,
    };

    if (isEdit.value) {
      await store.update(org.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/contacts/organizations";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
