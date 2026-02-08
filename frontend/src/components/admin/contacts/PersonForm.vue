<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Person" : "Create Person" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.name"
            label="Name"
            :rules="[rules.required]"
            class="mb-4"
          />

          <v-text-field
            v-model="form.job_title"
            label="Job Title"
            class="mb-4"
          />

          <v-select
            v-model="form.organization_id"
            :items="organizationItems"
            item-title="name"
            item-value="id"
            label="Organization"
            clearable
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

          <!-- Emails -->
          <div class="mb-4">
            <div class="text-subtitle-2 mb-1">Emails</div>
            <div v-for="(email, i) in form.emails" :key="i" class="d-flex align-center mb-2">
              <v-text-field
                v-model="email.value"
                :label="`Email ${i + 1}`"
                type="email"
                density="compact"
                hide-details
                class="mr-2"
              />
              <v-select
                v-model="email.label"
                :items="['work', 'home', 'other']"
                density="compact"
                hide-details
                style="max-width: 140px"
                class="mr-2"
              />
              <v-btn icon="mdi-close" size="small" variant="text" @click="form.emails.splice(i, 1)" />
            </div>
            <v-btn size="small" variant="tonal" prepend-icon="mdi-plus" @click="form.emails.push({ value: '', label: 'work' })">
              Add Email
            </v-btn>
          </div>

          <!-- Contact Numbers -->
          <div class="mb-4">
            <div class="text-subtitle-2 mb-1">Phone Numbers</div>
            <div v-for="(phone, i) in form.contact_numbers" :key="i" class="d-flex align-center mb-2">
              <v-text-field
                v-model="phone.value"
                :label="`Phone ${i + 1}`"
                density="compact"
                hide-details
                class="mr-2"
              />
              <v-select
                v-model="phone.label"
                :items="['work', 'home', 'mobile', 'other']"
                density="compact"
                hide-details
                style="max-width: 140px"
                class="mr-2"
              />
              <v-btn icon="mdi-close" size="small" variant="text" @click="form.contact_numbers.splice(i, 1)" />
            </div>
            <v-btn size="small" variant="tonal" prepend-icon="mdi-plus" @click="form.contact_numbers.push({ value: '', label: 'work' })">
              Add Phone
            </v-btn>
          </div>
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/contacts/persons" variant="outlined">Cancel</v-btn>
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
import { usePersonsStore } from "@/stores/admin/persons";

const data = window.__INITIAL_DATA__ || {};
const store = usePersonsStore();
const isEdit = computed(() => !!data.person);

interface EmailEntry { value: string; label: string }
interface PhoneEntry { value: string; label: string }
interface UserOption { id: number; label: string }

const organizationItems = computed(() => data.organizations || []);
const userItems = computed<UserOption[]>(() =>
  (data.users || []).map((u: any) => ({
    id: u.id,
    label: `${u.first_name} ${u.last_name}`,
  }))
);

const person = data.person;
const form = reactive({
  name: person?.name || "",
  job_title: person?.job_title || "",
  organization_id: person?.organization_id || null,
  user_id: person?.user_id || null,
  emails: (person?.emails || []).map((e: any) => ({ ...e })) as EmailEntry[],
  contact_numbers: (person?.contact_numbers || []).map((p: any) => ({ ...p })) as PhoneEntry[],
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
    const payload = {
      name: form.name,
      job_title: form.job_title || null,
      organization_id: form.organization_id || null,
      user_id: form.user_id || null,
      emails: form.emails.filter((e) => e.value),
      contact_numbers: form.contact_numbers.filter((p) => p.value),
    };

    if (isEdit.value) {
      await store.update(person.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/contacts/persons";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
