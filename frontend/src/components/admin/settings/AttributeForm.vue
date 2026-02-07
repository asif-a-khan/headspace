<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Attribute" : "Create Attribute" }}</h1>
    <v-card max-width="700">
      <v-card-text>
        <v-alert v-if="error" type="error" density="compact" class="mb-4">
          {{ error }}
        </v-alert>
        <v-form @submit.prevent="submit">
          <v-text-field
            v-model="form.code"
            label="Code"
            :readonly="isEdit"
            :hint="isEdit ? 'Code cannot be changed after creation' : 'Unique identifier (e.g. custom_field_1)'"
            persistent-hint
            required
          />
          <v-text-field v-model="form.name" label="Name" required class="mt-2" />
          <v-select
            v-model="form.entity_type"
            :items="entityTypes"
            item-title="label"
            item-value="value"
            label="Entity Type"
            :disabled="isEdit"
            required
            class="mt-2"
          />
          <v-select
            v-model="form.type"
            :items="attributeTypes"
            item-title="label"
            item-value="value"
            label="Type"
            :disabled="isEdit"
            required
            class="mt-2"
          />

          <!-- Options for select/multiselect -->
          <div v-if="hasOptions" class="mt-4">
            <div class="d-flex align-center mb-2">
              <span class="text-subtitle-1">Options</span>
              <v-spacer />
              <v-btn size="small" variant="text" prepend-icon="mdi-plus" @click="addOption">
                Add Option
              </v-btn>
            </div>
            <div v-for="(opt, idx) in form.options" :key="idx" class="d-flex align-center gap-2 mb-1">
              <v-text-field
                v-model="opt.name"
                :label="`Option ${idx + 1}`"
                density="compact"
                hide-details
              />
              <v-btn
                icon="mdi-close"
                size="x-small"
                variant="text"
                color="error"
                @click="removeOption(idx)"
              />
            </div>
          </div>

          <v-text-field
            v-model="form.validation"
            label="Validation"
            hint="e.g. numeric, decimal"
            persistent-hint
            class="mt-2"
          />
          <v-text-field
            v-model.number="form.sort_order"
            label="Sort Order"
            type="number"
            class="mt-2"
          />
          <div class="d-flex flex-wrap gap-x-6 mt-2">
            <v-switch v-model="form.is_required" label="Required" hide-details />
            <v-switch v-model="form.is_unique" label="Unique" hide-details />
            <v-switch v-model="form.quick_add" label="Quick Add" hide-details />
          </div>
          <div class="d-flex gap-2 mt-6">
            <v-btn type="submit" color="primary" :loading="loading">
              {{ isEdit ? "Update" : "Create" }}
            </v-btn>
            <v-btn href="/admin/settings/attributes" variant="text">Cancel</v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { useAttributesStore } from "@/stores/admin/attributes";

const data = window.__INITIAL_DATA__ || {};
const store = useAttributesStore();
const isEdit = !!data.attribute;
const error = ref("");
const loading = ref(false);

const entityTypes = [
  { label: "Leads", value: "leads" },
  { label: "Persons", value: "persons" },
  { label: "Organizations", value: "organizations" },
  { label: "Products", value: "products" },
  { label: "Quotes", value: "quotes" },
];

const attributeTypes = [
  { label: "Text", value: "text" },
  { label: "Textarea", value: "textarea" },
  { label: "Boolean", value: "boolean" },
  { label: "Integer", value: "integer" },
  { label: "Decimal", value: "decimal" },
  { label: "Date", value: "date" },
  { label: "Date & Time", value: "datetime" },
  { label: "Select", value: "select" },
  { label: "Multi Select", value: "multiselect" },
  { label: "Email", value: "email" },
  { label: "Phone", value: "phone" },
  { label: "Address", value: "address" },
  { label: "Image", value: "image" },
  { label: "File", value: "file" },
];

interface FormOption {
  id?: number;
  name: string;
  sort_order: number;
  is_delete?: boolean;
}

const existingOptions: FormOption[] = (data.options || []).map(
  (o: { id: number; name: string; sort_order: number }) => ({
    id: o.id,
    name: o.name,
    sort_order: o.sort_order,
  }),
);

const form = reactive({
  code: data.attribute?.code || "",
  name: data.attribute?.name || "",
  type: data.attribute?.type || "text",
  entity_type: data.attribute?.entity_type || "leads",
  sort_order: data.attribute?.sort_order ?? 0,
  validation: data.attribute?.validation || "",
  is_required: data.attribute?.is_required ?? false,
  is_unique: data.attribute?.is_unique ?? false,
  quick_add: data.attribute?.quick_add ?? false,
  options: existingOptions.length > 0 ? existingOptions : ([] as FormOption[]),
});

const hasOptions = computed(
  () => form.type === "select" || form.type === "multiselect",
);

function addOption() {
  form.options.push({ name: "", sort_order: form.options.length });
}

function removeOption(idx: number) {
  const opt = form.options[idx];
  if (opt.id) {
    // Mark for deletion instead of removing (so backend can delete it)
    opt.is_delete = true;
  } else {
    form.options.splice(idx, 1);
  }
}

async function submit() {
  error.value = "";
  loading.value = true;
  try {
    // Filter out deleted options with no ID (newly added then removed)
    const payload = {
      ...form,
      options: hasOptions.value
        ? form.options.filter((o) => o.id || !o.is_delete)
        : undefined,
    };
    if (isEdit) {
      await store.update(data.attribute.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/settings/attributes";
  } catch (e: any) {
    error.value = e.message || "Failed to save";
  } finally {
    loading.value = false;
  }
}
</script>
