<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Web Forms</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        href="/admin/settings/web-forms/create"
        prepend-icon="mdi-plus"
      >
        Create Web Form
      </v-btn>
    </div>

    <v-card>
      <v-data-table
        :headers="headers"
        :items="store.items"
        item-value="id"
        :items-per-page="25"
      >
        <template #item.form_id="{ item }">
          <code>{{ item.form_id }}</code>
        </template>
        <template #item.create_lead="{ item }">
          <v-icon :color="item.create_lead ? 'success' : 'grey'">
            {{ item.create_lead ? 'mdi-check-circle' : 'mdi-close-circle' }}
          </v-icon>
        </template>
        <template #item.actions="{ item }">
          <v-btn
            size="small"
            variant="text"
            icon="mdi-eye"
            :href="`/web-forms/${item.form_id}`"
            target="_blank"
            title="Preview"
          />
          <v-btn
            size="small"
            variant="text"
            icon="mdi-code-tags"
            title="Embed Code"
            @click="showEmbed(item)"
          />
          <v-btn
            v-if="canEdit"
            size="small"
            variant="text"
            icon="mdi-pencil"
            :href="`/admin/settings/web-forms/${item.id}/edit`"
          />
          <v-btn
            v-if="canDelete"
            size="small"
            variant="text"
            icon="mdi-delete"
            color="error"
            @click="confirmDelete(item)"
          />
        </template>
      </v-data-table>
    </v-card>

    <!-- Delete dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Web Form</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ deleteTarget?.title }}"?
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" @click="doDelete" :loading="deleting">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Embed code dialog -->
    <v-dialog v-model="embedDialog" max-width="500">
      <v-card>
        <v-card-title>Embed Code</v-card-title>
        <v-card-text>
          <p class="text-body-2 mb-3">Copy this code to embed the form on your website:</p>
          <v-textarea
            :model-value="embedCode"
            readonly
            rows="4"
            variant="outlined"
            density="compact"
            @click="selectAll"
            ref="embedRef"
          />
          <v-btn size="small" variant="tonal" @click="copyEmbed" prepend-icon="mdi-content-copy">
            Copy to Clipboard
          </v-btn>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="embedDialog = false">Close</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="snackbar" :color="snackbarColor" :timeout="3000">
      {{ snackbarText }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useWebFormsStore, type WebForm } from "@/stores/admin/web_forms";

const data = window.__INITIAL_DATA__ || {};
const store = useWebFormsStore();
store.hydrate(data.web_forms || []);

const permissionType = data.permission_type || "";
const permissions = data.permissions || [];
const canCreate = computed(() => permissionType === "all" || permissions.includes("settings.web_forms.create"));
const canEdit = computed(() => permissionType === "all" || permissions.includes("settings.web_forms.edit"));
const canDelete = computed(() => permissionType === "all" || permissions.includes("settings.web_forms.delete"));

const headers = [
  { title: "Title", key: "title" },
  { title: "Form ID", key: "form_id" },
  { title: "Creates Lead", key: "create_lead", align: "center" as const },
  { title: "Actions", key: "actions", sortable: false, align: "end" as const },
];

// Delete
const deleteDialog = ref(false);
const deleteTarget = ref<WebForm | null>(null);
const deleting = ref(false);

function confirmDelete(item: WebForm) {
  deleteTarget.value = item;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deleteTarget.value) return;
  deleting.value = true;
  try {
    await store.remove(deleteTarget.value.id);
    snackbarText.value = "Web form deleted.";
    snackbarColor.value = "success";
    snackbar.value = true;
  } catch {
    snackbarText.value = "Failed to delete web form.";
    snackbarColor.value = "error";
    snackbar.value = true;
  } finally {
    deleting.value = false;
    deleteDialog.value = false;
  }
}

// Embed
const embedDialog = ref(false);
const embedCode = ref("");
const embedRef = ref<any>(null);

function showEmbed(item: WebForm) {
  const origin = window.location.origin;
  embedCode.value = `<iframe src="${origin}/web-forms/${item.form_id}" width="100%" height="600" frameborder="0" style="border: none;"></iframe>`;
  embedDialog.value = true;
}

function selectAll(e: Event) {
  const target = e.target as HTMLTextAreaElement;
  target.select();
}

function copyEmbed() {
  navigator.clipboard.writeText(embedCode.value);
  snackbarText.value = "Copied to clipboard.";
  snackbarColor.value = "success";
  snackbar.value = true;
}

const snackbar = ref(false);
const snackbarText = ref("");
const snackbarColor = ref("success");
</script>
