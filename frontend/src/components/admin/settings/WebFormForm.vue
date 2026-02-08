<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Web Form" : "Create Web Form" }}</h1>

    <v-row>
      <v-col cols="12" md="7">
        <v-card>
          <v-card-title>Form Settings</v-card-title>
          <v-card-text>
            <v-text-field
              v-model="form.title"
              label="Title"
              :rules="[rules.required]"
              variant="outlined"
              density="compact"
              class="mb-4"
            />
            <v-textarea
              v-model="form.description"
              label="Description"
              rows="2"
              variant="outlined"
              density="compact"
              class="mb-4"
            />
            <v-text-field
              v-model="form.submit_button_label"
              label="Submit Button Label"
              variant="outlined"
              density="compact"
              class="mb-4"
            />
            <v-select
              v-model="form.submit_success_action"
              :items="successActions"
              item-title="label"
              item-value="value"
              label="After Submit"
              variant="outlined"
              density="compact"
              class="mb-4"
            />
            <v-text-field
              v-model="form.submit_success_content"
              :label="form.submit_success_action === 'redirect' ? 'Redirect URL' : 'Success Message'"
              variant="outlined"
              density="compact"
              class="mb-4"
            />
            <v-checkbox
              v-model="form.create_lead"
              label="Create lead from submission"
              hide-details
              class="mb-4"
            />
          </v-card-text>
        </v-card>

        <!-- Attributes Section -->
        <v-card class="mt-4">
          <v-card-title>
            Form Fields
            <v-btn
              size="small"
              variant="tonal"
              color="primary"
              class="ml-3"
              prepend-icon="mdi-plus"
              @click="addAttribute"
            >
              Add Field
            </v-btn>
          </v-card-title>
          <v-card-text>
            <div v-if="!formAttrs.length" class="text-caption text-medium-emphasis">
              No fields added yet. Click "Add Field" to include attributes.
            </div>
            <v-table v-else density="compact">
              <thead>
                <tr>
                  <th>#</th>
                  <th>Attribute</th>
                  <th>Label Override</th>
                  <th>Placeholder</th>
                  <th>Required</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(attr, idx) in formAttrs" :key="idx">
                  <td>{{ idx + 1 }}</td>
                  <td>
                    <v-select
                      v-model="attr.attribute_id"
                      :items="availableAttributes"
                      item-title="display"
                      item-value="id"
                      variant="outlined"
                      density="compact"
                      hide-details
                      style="min-width: 180px"
                    />
                  </td>
                  <td>
                    <v-text-field
                      v-model="attr.name"
                      variant="outlined"
                      density="compact"
                      hide-details
                      placeholder="(use attribute name)"
                      style="min-width: 140px"
                    />
                  </td>
                  <td>
                    <v-text-field
                      v-model="attr.placeholder"
                      variant="outlined"
                      density="compact"
                      hide-details
                      style="min-width: 140px"
                    />
                  </td>
                  <td>
                    <v-checkbox
                      v-model="attr.is_required"
                      hide-details
                      density="compact"
                    />
                  </td>
                  <td>
                    <v-btn
                      size="small"
                      variant="text"
                      icon="mdi-delete"
                      color="error"
                      @click="formAttrs.splice(idx, 1)"
                    />
                  </td>
                </tr>
              </tbody>
            </v-table>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="5">
        <!-- Styling -->
        <v-card>
          <v-card-title>Appearance</v-card-title>
          <v-card-text>
            <div v-for="colorField in colorFields" :key="colorField.key" class="mb-4">
              <label class="text-body-2 d-block mb-1">{{ colorField.label }}</label>
              <div class="d-flex align-center ga-3">
                <v-text-field
                  v-model="form[colorField.key]"
                  variant="outlined"
                  density="compact"
                  hide-details
                  style="max-width: 140px"
                />
                <div
                  :style="{
                    width: '28px',
                    height: '28px',
                    borderRadius: '4px',
                    backgroundColor: form[colorField.key] || '#CCC',
                    border: '1px solid rgba(0,0,0,0.2)',
                  }"
                />
              </div>
            </div>
          </v-card-text>
        </v-card>

        <!-- Preview -->
        <v-card class="mt-4" v-if="isEdit && webForm">
          <v-card-title>Embed</v-card-title>
          <v-card-text>
            <p class="text-body-2 mb-2">
              Form ID: <code>{{ webForm.form_id }}</code>
            </p>
            <v-btn
              size="small"
              variant="tonal"
              :href="`/web-forms/${webForm.form_id}`"
              target="_blank"
              prepend-icon="mdi-eye"
            >
              Preview Form
            </v-btn>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <div class="d-flex mt-4">
      <v-btn href="/admin/settings/web-forms" variant="outlined">Cancel</v-btn>
      <v-spacer />
      <v-btn color="primary" @click="save" :loading="saving" prepend-icon="mdi-content-save">
        {{ isEdit ? "Update" : "Create" }}
      </v-btn>
    </div>

    <v-snackbar v-model="snackbar" :color="snackbarColor" :timeout="3000">
      {{ snackbarText }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { useWebFormsStore } from "@/stores/admin/web_forms";

const data = window.__INITIAL_DATA__ || {};
const store = useWebFormsStore();
const webForm = data.web_form || null;
const isEdit = computed(() => !!webForm);

interface FormAttr {
  attribute_id: number | null;
  name: string;
  placeholder: string;
  is_required: boolean;
  sort_order: number;
}

// Available attributes for the picker
const allAttributes = (data.attributes || []) as Array<{
  id: number;
  name: string;
  code: string;
  entity_type: string;
  type: string;
}>;

const availableAttributes = computed(() =>
  allAttributes.map((a) => ({
    id: a.id,
    display: `${a.name} (${a.code}) [${a.entity_type}]`,
  })),
);

// Initialize form attrs from existing data
const existingAttrs = (data.form_attributes || []) as Array<{
  attribute_id: number;
  name: string | null;
  placeholder: string | null;
  is_required: boolean;
  sort_order: number | null;
}>;

const formAttrs = ref<FormAttr[]>(
  existingAttrs.map((a) => ({
    attribute_id: a.attribute_id,
    name: a.name || "",
    placeholder: a.placeholder || "",
    is_required: a.is_required,
    sort_order: a.sort_order || 0,
  })),
);

function addAttribute() {
  formAttrs.value.push({
    attribute_id: null,
    name: "",
    placeholder: "",
    is_required: false,
    sort_order: formAttrs.value.length,
  });
}

const successActions = [
  { label: "Show Message", value: "message" },
  { label: "Redirect to URL", value: "redirect" },
];

const colorFields = [
  { key: "background_color" as const, label: "Background Color" },
  { key: "form_background_color" as const, label: "Form Background Color" },
  { key: "form_title_color" as const, label: "Title Color" },
  { key: "form_submit_button_color" as const, label: "Button Color" },
  { key: "attribute_label_color" as const, label: "Label Color" },
];

const form = reactive({
  title: webForm?.title || "",
  description: webForm?.description || "",
  submit_button_label: webForm?.submit_button_label || "Submit",
  submit_success_action: webForm?.submit_success_action || "message",
  submit_success_content: webForm?.submit_success_content || "Thank you for your submission.",
  create_lead: webForm?.create_lead ?? true,
  background_color: webForm?.background_color || "#F7F8F9",
  form_background_color: webForm?.form_background_color || "#FFFFFF",
  form_title_color: webForm?.form_title_color || "#263238",
  form_submit_button_color: webForm?.form_submit_button_color || "#0E90D9",
  attribute_label_color: webForm?.attribute_label_color || "#546E7A",
});

const rules = {
  required: (v: string) => !!v || "Required",
};

const saving = ref(false);
const snackbar = ref(false);
const snackbarText = ref("");
const snackbarColor = ref("success");

async function save() {
  if (!form.title) {
    snackbarText.value = "Title is required.";
    snackbarColor.value = "error";
    snackbar.value = true;
    return;
  }

  saving.value = true;
  try {
    const payload = {
      ...form,
      attributes: formAttrs.value
        .filter((a) => a.attribute_id != null)
        .map((a, idx) => ({
          attribute_id: a.attribute_id,
          name: a.name || null,
          placeholder: a.placeholder || null,
          is_required: a.is_required,
          sort_order: idx,
        })),
    };

    if (isEdit.value) {
      await store.update(webForm.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/settings/web-forms";
  } catch (err: any) {
    snackbarText.value = err.message || "Failed to save.";
    snackbarColor.value = "error";
    snackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
