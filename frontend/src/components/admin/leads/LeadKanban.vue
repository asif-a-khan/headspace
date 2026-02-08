<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5 font-weight-bold">Kanban Board</h1>
      <v-spacer />
      <v-text-field
        v-model="searchQuery"
        density="compact"
        hide-details
        placeholder="Search leads..."
        prepend-inner-icon="mdi-magnify"
        clearable
        style="max-width: 200px"
        class="mr-3"
      />
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
      <v-btn variant="outlined" class="mr-2" href="/admin/leads/list">
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

    <div class="kanban-board d-flex ga-3 pb-4">
      <div
        v-for="stage in pipelineStages"
        :key="stage.id"
        class="kanban-column"
      >
        <v-card variant="outlined" class="kanban-column-card">
          <div class="kanban-column-header px-3 py-2">
            <div class="d-flex align-center">
              <span class="text-subtitle-2 font-weight-medium">
                {{ stage.stage_name }} ({{ getStageCards(stage.id).length }})
              </span>
              <v-spacer />
              <span v-if="getStageValue(stage.id)" class="text-caption text-success font-weight-medium">
                ${{ getStageValue(stage.id) }}
              </span>
            </div>
            <v-progress-linear
              :model-value="getStageValuePercent(stage.id)"
              color="success"
              height="4"
              class="mt-1"
              rounded
            />
          </div>
          <v-card-text class="kanban-cards pa-2">
            <draggable
              :list="getStageCards(stage.id)"
              group="kanban"
              item-key="id"
              ghost-class="kanban-ghost"
              handle=".kanban-card"
              :animation="200"
              class="kanban-drop-zone"
              @change="(evt: any) => onStageChange(stage.id, evt)"
            >
              <template #item="{ element }">
                <div class="kanban-card mb-2">
                  <v-card
                    variant="flat"
                    class="pa-3 kanban-card-inner"
                    :class="{ 'rotten-card': element.rotten_days && element.rotten_days > 0 }"
                    @click="goToLead(element.id)"
                  >
                    <!-- Person + Org with avatar -->
                    <div v-if="element.person_name" class="d-flex align-start mb-2">
                      <v-avatar size="36" color="primary" variant="tonal" class="mr-2 flex-shrink-0">
                        <span class="text-caption font-weight-bold">
                          {{ element.person_name.charAt(0).toUpperCase() }}
                        </span>
                      </v-avatar>
                      <div class="flex-grow-1" style="min-width: 0">
                        <div class="text-body-2 font-weight-medium text-truncate">
                          {{ element.person_name }}
                        </div>
                        <div v-if="element.organization_name" class="text-caption text-medium-emphasis text-truncate">
                          {{ element.organization_name }}
                        </div>
                      </div>
                      <v-tooltip v-if="element.rotten_days && element.rotten_days > 0" location="top">
                        <template #activator="{ props }">
                          <v-icon v-bind="props" color="error" size="small" class="ml-1 flex-shrink-0">mdi-alert-circle</v-icon>
                        </template>
                        Rotten for {{ element.rotten_days }} day{{ element.rotten_days === 1 ? '' : 's' }}
                      </v-tooltip>
                    </div>

                    <!-- Rotten icon when no person -->
                    <div v-else-if="element.rotten_days && element.rotten_days > 0" class="d-flex justify-end mb-1">
                      <v-tooltip location="top">
                        <template #activator="{ props }">
                          <v-icon v-bind="props" color="error" size="small">mdi-alert-circle</v-icon>
                        </template>
                        Rotten for {{ element.rotten_days }} day{{ element.rotten_days === 1 ? '' : 's' }}
                      </v-tooltip>
                    </div>

                    <!-- Lead title -->
                    <div class="text-subtitle-2 font-weight-medium mb-2">{{ element.title }}</div>

                    <!-- Badge chips -->
                    <div class="d-flex flex-wrap ga-1">
                      <v-chip v-if="element.user_name" size="x-small" variant="tonal" color="info" prepend-icon="mdi-account">
                        {{ element.user_name }}
                      </v-chip>
                      <v-chip v-if="element.lead_value" size="x-small" variant="tonal" color="success" prepend-icon="mdi-currency-usd">
                        {{ Number(element.lead_value).toLocaleString() }}
                      </v-chip>
                      <v-chip v-if="element.source_name" size="x-small" variant="tonal" prepend-icon="mdi-source-branch">
                        {{ element.source_name }}
                      </v-chip>
                      <v-chip v-if="element.type_name" size="x-small" variant="tonal" prepend-icon="mdi-tag">
                        {{ element.type_name }}
                      </v-chip>
                    </div>

                    <!-- Tags -->
                    <div v-if="element.tags_json && element.tags_json.length" class="d-flex flex-wrap ga-1 mt-1">
                      <v-chip
                        v-for="tag in element.tags_json"
                        :key="tag.id"
                        :color="tag.color || '#6366F1'"
                        size="x-small"
                        variant="tonal"
                      >
                        {{ tag.name }}
                      </v-chip>
                    </div>
                  </v-card>
                </div>
              </template>
            </draggable>
            <div v-if="!getStageCards(stage.id).length" class="text-center pa-6">
              <v-icon size="40" color="grey-lighten-1" class="mb-2">mdi-clipboard-text-outline</v-icon>
              <div class="text-caption text-medium-emphasis">No leads in this stage</div>
            </div>
          </v-card-text>
        </v-card>
      </div>
    </div>

    <v-snackbar v-model="errorSnackbar" color="error" :timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted } from "vue";
