<template>
  <div>
    <!-- Header with date filter -->
    <div class="d-flex align-center justify-space-between mb-6 flex-wrap ga-2">
      <div>
        <h1 class="text-h5 font-weight-bold mb-1">Dashboard</h1>
        <p class="text-body-2 text-secondary">Welcome to {{ companyName }}</p>
      </div>
      <div class="d-flex align-center ga-3">
        <v-text-field
          v-model="startDate"
          type="date"
          label="From"
          density="compact"
          hide-details
          variant="outlined"
          style="max-width: 170px"
          @change="loadStats"
        />
        <v-text-field
          v-model="endDate"
          type="date"
          label="To"
          density="compact"
          hide-details
          variant="outlined"
          style="max-width: 170px"
          @change="loadStats"
        />
        <v-btn size="small" variant="tonal" @click="resetDates">Reset</v-btn>
      </div>
    </div>

    <!-- Two-column layout matching Krayin -->
    <div class="dashboard-layout d-flex ga-4" style="align-items: flex-start">
      <!-- LEFT Column (main content) -->
      <div style="flex: 1; min-width: 0">
        <!-- Revenue Stats Card -->
        <v-card  class="mb-4">
          <v-card-text class="pa-5">
            <!-- Won Revenue Bar -->
            <div class="d-flex align-center mb-4">
              <div style="width: 120px" class="text-body-2 font-weight-medium">Won Revenue</div>
              <div style="flex: 1" class="mx-3">
                <div
                  class="revenue-bar rounded"
                  :style="{ width: wonBarWidth + '%', backgroundColor: '#10B981', minWidth: '4px' }"
                />
              </div>
              <div style="width: 140px" class="text-right">
                <span class="text-body-1 font-weight-bold" style="color: #059669">${{ fmtNum(stats.won_revenue?.current) }}</span>
                <div class="d-flex align-center justify-end">
                  <v-icon
                    v-if="stats.won_revenue?.progress !== 0"
                    :color="(stats.won_revenue?.progress || 0) >= 0 ? 'success' : 'error'"
                    size="14"
                    class="mr-1"
                  >
                    {{ (stats.won_revenue?.progress || 0) >= 0 ? 'mdi-trending-up' : 'mdi-trending-down' }}
                  </v-icon>
                  <span
                    class="text-caption"
                    :class="(stats.won_revenue?.progress || 0) >= 0 ? 'text-success' : 'text-error'"
                  >
                    {{ formatProgress(stats.won_revenue?.progress) }}
                  </span>
                </div>
              </div>
            </div>
            <!-- Lost Revenue Bar -->
            <div class="d-flex align-center">
              <div style="width: 120px" class="text-body-2 font-weight-medium">Lost Revenue</div>
              <div style="flex: 1" class="mx-3">
                <div
                  class="revenue-bar rounded"
                  :style="{ width: lostBarWidth + '%', backgroundColor: '#EF4444', minWidth: '4px' }"
                />
              </div>
              <div style="width: 140px" class="text-right">
                <span class="text-body-1 font-weight-bold text-error">${{ fmtNum(stats.lost_revenue?.current) }}</span>
                <div class="d-flex align-center justify-end">
                  <v-icon
                    v-if="stats.lost_revenue?.progress !== 0"
                    :color="(stats.lost_revenue?.progress || 0) <= 0 ? 'success' : 'error'"
                    size="14"
                    class="mr-1"
                  >
                    {{ (stats.lost_revenue?.progress || 0) <= 0 ? 'mdi-trending-down' : 'mdi-trending-up' }}
                  </v-icon>
                  <span
                    class="text-caption"
                    :class="(stats.lost_revenue?.progress || 0) <= 0 ? 'text-success' : 'text-error'"
                  >
                    {{ formatProgress(stats.lost_revenue?.progress) }}
                  </span>
                </div>
              </div>
            </div>
          </v-card-text>
        </v-card>

        <!-- 6 KPIs in 3x2 grid -->
        <v-row class="mb-4">
          <v-col v-for="kpi in kpis" :key="kpi.label" cols="12" sm="6" md="4">
            <v-card  style="min-height: 100px">
              <v-card-text class="pa-4">
                <div class="text-caption text-medium-emphasis mb-1">{{ kpi.label }}</div>
                <div class="d-flex align-center justify-space-between">
                  <span class="text-h6 font-weight-bold">{{ kpi.prefix }}{{ fmtNum(kpi.value) }}</span>
                  <div v-if="kpi.progress !== 0" class="d-flex align-center">
                    <v-icon
                      :color="kpi.progress >= 0 ? 'success' : 'error'"
                      size="16"
                      class="mr-1"
                    >
                      {{ kpi.progress >= 0 ? 'mdi-trending-up' : 'mdi-trending-down' }}
                    </v-icon>
                    <span
                      class="text-caption font-weight-bold"
                      :class="kpi.progress >= 0 ? 'text-success' : 'text-error'"
                    >
                      {{ formatProgress(kpi.progress) }}
                    </span>
                  </div>
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>

        <!-- Leads Over Time Chart -->
        <v-card  class="mb-4">
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Leads Over Time
          </v-card-title>
          <v-card-text class="pa-4">
            <div style="height: 280px">
              <canvas ref="leadsChartCanvas"></canvas>
            </div>
          </v-card-text>
        </v-card>

        <!-- Top Products + Top Persons -->
        <v-row>
          <v-col cols="12" md="6">
            <v-card >
              <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
                Top Selling Products
              </v-card-title>
              <v-card-text class="pa-4">
                <v-list v-if="stats.top_products?.length" density="compact">
                  <v-list-item
                    v-for="product in stats.top_products"
                    :key="product.sku"
                    href="/admin/products"
                  >
                    <template #prepend>
                      <v-icon color="green" size="small" class="mr-2">mdi-package-variant-closed</v-icon>
                    </template>
                    <v-list-item-title class="text-body-2 font-weight-medium">
                      {{ product.name }}
                    </v-list-item-title>
                    <v-list-item-subtitle class="text-caption">
                      SKU: {{ product.sku }} &middot; Qty: {{ product.total_qty }}
                    </v-list-item-subtitle>
                    <template #append>
                      <span class="text-body-2 font-weight-bold text-success">
                        ${{ fmtNum(product.total_revenue) }}
                      </span>
                    </template>
                  </v-list-item>
                </v-list>
                <div v-else class="text-center text-medium-emphasis pa-8">
                  No product sales yet
                </div>
              </v-card-text>
            </v-card>
          </v-col>
          <v-col cols="12" md="6">
            <v-card >
              <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
                Top Persons
              </v-card-title>
              <v-card-text class="pa-4">
                <v-list v-if="stats.top_persons?.length" density="compact">
                  <v-list-item
                    v-for="person in stats.top_persons"
                    :key="person.id"
                    :href="`/admin/contacts/persons/${person.id}`"
                  >
                    <template #prepend>
                      <v-avatar color="primary" size="32" class="mr-2">
                        <span class="text-white text-caption font-weight-medium">
                          {{ (person.name || 'U').charAt(0).toUpperCase() }}
                        </span>
                      </v-avatar>
                    </template>
                    <v-list-item-title class="text-body-2 font-weight-medium">
                      {{ person.name }}
                    </v-list-item-title>
                    <v-list-item-subtitle class="text-caption">
                      {{ person.email || 'No email' }}
                    </v-list-item-subtitle>
                    <template #append>
                      <v-chip size="x-small" color="primary" variant="tonal">
                        {{ person.total_leads }} lead{{ person.total_leads !== 1 ? 's' : '' }}
                      </v-chip>
                    </template>
                  </v-list-item>
                </v-list>
                <div v-else class="text-center text-medium-emphasis pa-8">
                  No person data yet
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </div>

      <!-- RIGHT Column (sidebar) -->
      <div class="dashboard-sidebar">
        <!-- Open Leads By Stage (funnel) -->
        <v-card  class="mb-4">
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Open Leads By States
          </v-card-title>
          <v-card-text class="pa-4">
            <div style="height: 280px">
              <canvas ref="funnelChartCanvas"></canvas>
            </div>
          </v-card-text>
        </v-card>

        <!-- Revenue by Sources (doughnut) -->
        <v-card  class="mb-4">
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Revenue by Sources
          </v-card-title>
          <v-card-text class="pa-4">
            <div class="d-flex flex-column align-center" style="min-height: 240px">
              <div style="width: 200px; height: 200px">
                <canvas ref="sourceChartCanvas"></canvas>
              </div>
              <div v-if="stats.revenue_by_source?.length" class="mt-3 w-100">
                <div
                  v-for="(src, i) in stats.revenue_by_source"
                  :key="src.source_name"
                  class="d-flex align-center justify-space-between mb-1"
                >
                  <div class="d-flex align-center">
                    <span
                      class="d-inline-block rounded-circle mr-2"
                      :style="{ width: '8px', height: '8px', backgroundColor: doughnutColors[i % doughnutColors.length] }"
                    />
                    <span class="text-caption">{{ src.source_name }}</span>
                  </div>
                  <span class="text-caption font-weight-medium">${{ fmtNum(src.total) }}</span>
                </div>
              </div>
              <div v-else class="mt-3 text-body-2 text-medium-emphasis">No data yet</div>
            </div>
          </v-card-text>
        </v-card>

        <!-- Revenue by Types (doughnut) -->
        <v-card >
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Revenue by Types
          </v-card-title>
          <v-card-text class="pa-4">
            <div class="d-flex flex-column align-center" style="min-height: 240px">
              <div style="width: 200px; height: 200px">
                <canvas ref="typeChartCanvas"></canvas>
              </div>
              <div v-if="stats.revenue_by_type?.length" class="mt-3 w-100">
                <div
                  v-for="(t, i) in stats.revenue_by_type"
                  :key="t.type_name"
                  class="d-flex align-center justify-space-between mb-1"
                >
                  <div class="d-flex align-center">
                    <span
                      class="d-inline-block rounded-circle mr-2"
                      :style="{ width: '8px', height: '8px', backgroundColor: doughnutColors[i % doughnutColors.length] }"
                    />
                    <span class="text-caption">{{ t.type_name }}</span>
                  </div>
                  <span class="text-caption font-weight-medium">${{ fmtNum(t.total) }}</span>
                </div>
              </div>
              <div v-else class="mt-3 text-body-2 text-medium-emphasis">No data yet</div>
            </div>
          </v-card-text>
        </v-card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick, onBeforeUnmount } from "vue";
