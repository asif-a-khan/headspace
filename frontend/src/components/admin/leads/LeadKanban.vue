<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Kanban Board</h1>
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
          <v-card-title class="text-subtitle-1 bg-surface-variant py-2 px-3">
            <div class="d-flex align-center">
              {{ stage.stage_name }}
              <v-spacer />
              <v-chip size="x-small" variant="tonal">{{ getStageCards(stage.id).length }}</v-chip>
            </div>
            <div v-if="getStageValue(stage.id)" class="text-caption text-success font-weight-medium">
              ${{ getStageValue(stage.id) }}
            </div>
          </v-card-title>
          <v-card-text class="kanban-cards pa-2">
            <draggable
              :list="getStageCards(stage.id)"
              group="kanban"
              item-key="id"
              class="kanban-drop-zone"
              @change="(evt: any) => onStageChange(stage.id, evt)"
            >
              <template #item="{ element }">
                <div class="kanban-card mb-2">
                  <v-card
                    variant="elevated"
                    class="pa-3 kanban-card-inner"
                    :class="{ 'rotten-card': element.rotten_days && element.rotten_days > 0 }"
                    @click="goToLead(element.id)"
                  >
                    <div class="d-flex align-center">
                      <div class="text-subtitle-2 font-weight-medium mb-1 flex-grow-1">{{ element.title }}</div>
                      <v-tooltip v-if="element.rotten_days && element.rotten_days > 0" location="top">
                        <template #activator="{ props }">
                          <v-icon v-bind="props" color="error" size="small">mdi-alert-circle</v-icon>
                        </template>
                        Rotten for {{ element.rotten_days }} day{{ element.rotten_days === 1 ? '' : 's' }}
                      </v-tooltip>
                    </div>
                    <div v-if="element.person_name" class="text-caption text-medium-emphasis">
                      <v-icon size="x-small" class="mr-1">mdi-account</v-icon>{{ element.person_name }}
                    </div>
                    <div v-if="element.lead_value" class="text-caption text-success font-weight-medium mt-1">
                      ${{ Number(element.lead_value).toLocaleString() }}
                    </div>
                  </v-card>
                </div>
              </template>
            </draggable>
            <div v-if="!getStageCards(stage.id).length" class="text-center text-caption text-medium-emphasis pa-4">
              No leads
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
            (c.person_name && c.person_name.toLowerCase().includes(q))
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
}
.rotten-card {
  border-left: 3px solid rgb(var(--v-theme-error)) !important;
}
</style>