import draggable from "vuedraggable";
import { useLeadsStore, type KanbanCard } from "@/stores/admin/leads";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("leads.create") || data.permission_type === "all");

const store = useLeadsStore();

const pipelineItems = computed(() => data.pipelines || []);
const allStages = computed(() => data.stages || []);

const defaultPipeline = pipelineItems.value.find((p: any) => p.is_default) || pipelineItems.value[0];
const selectedPipeline = ref<number | null>(defaultPipeline?.id || null);
const searchQuery = ref("");

const errorSnackbar = ref(false);
const errorMessage = ref("");

const pipelineStages = computed(() =>
  allStages.value.filter((s: any) => s.lead_pipeline_id === selectedPipeline.value)
);

// Reactive grouped cards — vuedraggable needs mutable arrays per column
const cardsByStage = reactive<Record<number, KanbanCard[]>>({});

// Re-bucket store cards into cardsByStage whenever store or pipeline changes
watch(
  [() => store.kanbanCards, pipelineStages, searchQuery],
  () => {
    const q = searchQuery.value?.toLowerCase() || "";
    for (const stage of pipelineStages.value) {
      let cards = store.kanbanCards.filter((c) => c.lead_pipeline_stage_id === stage.id);
      if (q) {
        cards = cards.filter(
          (c) =>
            c.title.toLowerCase().includes(q) ||
            (c.person_name && c.person_name.toLowerCase().includes(q)) ||
            (c.organization_name && c.organization_name.toLowerCase().includes(q))
        );
      }
      cardsByStage[stage.id] = cards;
    }
  },
  { immediate: true, deep: true }
);

function getStageCards(stageId: number): KanbanCard[] {
  return cardsByStage[stageId] || [];
}

function getStageValue(stageId: number): string {
  const cards = store.kanbanCards.filter((c) => c.lead_pipeline_stage_id === stageId);
  const total = cards.reduce((sum, c) => sum + (c.lead_value ? Number(c.lead_value) : 0), 0);
  return total ? total.toLocaleString() : "";
}

const totalPipelineValue = computed(() =>
  store.kanbanCards.reduce((sum, c) => sum + (c.lead_value ? Number(c.lead_value) : 0), 0)
);

function getStageValuePercent(stageId: number): number {
  if (totalPipelineValue.value <= 0) return 0;
  const cards = store.kanbanCards.filter((c) => c.lead_pipeline_stage_id === stageId);
  const stageTotal = cards.reduce((sum, c) => sum + (c.lead_value ? Number(c.lead_value) : 0), 0);
  return (stageTotal / totalPipelineValue.value) * 100;
}

async function onStageChange(stageId: number, evt: any) {
  // Only handle 'added' events (card dropped into this column)
  if (!evt.added) return;

  const card = evt.added.element as KanbanCard;
  if (!card || card.lead_pipeline_stage_id === stageId) return;

  // Update local store state so reactivity stays correct
  const storeCard = store.kanbanCards.find((c) => c.id === card.id);
  if (storeCard) {
    storeCard.lead_pipeline_stage_id = stageId;
  }

  try {
    await store.moveToStage(card.id, stageId);
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to move lead.";
    errorSnackbar.value = true;
    await loadKanban();
  }
}

function goToLead(id: number) {
  window.location.href = `/admin/leads/${id}`;
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
  height: 65vh;
  overflow-x: auto;
  scrollbar-width: none; /* Firefox */
}
.kanban-board::-webkit-scrollbar {
  display: none; /* Chrome/Safari/Edge */
}
.kanban-column {
  width: 275px;
  min-width: 275px;
  max-width: 275px;
  flex-shrink: 0;
}
.kanban-column {
  display: flex;
  flex-direction: column;
}
.kanban-column-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.kanban-column-header {
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
.kanban-cards {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.kanban-drop-zone {
  min-height: 60px;
}
.kanban-card {
  cursor: grab;
}
.kanban-card:active {
  cursor: grabbing;
}
.kanban-card-inner {
  cursor: grab;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
.rotten-card {
  border-left: 3px solid rgb(var(--v-theme-error)) !important;
}

/* Drag ghost styling */
:deep(.kanban-ghost) {
  opacity: 0.5;
}
:deep(.kanban-ghost .kanban-card-inner) {
  border: 2px dashed rgb(var(--v-theme-primary)) !important;
}
</style>
