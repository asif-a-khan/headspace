<template>
  <div>
    <h1 class="text-h5 font-weight-bold mb-2">Dashboard</h1>
    <p class="text-body-1 text-secondary mb-6">Welcome to {{ companyName }}</p>

    <!-- Stat Cards -->
    <v-row class="mb-4">
      <v-col cols="12" sm="6" md="3">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center">
              <v-avatar color="primary" size="40" class="mr-3">
                <v-icon color="white">mdi-filter-variant</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">{{ stats.total_leads }}</p>
                <p class="text-body-2 text-secondary">Total Leads</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" sm="6" md="3">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center">
              <v-avatar color="success" size="40" class="mr-3">
                <v-icon color="white">mdi-handshake</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">{{ stats.won_deals?.count || 0 }}</p>
                <p class="text-body-2 text-secondary">Won Deals</p>
                <p class="text-caption text-success" v-if="stats.won_deals?.value">
                  ${{ Number(stats.won_deals.value).toLocaleString() }}
                </p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" sm="6" md="3">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center">
              <v-avatar color="warning" size="40" class="mr-3">
                <v-icon color="white">mdi-currency-usd</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">
                  ${{ stats.open_revenue ? Number(stats.open_revenue).toLocaleString() : '0' }}
                </p>
                <p class="text-body-2 text-secondary">Open Revenue</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" sm="6" md="3">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center">
              <v-avatar color="info" size="40" class="mr-3">
                <v-icon color="white">mdi-calendar-check</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">{{ stats.activities_due }}</p>
                <p class="text-body-2 text-secondary">Activities Due Today</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Charts Row -->
    <v-row>
      <v-col cols="12" md="7">
        <v-card rounded="lg" elevation="0">
          <v-card-title class="text-subtitle-1 font-weight-medium">Pipeline Funnel</v-card-title>
          <v-card-text>
            <div v-if="stats.leads_by_stage?.length" class="pipeline-bars">
              <div
                v-for="stage in stats.leads_by_stage"
                :key="stage.stage_name"
                class="d-flex align-center mb-3"
              >
                <span class="text-body-2 mr-3" style="min-width: 100px">{{ stage.stage_name }}</span>
                <div class="flex-grow-1 mr-3">
                  <v-progress-linear
                    :model-value="maxStageCount ? (stage.count / maxStageCount) * 100 : 0"
                    color="primary"
                    height="24"
                    rounded
                  >
                    <template #default>
                      <span class="text-caption font-weight-medium">{{ stage.count }}</span>
                    </template>
                  </v-progress-linear>
                </div>
              </div>
            </div>
            <div v-else class="text-center text-medium-emphasis pa-8">
              No open leads yet
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" md="5">
        <v-card rounded="lg" elevation="0">
          <v-card-title class="text-subtitle-1 font-weight-medium">Revenue by Source</v-card-title>
          <v-card-text>
            <v-list v-if="stats.revenue_by_source?.length" density="compact">
              <v-list-item
                v-for="src in stats.revenue_by_source"
                :key="src.source_name"
              >
                <template #prepend>
                  <v-icon :color="sourceColor(src.source_name)" size="small" class="mr-2">mdi-circle</v-icon>
                </template>
                <v-list-item-title>{{ src.source_name }}</v-list-item-title>
                <template #append>
                  <span class="font-weight-medium">${{ Number(src.total).toLocaleString() }}</span>
                </template>
              </v-list-item>
            </v-list>
            <div v-else class="text-center text-medium-emphasis pa-8">
              No won deals yet
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { get } from "@/api/client";

const data = (window as any).__INITIAL_DATA__ || {};
const companyName = computed(() => data.company_name || "Headspace");

interface DashboardStats {
  total_leads: number;
  open_leads: number;
  won_deals: { count: number; value: string | null };
  open_revenue: string | null;
  activities_due: number;
  leads_by_stage: Array<{ stage_name: string; count: number }>;
  revenue_by_source: Array<{ source_name: string; total: string }>;
}

const stats = ref<DashboardStats>({
  total_leads: 0,
  open_leads: 0,
  won_deals: { count: 0, value: null },
  open_revenue: null,
  activities_due: 0,
  leads_by_stage: [],
  revenue_by_source: [],
});

const maxStageCount = computed(() => {
  if (!stats.value.leads_by_stage.length) return 0;
  return Math.max(...stats.value.leads_by_stage.map((s) => s.count));
});

const sourceColors = ["#6366F1", "#10B981", "#F59E0B", "#EF4444", "#8B5CF6", "#06B6D4"];
function sourceColor(name: string | null): string {
  if (!name) return sourceColors[0];
  const idx = Math.abs(name.split("").reduce((a, c) => a + c.charCodeAt(0), 0)) % sourceColors.length;
  return sourceColors[idx];
}

onMounted(async () => {
  try {
    const res = await get<DashboardStats>("/admin/api/dashboard/stats");
    stats.value = res;
  } catch {
    // Dashboard stats failed to load, keep defaults
  }
});
</script>
