<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Pipeline" : "Create Pipeline" }}</h1>
    <v-card max-width="700">
      <v-card-text>
        <v-alert v-if="error" type="error" density="compact" class="mb-4">
          {{ error }}
        </v-alert>
        <v-form @submit.prevent="submit">
          <v-text-field v-model="form.name" label="Name" required />
          <v-text-field
            v-model.number="form.rotten_days"
            label="Rotten Days"
            type="number"
            hint="Days before a lead is considered stale"
            persistent-hint
            class="mt-2"
          />
          <v-switch v-model="form.is_default" label="Default Pipeline" class="mt-2" />

          <div class="mt-4">
            <div class="d-flex align-center mb-2">
              <span class="text-subtitle-1">Stages</span>
            </div>
            <v-table density="compact">
              <thead>
                <tr>
                  <th>Stage</th>
                  <th width="120">Probability %</th>
                  <th width="60"></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(ps, idx) in form.stages" :key="idx">
                  <td>
                    <v-select
                      v-model="ps.lead_stage_id"
                      :items="availableStages"
                      item-title="name"
                      item-value="id"
                      density="compact"
                      hide-details
                      variant="underlined"
                    />
                  </td>
                  <td>
                    <v-text-field
                      v-model.number="ps.probability"
                      type="number"
                      density="compact"
                      hide-details
                      variant="underlined"
                      min="0"
                      max="100"
                    />
                  </td>
                  <td>
                    <v-btn icon="mdi-close" size="x-small" variant="text" color="error" @click="removeStage(idx)" />
                  </td>
                </tr>
              </tbody>
            </v-table>
            <v-btn size="small" variant="text" prepend-icon="mdi-plus" class="mt-1" @click="addStage">
              Add Stage
            </v-btn>
          </div>

          <div class="d-flex gap-2 mt-6">
            <v-btn type="submit" color="primary" :loading="loading">
              {{ isEdit ? "Update" : "Create" }}
            </v-btn>
            <v-btn href="/admin/settings/pipelines" variant="text">Cancel</v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { usePipelinesStore } from "@/stores/admin/pipelines";

const data = window.__INITIAL_DATA__ || {};
const store = usePipelinesStore();
const isEdit = !!data.pipeline;
const error = ref("");
const loading = ref(false);

const breadcrumbs = [
  { title: "Settings", href: "/admin/settings" },
  { title: "Pipelines", href: "/admin/settings/pipelines" },
  { title: data.pipeline ? "Edit" : "Create", disabled: true },
];

const availableStages = data.stages || [];

interface FormStage {
  lead_stage_id: number | null;
  probability: number;
  sort_order: number;
}

const existingStages: FormStage[] = (data.pipeline_stages || []).map(
  (ps: { lead_stage_id: number; probability: number; sort_order: number }) => ({
    lead_stage_id: ps.lead_stage_id,
    probability: ps.probability,
    sort_order: ps.sort_order,
  }),
);

const form = reactive({
  name: data.pipeline?.name || "",
  is_default: data.pipeline?.is_default ?? false,
  rotten_days: data.pipeline?.rotten_days ?? 30,
  stages: existingStages.length > 0 ? existingStages : ([] as FormStage[]),
});

function addStage() {
  form.stages.push({
    lead_stage_id: null,
    probability: 100,
    sort_order: form.stages.length,
  });
}

function removeStage(idx: number) {
  form.stages.splice(idx, 1);
}

async function submit() {
  error.value = "";
  loading.value = true;
  try {
    const payload = {
      ...form,
      stages: form.stages
        .filter((s) => s.lead_stage_id != null)
        .map((s, i) => ({ ...s, sort_order: i })),
    };
    if (isEdit) {
      await store.update(data.pipeline.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/settings/pipelines";
  } catch (e: any) {
    error.value = e.message || "Failed to save";
  } finally {
    loading.value = false;
  }
}
</script>