import { get } from "@/api/client";
import {
  Chart,
  BarController,
  BarElement,
  CategoryScale,
  LinearScale,
  DoughnutController,
  ArcElement,
  Tooltip,
  Legend,
} from "chart.js";

Chart.register(
  BarController,
  BarElement,
  CategoryScale,
  LinearScale,
  DoughnutController,
  ArcElement,
  Tooltip,
  Legend,
);

const data = (window as any).__INITIAL_DATA__ || {};
const companyName = computed(() => data.company_name || "Headspace");

// Date filter — default to last 30 days
function defaultStartDate(): string {
  const d = new Date();
  d.setDate(d.getDate() - 30);
  return d.toISOString().slice(0, 10);
}
function defaultEndDate(): string {
  return new Date().toISOString().slice(0, 10);
}

const startDate = ref(defaultStartDate());
const endDate = ref(defaultEndDate());

interface KpiStat {
  current: number | string;
  previous?: number | string;
  progress: number;
}

interface DashboardStats {
  total_leads: KpiStat;
  avg_lead_value: KpiStat;
  avg_leads_per_day: KpiStat;
  total_quotations: KpiStat;
  total_persons: KpiStat;
  total_organizations: KpiStat;
  won_revenue: KpiStat;
  lost_revenue: KpiStat;
  leads_by_stage: Array<{ stage_name: string; count: number }>;
  revenue_by_source: Array<{ source_name: string; total: string }>;
  revenue_by_type: Array<{ type_name: string; total: string }>;
  leads_over_time: Array<{ period: string; total: number; won: number; lost: number }>;
  top_products: Array<{ name: string; sku: string; total_revenue: string; total_qty: number }>;
  top_persons: Array<{ id: number; name: string; email: string | null; total_leads: number }>;
}

