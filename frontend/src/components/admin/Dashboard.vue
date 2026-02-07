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

    <!-- Stat Cards Row 1 -->
    <v-row class="mb-2">
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
                  ${{ fmtNum(stats.won_deals.value) }}
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
              <v-avatar color="error" size="40" class="mr-3">
                <v-icon color="white">mdi-close-circle</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">{{ stats.lost_deals?.count || 0 }}</p>
                <p class="text-body-2 text-secondary">Lost Deals</p>
                <p class="text-caption text-error" v-if="stats.lost_deals?.value">
                  ${{ fmtNum(stats.lost_deals.value) }}
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
                  ${{ stats.open_revenue ? fmtNum(stats.open_revenue) : '0' }}
                </p>
                <p class="text-body-2 text-secondary">Open Revenue</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Stat Cards Row 2 -->
    <v-row class="mb-4">
      <v-col cols="12" sm="6" md="3">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center">
              <v-avatar color="indigo" size="40" class="mr-3">
                <v-icon color="white">mdi-chart-line</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">
                  ${{ stats.avg_lead_value ? fmtNum(stats.avg_lead_value) : '0' }}
                </p>
                <p class="text-body-2 text-secondary">Avg Lead Value</p>
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
      <v-col cols="12" sm="6" md="3">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center">
              <v-avatar color="teal" size="40" class="mr-3">
                <v-icon color="white">mdi-file-document-outline</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">{{ stats.total_quotes }}</p>
                <p class="text-body-2 text-secondary">Total Quotes</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" sm="6" md="3">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center">
              <v-avatar color="deep-purple" size="40" class="mr-3">
                <v-icon color="white">mdi-account-multiple</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">{{ stats.total_persons }}</p>
                <p class="text-body-2 text-secondary">Total Persons</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Charts Row 1: Leads Over Time + Pipeline Funnel -->
    <v-row class="mb-4">
      <v-col cols="12" md="7">
        <v-card rounded="lg" elevation="0">
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Leads Over Time
          </v-card-title>
          <v-card-text class="pa-4">
            <div v-if="stats.leads_over_time?.length" style="height: 280px">
              <canvas ref="leadsChartCanvas"></canvas>
            </div>
            <div v-else class="text-center text-medium-emphasis pa-8">
              No lead data for this period
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" md="5">
        <v-card rounded="lg" elevation="0">
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Pipeline Funnel
          </v-card-title>
          <v-card-text class="pa-4">
            <div v-if="stats.leads_by_stage?.length" style="height: 280px">
              <canvas ref="funnelChartCanvas"></canvas>
            </div>
            <div v-else class="text-center text-medium-emphasis pa-8">
              No open leads yet
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Charts Row 2: Revenue by Source + Revenue by Type -->
    <v-row class="mb-4">
      <v-col cols="12" md="6">
        <v-card rounded="lg" elevation="0">
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Revenue by Source
          </v-card-title>
          <v-card-text class="pa-4">
            <div v-if="stats.revenue_by_source?.length" class="d-flex align-center justify-center" style="height: 260px">
              <div style="width: 220px; height: 220px">
                <canvas ref="sourceChartCanvas"></canvas>
              </div>
              <div class="ml-4">
                <div
                  v-for="(src, i) in stats.revenue_by_source"
                  :key="src.source_name"
                  class="d-flex align-center mb-2"
                >
                  <span
                    class="d-inline-block rounded-circle mr-2"
                    :style="{ width: '10px', height: '10px', backgroundColor: doughnutColors[i % doughnutColors.length] }"
                  />
                  <span class="text-body-2 mr-2">{{ src.source_name }}</span>
                  <span class="text-body-2 font-weight-medium">${{ fmtNum(src.total) }}</span>
                </div>
              </div>
            </div>
            <div v-else class="text-center text-medium-emphasis pa-8">
              No won deals yet
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" md="6">
        <v-card rounded="lg" elevation="0">
          <v-card-title class="text-subtitle-1 font-weight-medium pa-4 pb-0">
            Revenue by Type
          </v-card-title>
          <v-card-text class="pa-4">
            <div v-if="stats.revenue_by_type?.length" class="d-flex align-center justify-center" style="height: 260px">
              <div style="width: 220px; height: 220px">
                <canvas ref="typeChartCanvas"></canvas>
              </div>
              <div class="ml-4">
                <div
                  v-for="(t, i) in stats.revenue_by_type"
                  :key="t.type_name"
                  class="d-flex align-center mb-2"
                >
                  <span
                    class="d-inline-block rounded-circle mr-2"
                    :style="{ width: '10px', height: '10px', backgroundColor: doughnutColors[i % doughnutColors.length] }"
                  />
                  <span class="text-body-2 mr-2">{{ t.type_name }}</span>
                  <span class="text-body-2 font-weight-medium">${{ fmtNum(t.total) }}</span>
                </div>
              </div>
            </div>
            <div v-else class="text-center text-medium-emphasis pa-8">
              No won deals yet
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Bottom Row: Top Products + Top Persons -->
    <v-row>
      <v-col cols="12" md="6">
        <v-card rounded="lg" elevation="0">
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
        <v-card rounded="lg" elevation="0">
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
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick, onBeforeUnmount } from "vue";
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

