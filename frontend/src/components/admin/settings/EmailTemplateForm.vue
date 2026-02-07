<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Email Template" : "Create Email Template" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.name"
            label="Template Name"
            :rules="[rules.required]"
            class="mb-3"
          />
          <v-text-field
            v-model="form.subject"
            label="Subject"
            :rules="[rules.required]"
            class="mb-3"
          />
          <v-textarea
            v-model="form.content"
            label="Content"
            rows="10"
            class="mb-3"
          />
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/settings/email-templates" variant="text">Cancel</v-btn>
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
import { useEmailTemplatesStore } from "@/stores/admin/email_templates";

const data = window.__INITIAL_DATA__ || {};
const store = useEmailTemplatesStore();
const isEdit = computed(() => !!data.email_template);
const template = data.email_template;

const form = reactive({
  name: template?.name || "",
  subject: template?.subject || "",
  content: template?.content || "",
});

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
      name: form.name,
      subject: form.subject,
      content: form.content || "",
    };

    if (isEdit.value) {
      await store.update(template.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/settings/email-templates";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
