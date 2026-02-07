<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Kanban Board</h1>
      <v-spacer />
      <v-select
        v-model="selectedPipeline"
        :items="pipelineItems"
        item-title="name"
        item-value="id"
        label="Pipeline"
        density="compact"
        hide-details
        style="max-width: 240px"
        class="mr-3"
        @update:model-value="loadKanban"
      />
      <v-btn variant="outlined" class="mr-2" href="/admin/leads">
        <v-icon start>mdi-view-list</v-icon>
        List View
      </v-btn>
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/leads/create"
      >
        Create Lead
      </v-btn>
    </div>

    <div class="kanban-board d-flex ga-3 overflow-x-auto pb-4">
      <div
        v-for="stage in pipelineStages"
        :key="stage.id"
        class="kanban-column"
      >
        <v-card variant="outlined" class="kanban-column-card">
          <v-card-title class="text-subtitle-1 bg-surface-variant py-2 px-3 d-flex align-center">
            {{ stage.stage_name }}
            <v-spacer />
            <v-chip size="x-small" variant="tonal">{{ stageCards(stage.id).length }}</v-chip>
          </v-card-title>
          <v-card-text class="kanban-cards pa-2">
            <div
              v-for="card in stageCards(stage.id)"
              :key="card.id"
              class="kanban-card mb-2"
            >
              <v-card
                variant="elevated"
                :href="`/admin/leads/${card.id}/edit`"
                class="pa-3"
              >
                <div class="text-subtitle-2 font-weight-medium mb-1">{{ card.title }}</div>
                <div v-if="card.person_name" class="text-caption text-medium-emphasis">
                  <v-icon size="x-small" class="mr-1">mdi-account</v-icon>{{ card.person_name }}
                </div>
                <div v-if="card.lead_value" class="text-caption text-success font-weight-medium mt-1">
                  ${{ Number(card.lead_value).toLocaleString() }}
                </div>
              </v-card>
            </div>
            <div v-if="!stageCards(stage.id).length" class="text-center text-caption text-medium-emphasis pa-4">
              No leads
            </div>
          </v-card-text>
        </v-card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useLeadsStore, type KanbanCard } from "@/stores/admin/leads";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("leads.create") || data.permission_type === "all");

const store = useLeadsStore();

const pipelineItems = computed(() => data.pipelines || []);
const allStages = computed(() => data.stages || []);

// Auto-select default pipeline
const defaultPipeline = pipelineItems.value.find((p: any) => p.is_default) || pipelineItems.value[0];
const selectedPipeline = ref<number | null>(defaultPipeline?.id || null);

const pipelineStages = computed(() =>
  allStages.value.filter((s: any) => s.lead_pipeline_id === selectedPipeline.value)
);

function stageCards(stageId: number): KanbanCard[] {
  return store.kanbanCards.filter((c) => c.lead_pipeline_stage_id === stageId);
}

async function loadKanban() {
  if (selectedPipeline.value) {
    await store.fetchKanban(selectedPipeline.value);
  }
}

onMounted(() => {
  loadKanban();
});
</script>

<style scoped>
.kanban-board {
  min-height: 500px;
}
.kanban-column {
  min-width: 280px;
  max-width: 320px;
  flex-shrink: 0;
}
.kanban-column-card {
  height: 100%;
}
.kanban-cards {
  min-height: 200px;
  max-height: calc(100vh - 280px);
  overflow-y: auto;
}
</style>
