<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Activity" : "Create Activity" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.title"
            label="Title"
            class="mb-3"
          />

          <v-select
            v-model="form.type"
            :items="activityTypes"
            label="Type"
            :rules="[rules.required]"
            class="mb-3"
          />

          <v-textarea
            v-model="form.comment"
            label="Comment / Notes"
            rows="3"
            class="mb-3"
          />

          <v-text-field
            v-model="form.location"
            label="Location"
            class="mb-3"
          />

          <div class="d-flex ga-3 mb-3">
            <v-text-field
              v-model="form.schedule_from"
              label="Schedule From"
              type="datetime-local"
            />
            <v-text-field
              v-model="form.schedule_to"
              label="Schedule To"
              type="datetime-local"
            />
          </div>

          <v-checkbox
            v-model="form.is_done"
            label="Mark as done"
            class="mb-3"
          />
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/activities" variant="text">Cancel</v-btn>
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
import { useActivitiesStore } from "@/stores/admin/activities";

const data = window.__INITIAL_DATA__ || {};
const store = useActivitiesStore();
const isEdit = computed(() => !!data.activity);

const breadcrumbs = computed(() => [
  { title: "Activities", href: "/admin/activities" },
  { title: isEdit.value ? "Edit" : "Create", disabled: true },
]);

const activityTypes = ["call", "meeting", "note", "task"];

function toLocalDatetime(isoString: string | null): string {
  if (!isoString) return "";
  const d = new Date(isoString);
  const offset = d.getTimezoneOffset();
  const local = new Date(d.getTime() - offset * 60000);
  return local.toISOString().slice(0, 16);
}

const activity = data.activity;
const form = reactive({
  title: activity?.title || "",
  type: activity?.type || "note",
  comment: activity?.comment || "",
  location: activity?.location || "",
  schedule_from: toLocalDatetime(activity?.schedule_from),
  schedule_to: toLocalDatetime(activity?.schedule_to),
  is_done: activity?.is_done || false,
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
    const payload: Record<string, unknown> = {
      title: form.title || null,
      type: form.type,
      comment: form.comment || null,
      location: form.location || null,
      schedule_from: form.schedule_from ? new Date(form.schedule_from).toISOString() : null,
      schedule_to: form.schedule_to ? new Date(form.schedule_to).toISOString() : null,
      is_done: form.is_done,
    };

    if (isEdit.value) {
      await store.update(activity.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/activities";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
