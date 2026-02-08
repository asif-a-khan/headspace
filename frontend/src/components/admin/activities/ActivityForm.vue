<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Activity" : "Create Activity" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.title"
            label="Title"
            class="mb-4"
          />

          <v-select
            v-model="form.type"
            :items="activityTypes"
            label="Type"
            :rules="[rules.required]"
            class="mb-4"
          />

          <v-textarea
            v-model="form.comment"
            label="Comment / Notes"
            rows="3"
            class="mb-4"
          />

          <v-text-field
            v-model="form.location"
            label="Location"
            class="mb-4"
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

          <!-- Participant Users -->
          <v-select
            v-model="form.participant_user_ids"
            :items="userOptions"
            item-title="label"
            item-value="value"
            label="Participant Users"
            multiple
            chips
            closable-chips
            class="mb-4"
          />

          <!-- Participant Persons (search) -->
          <v-combobox
            v-model="selectedPersons"
            :items="personSearchResults"
            item-title="label"
            item-value="value"
            label="Participant Persons"
            multiple
            chips
            closable-chips
            class="mb-4"
            return-object
            :loading="personSearching"
            @update:search="onPersonSearch"
            no-filter
            hide-no-data
          />

          <!-- Associated Leads -->
          <v-combobox
            v-model="selectedLeads"
            :items="leadSearchResults"
            item-title="label"
            item-value="value"
            label="Associated Leads"
            multiple
            chips
            closable-chips
            class="mb-4"
            return-object
            :loading="leadSearching"
            @update:search="onLeadSearch"
            no-filter
            hide-no-data
          />

          <v-checkbox
            v-model="form.is_done"
            label="Mark as done"
            class="mb-4"
          />

          <!-- File Attachments (edit mode) -->
          <template v-if="isEdit">
            <div class="text-subtitle-2 mb-2">Attachments</div>
            <div v-if="files.length" class="mb-4">
              <v-chip
                v-for="f in files"
                :key="f.id"
                class="mr-2 mb-1"
                closable
                @click:close="removeFile(f.id)"
                @click="downloadFile(f)"
              >
                <v-icon start size="small">mdi-file</v-icon>
                {{ f.file_name }}
                <span class="text-caption ml-1 text-medium-emphasis">({{ formatSize(f.file_size) }})</span>
              </v-chip>
            </div>
            <div v-else class="text-caption text-medium-emphasis mb-3">No files attached.</div>
            <v-file-input
              v-model="newFiles"
              label="Upload Files"
              multiple
              prepend-icon="mdi-paperclip"
              show-size
              class="mb-4"
              :loading="uploading"
            />
            <v-btn
              v-if="newFiles && newFiles.length"
              variant="tonal"
              size="small"
              color="primary"
              :loading="uploading"
              class="mb-4"
              @click="uploadFiles"
            >
              Upload {{ newFiles.length }} file{{ newFiles.length === 1 ? '' : 's' }}
            </v-btn>
          </template>
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/activities" variant="outlined">Cancel</v-btn>
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
import { get } from "@/api/client";

const data = window.__INITIAL_DATA__ || {};
const store = useActivitiesStore();
const isEdit = computed(() => !!data.activity);

const activityTypes = ["call", "meeting", "note", "task"];

// User options for participant select
const userOptions = computed(() =>
  (data.users || []).map((u: any) => ({
    label: `${u.first_name} ${u.last_name}`,
    value: u.id,
  })),
);

// Existing participants
const existingParticipants = data.participants || [];
const existingUserIds = existingParticipants
  .filter((p: any) => p.user_id != null)
  .map((p: any) => p.user_id);
const existingPersons = existingParticipants
  .filter((p: any) => p.person_id != null)
  .map((p: any) => ({ label: p.person_name || `Person #${p.person_id}`, value: p.person_id }));

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
  participant_user_ids: existingUserIds as number[],
});

const selectedPersons = ref<Array<{ label: string; value: number }>>(existingPersons);