const emptyKpi = (): KpiStat => ({ current: 0, progress: 0 });

const stats = ref<DashboardStats>({
  total_leads: emptyKpi(),
  avg_lead_value: emptyKpi(),
  avg_leads_per_day: emptyKpi(),
  total_quotations: emptyKpi(),
  total_persons: emptyKpi(),
  total_organizations: emptyKpi(),
  won_revenue: emptyKpi(),
  lost_revenue: emptyKpi(),
  leads_by_stage: [],
  revenue_by_source: [],
  revenue_by_type: [],
  leads_over_time: [],
  top_products: [],
  top_persons: [],
});

// KPI grid definitions
const kpis = computed(() => [
  { label: "Average Lead Value", value: stats.value.avg_lead_value?.current, prefix: "$", progress: stats.value.avg_lead_value?.progress || 0 },
  { label: "Total Leads", value: stats.value.total_leads?.current, prefix: "", progress: stats.value.total_leads?.progress || 0 },
  { label: "Average Leads Per Day", value: stats.value.avg_leads_per_day?.current, prefix: "", progress: stats.value.avg_leads_per_day?.progress || 0 },
  { label: "Total Quotations", value: stats.value.total_quotations?.current, prefix: "", progress: stats.value.total_quotations?.progress || 0 },
  { label: "Total Persons", value: stats.value.total_persons?.current, prefix: "", progress: stats.value.total_persons?.progress || 0 },
  { label: "Total Organizations", value: stats.value.total_organizations?.current, prefix: "", progress: stats.value.total_organizations?.progress || 0 },
]);

