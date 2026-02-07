<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Lead" : "Create Lead" }}</h1>

    <v-card max-width="700">
      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <v-text-field
            v-model="form.title"
            label="Title"
            :rules="[rules.required]"
            class="mb-3"
          />

          <v-textarea
            v-model="form.description"
            label="Description"
            rows="3"
            class="mb-3"
          />

          <div class="d-flex ga-3 mb-3">
            <v-text-field
              v-model="form.lead_value"
              label="Lead Value"
              type="number"
              step="0.01"
              prefix="$"
            />
            <v-text-field
              v-model="form.expected_close_date"
              label="Expected Close Date"
              type="date"
            />
          </div>

          <v-select
            v-model="form.person_id"
            :items="personItems"
            item-title="name"
            item-value="id"
            label="Contact Person"
            clearable
            class="mb-3"
          />

          <div class="d-flex ga-3 mb-3">
            <v-select
              v-model="form.lead_pipeline_id"
              :items="pipelineItems"
              item-title="name"
              item-value="id"
              label="Pipeline"
              :rules="[rules.required]"
              @update:model-value="onPipelineChange"
            />
            <v-select
              v-model="form.lead_pipeline_stage_id"
              :items="filteredStages"
              item-title="stage_name"
              item-value="id"
              label="Stage"
              :rules="[rules.required]"
            />
          </div>

          <div class="d-flex ga-3 mb-3">
            <v-select
              v-model="form.lead_source_id"
              :items="sourceItems"
              item-title="name"
              item-value="id"
              label="Source"
              clearable
            />
            <v-select
              v-model="form.lead_type_id"
              :items="typeItems"
              item-title="name"
              item-value="id"
              label="Type"
              clearable
            />
          </div>

          <v-select
            v-model="form.user_id"
            :items="userItems"
            item-title="label"
            item-value="id"
            label="Assigned To"
            clearable
            class="mb-3"
          />
        </v-form>
      </v-card-text>
      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/leads" variant="text">Cancel</v-btn>
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
import { useLeadsStore } from "@/stores/admin/leads";

const data = window.__INITIAL_DATA__ || {};
const store = useLeadsStore();
const isEdit = computed(() => !!data.lead);

const pipelineItems = computed(() => data.pipelines || []);
const allStages = computed(() => data.stages || []);
const sourceItems = computed(() => data.sources || []);
const typeItems = computed(() => data.types || []);
const personItems = computed(() => data.persons || []);
const userItems = computed(() =>
  (data.users || []).map((u: any) => ({
    id: u.id,
    label: `${u.first_name} ${u.last_name}`,
  }))
);

const lead = data.lead;
const form = reactive({
  title: lead?.title || "",
  description: lead?.description || "",
  lead_value: lead?.lead_value || "",
  expected_close_date: lead?.expected_close_date || "",
  person_id: lead?.person_id || null,
  lead_pipeline_id: lead?.lead_pipeline_id || null,
  lead_pipeline_stage_id: lead?.lead_pipeline_stage_id || null,
  lead_source_id: lead?.lead_source_id || null,
  lead_type_id: lead?.lead_type_id || null,
  user_id: lead?.user_id || null,
});

// If no pipeline selected, pick default
if (!form.lead_pipeline_id && pipelineItems.value.length) {
  const defaultPipeline = pipelineItems.value.find((p: any) => p.is_default) || pipelineItems.value[0];
  form.lead_pipeline_id = defaultPipeline.id;
}

const filteredStages = computed(() =>
  allStages.value.filter((s: any) => s.lead_pipeline_id === form.lead_pipeline_id)
);

// If no stage selected, pick first of filtered stages
if (!form.lead_pipeline_stage_id && filteredStages.value.length) {
  form.lead_pipeline_stage_id = filteredStages.value[0].id;
}

function onPipelineChange() {
  const stages = filteredStages.value;
  form.lead_pipeline_stage_id = stages.length ? stages[0].id : null;
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
    const payload = {
      title: form.title,
      description: form.description || null,
      lead_value: form.lead_value ? Number(form.lead_value) : null,
      expected_close_date: form.expected_close_date || null,
      person_id: form.person_id || null,
      lead_pipeline_id: form.lead_pipeline_id,
      lead_pipeline_stage_id: form.lead_pipeline_stage_id,
      lead_source_id: form.lead_source_id || null,
      lead_type_id: form.lead_type_id || null,
      user_id: form.user_id || null,
    };

    if (isEdit.value) {
      await store.update(lead.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/leads";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