// Date filter
const startDate = ref("");
const endDate = ref("");

interface DashboardStats {
  total_leads: number;
  open_leads: number;
  won_deals: { count: number; value: string | null };
  lost_deals: { count: number; value: string | null };
  open_revenue: string | null;
  avg_lead_value: string | null;
  total_quotes: number;
  total_persons: number;
  total_organizations: number;
  activities_due: number;
  leads_by_stage: Array<{ stage_name: string; count: number }>;
  revenue_by_source: Array<{ source_name: string; total: string }>;
  revenue_by_type: Array<{ type_name: string; total: string }>;
  leads_over_time: Array<{ period: string; total: number; won: number; lost: number }>;
  top_products: Array<{ name: string; sku: string; total_revenue: string; total_qty: number }>;
  top_persons: Array<{ id: number; name: string; email: string | null; total_leads: number }>;
}

const stats = ref<DashboardStats>({
  total_leads: 0,
  open_leads: 0,
  won_deals: { count: 0, value: null },
  lost_deals: { count: 0, value: null },
  open_revenue: null,
  avg_lead_value: null,
  total_quotes: 0,
  total_persons: 0,
  total_organizations: 0,
  activities_due: 0,
  leads_by_stage: [],
  revenue_by_source: [],
  revenue_by_type: [],
  leads_over_time: [],
  top_products: [],
  top_persons: [],
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

function fmtNum(val: string | number | null): string {
  if (val == null) return "0";
  return Number(val).toLocaleString();
}

function resetDates() {
  startDate.value = "";
  endDate.value = "";
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
  if (leadsChartCanvas.value && stats.value.leads_over_time?.length) {
    const labels = stats.value.leads_over_time.map((d) => d.period);
    leadsChart = new Chart(leadsChartCanvas.value, {
      type: "bar",
      data: {
        labels,
        datasets: [
          {
            label: "Total",
            data: stats.value.leads_over_time.map((d) => d.total),
            backgroundColor: "#8979FF",
            borderRadius: 4,
            barThickness: 16,
          },
          {
            label: "Won",
            data: stats.value.leads_over_time.map((d) => d.won),
            backgroundColor: "#3CC3DF",
            borderRadius: 4,
            barThickness: 16,
          },
          {
            label: "Lost",
            data: stats.value.leads_over_time.map((d) => d.lost),
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
  if (funnelChartCanvas.value && stats.value.leads_by_stage?.length) {
    const labels = stats.value.leads_by_stage.map((s) => s.stage_name);
    const values = stats.value.leads_by_stage.map((s) => s.count);
    const colors = stats.value.leads_by_stage.map((_, i) => {
      const t = i / Math.max(stats.value.leads_by_stage.length - 1, 1);
      return `hsl(${170 + t * 40}, 60%, ${55 - t * 10}%)`;
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
  if (sourceChartCanvas.value && stats.value.revenue_by_source?.length) {
    sourceChart = new Chart(sourceChartCanvas.value, {
      type: "doughnut",
      data: {
        labels: stats.value.revenue_by_source.map((s) => s.source_name),
        datasets: [
          {
            data: stats.value.revenue_by_source.map((s) => Number(s.total)),
            backgroundColor: doughnutColors.slice(0, stats.value.revenue_by_source.length),
            borderWidth: 2,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: { legend: { display: false } },
        cutout: "60%",
      },
    });
  }

  // Revenue by Type (doughnut)
  if (typeChartCanvas.value && stats.value.revenue_by_type?.length) {
    typeChart = new Chart(typeChartCanvas.value, {
      type: "doughnut",
      data: {
        labels: stats.value.revenue_by_type.map((t) => t.type_name),
        datasets: [
          {
            data: stats.value.revenue_by_type.map((t) => Number(t.total)),
            backgroundColor: doughnutColors.slice(0, stats.value.revenue_by_type.length),
            borderWidth: 2,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: { legend: { display: false } },
        cutout: "60%",
      },
    });
  }
}

onMounted(() => {
  loadStats();
});

onBeforeUnmount(() => {
  destroyCharts();
});
</script>