// Existing linked leads
const existingLinkedLeads = (data.linked_leads || []).map((l: any) => ({
  label: `#${l.id} - ${l.title}`,
  value: l.id,
}));
const selectedLeads = ref<Array<{ label: string; value: number }>>(existingLinkedLeads);

// Lead search
const leadSearchResults = ref<Array<{ label: string; value: number }>>([]);
const leadSearching = ref(false);
let leadSearchTimer: ReturnType<typeof setTimeout> | null = null;

function onLeadSearch(val: string) {
  if (leadSearchTimer) clearTimeout(leadSearchTimer);
  if (!val || val.length < 2) {
    leadSearchResults.value = [];
    return;
  }
  leadSearchTimer = setTimeout(async () => {
    leadSearching.value = true;
    try {
      const res = await get<{ data: Array<{ id: number; title: string }> }>(
        `/admin/api/leads/search?q=${encodeURIComponent(val)}`,
      );
      leadSearchResults.value = res.data.map((l) => ({ label: `#${l.id} - ${l.title}`, value: l.id }));
    } catch {
      leadSearchResults.value = [];
    } finally {
      leadSearching.value = false;
    }
  }, 300);
}

// Person search
const personSearchResults = ref<Array<{ label: string; value: number }>>([]);
const personSearching = ref(false);
let personSearchTimer: ReturnType<typeof setTimeout> | null = null;

function onPersonSearch(val: string) {
  if (personSearchTimer) clearTimeout(personSearchTimer);
  if (!val || val.length < 2) {
    personSearchResults.value = [];
    return;
  }
  personSearchTimer = setTimeout(async () => {
    personSearching.value = true;
    try {
      const res = await get<{ data: Array<{ id: number; name: string }> }>(
        `/admin/api/contacts/persons/search?q=${encodeURIComponent(val)}`,
      );
      personSearchResults.value = res.data.map((p) => ({ label: p.name, value: p.id }));
    } catch {
      personSearchResults.value = [];
    } finally {
      personSearching.value = false;
    }
  }, 300);
}

// File attachments
interface ActivityFileInfo {
  id: number;
  file_name: string;
  file_size: number;
}
const files = ref<ActivityFileInfo[]>(data.files || []);
const newFiles = ref<File[]>([]);
const uploading = ref(false);

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}

function downloadFile(f: ActivityFileInfo) {
  window.open(`/admin/api/activities/${activity.id}/files/${f.id}`, "_blank");
}

async function removeFile(fileId: number) {
  const meta = document.querySelector('meta[name="csrf-token"]');
  const csrfToken = meta?.getAttribute("content") ?? "";
  try {
    await fetch(`/admin/api/activities/${activity.id}/files/${fileId}`, {
      method: "DELETE",
      headers: { "X-CSRF-Token": csrfToken },
      credentials: "same-origin",
    });
    files.value = files.value.filter((f) => f.id !== fileId);
  } catch {
    errorMessage.value = "Failed to delete file.";
    errorSnackbar.value = true;
  }
}

async function uploadFiles() {
  if (!newFiles.value?.length) return;
  uploading.value = true;
  const meta = document.querySelector('meta[name="csrf-token"]');
  const csrfToken = meta?.getAttribute("content") ?? "";
  const formData = new FormData();
  for (const f of newFiles.value) {
    formData.append("file", f);
  }
  try {
    const resp = await fetch(`/admin/api/activities/${activity.id}/files`, {
      method: "POST",
      headers: { "X-CSRF-Token": csrfToken },
      credentials: "same-origin",
      body: formData,
    });
    const json = await resp.json();
    if (json.data) {
      files.value.push(...json.data);
    }
    newFiles.value = [];
  } catch {
    errorMessage.value = "Failed to upload files.";
    errorSnackbar.value = true;
  } finally {
    uploading.value = false;
  }
}

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
      participant_user_ids: form.participant_user_ids,
      participant_person_ids: selectedPersons.value.map((p) => p.value),
      lead_ids: selectedLeads.value.map((l) => l.value),
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