// Revenue bar widths (relative to max of won+lost)
const wonBarWidth = computed(() => {
  const won = Number(stats.value.won_revenue?.current || 0);
  const lost = Number(stats.value.lost_revenue?.current || 0);
  const max = Math.max(won, lost, 1);
  return (won / max) * 100;
});

const lostBarWidth = computed(() => {
  const won = Number(stats.value.won_revenue?.current || 0);
  const lost = Number(stats.value.lost_revenue?.current || 0);
  const max = Math.max(won, lost, 1);
  return (lost / max) * 100;
});

// Chart refs
const leadsChartCanvas = ref<HTMLCanvasElement | null>(null);
const funnelChartCanvas = ref<HTMLCanvasElement | null>(null);
const sourceChartCanvas = ref<HTMLCanvasElement | null>(null);
const typeChartCanvas = ref<HTMLCanvasElement | null>(null);

let leadsChart: Chart | null = null;
let funnelChart: Chart | null = null;
let sourceChart: Chart | null = null;
let typeChart: Chart | null = null;

const doughnutColors = ["#8979FF", "#FF928A", "#3CC3DF", "#F59E0B", "#10B981", "#8B5CF6", "#EC4899"];

function fmtNum(val: string | number | null | undefined): string {
  if (val == null) return "0";
  return Number(val).toLocaleString();
}

function formatProgress(val: number | null | undefined): string {
  if (val == null || val === 0) return "0%";
  return (val > 0 ? "+" : "") + val.toFixed(1) + "%";
}

function resetDates() {
  startDate.value = defaultStartDate();
  endDate.value = defaultEndDate();
  loadStats();
}

async function loadStats() {
  try {
    const params = new URLSearchParams();
    if (startDate.value) params.set("start", startDate.value);
    if (endDate.value) params.set("end", endDate.value);
    const qs = params.toString();
    const url = `/admin/api/dashboard/stats${qs ? "?" + qs : ""}`;
    const res = await get<DashboardStats>(url);
    stats.value = res;
    await nextTick();
    renderCharts();
  } catch {
    // Dashboard stats failed to load
  }
}

function destroyCharts() {
  if (leadsChart) { leadsChart.destroy(); leadsChart = null; }
  if (funnelChart) { funnelChart.destroy(); funnelChart = null; }
  if (sourceChart) { sourceChart.destroy(); sourceChart = null; }
  if (typeChart) { typeChart.destroy(); typeChart = null; }
}

function renderCharts() {
  destroyCharts();

  // Leads Over Time (grouped bar chart)
  if (leadsChartCanvas.value) {
    const lot = stats.value.leads_over_time || [];
    const labels = lot.map((d) => d.period);
    leadsChart = new Chart(leadsChartCanvas.value, {
      type: "bar",
      data: {
        labels,
        datasets: [
          {
            label: "Total",
            data: lot.map((d) => d.total),
            backgroundColor: "#8979FF",
            borderRadius: 4,
            barThickness: 16,
          },
          {
            label: "Won",
            data: lot.map((d) => d.won),
            backgroundColor: "#63CFE5",
            borderRadius: 4,
            barThickness: 16,
          },
          {
            label: "Lost",
            data: lot.map((d) => d.lost),
            backgroundColor: "#FFA8A1",
            borderRadius: 4,
            barThickness: 16,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: { legend: { position: "bottom", labels: { usePointStyle: true, padding: 16 } } },
        scales: {
          y: { beginAtZero: true, ticks: { stepSize: 1 }, grid: { color: "rgba(0,0,0,0.06)" } },
          x: { grid: { display: false } },
        },
      },
    });
  }

  // Pipeline Funnel (horizontal bar chart)
  if (funnelChartCanvas.value) {
    const stages = stats.value.leads_by_stage || [];
    const labels = stages.map((s) => s.stage_name);
    const values = stages.map((s) => s.count);
    const colors = stages.map((_, i) => {
      const t = i / Math.max(stages.length - 1, 1);
      const r = Math.round(144 + (50 - 144) * t);
      const g = Math.round(247 + (204 - 247) * t);
      const b = Math.round(236 + (188 - 236) * t);
      return `rgba(${r},${g},${b},${0.8 + t * 0.2})`;
    });
    funnelChart = new Chart(funnelChartCanvas.value, {
      type: "bar",
      data: {
        labels,
        datasets: [
          {
            data: values,
            backgroundColor: colors,
            borderRadius: 4,
            barThickness: 28,
          },
        ],
      },
      options: {
        indexAxis: "y",
        responsive: true,
        maintainAspectRatio: false,
        plugins: { legend: { display: false }, tooltip: { enabled: true } },
        scales: {
          x: { beginAtZero: true, ticks: { stepSize: 1 }, grid: { color: "rgba(0,0,0,0.06)" } },
          y: { grid: { display: false } },
        },
      },
    });
  }

  // Revenue by Source (doughnut)
  if (sourceChartCanvas.value) {
    const sources = stats.value.revenue_by_source || [];
    if (sources.length) {
      sourceChart = new Chart(sourceChartCanvas.value, {
        type: "doughnut",
        data: {
          labels: sources.map((s) => s.source_name),
          datasets: [{ data: sources.map((s) => Number(s.total)), backgroundColor: doughnutColors.slice(0, sources.length), borderWidth: 2 }],
        },
        options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false } }, cutout: "60%" },
      });
    } else {
      sourceChart = new Chart(sourceChartCanvas.value, {
        type: "doughnut",
        data: { labels: ["No data"], datasets: [{ data: [1], backgroundColor: ["#E0E0E0"], borderWidth: 0 }] },
        options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false }, tooltip: { enabled: false } }, cutout: "60%" },
      });
    }
  }

  // Revenue by Type (doughnut)
  if (typeChartCanvas.value) {
    const types = stats.value.revenue_by_type || [];
    if (types.length) {
      typeChart = new Chart(typeChartCanvas.value, {
        type: "doughnut",
        data: {
          labels: types.map((t) => t.type_name),
          datasets: [{ data: types.map((t) => Number(t.total)), backgroundColor: doughnutColors.slice(0, types.length), borderWidth: 2 }],
        },
        options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false } }, cutout: "60%" },
      });
    } else {
      typeChart = new Chart(typeChartCanvas.value, {
        type: "doughnut",
        data: { labels: ["No data"], datasets: [{ data: [1], backgroundColor: ["#E0E0E0"], borderWidth: 0 }] },
        options: { responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false }, tooltip: { enabled: false } }, cutout: "60%" },
      });
    }
  }
}

onMounted(() => {
  loadStats();
});

onBeforeUnmount(() => {
  destroyCharts();
});
</script>

<style scoped>
.dashboard-layout {
  flex-wrap: wrap;
}
.dashboard-sidebar {
  width: 378px;
  min-width: 378px;
  max-width: 378px;
  flex-shrink: 0;
}
.revenue-bar {
  height: 24px;
  transition: width 0.3s ease;
}
@media (max-width: 960px) {
  .dashboard-sidebar {
    width: 100%;
    min-width: 100%;
    max-width: 100%;
  }
}
</style>
