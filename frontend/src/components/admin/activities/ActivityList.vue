<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5 font-weight-bold">Activities</h1>
      <v-spacer />
      <v-btn-toggle v-model="viewMode" mandatory density="compact" class="mr-3">
        <v-btn value="table" size="small">
          <v-icon>mdi-table</v-icon>
        </v-btn>
        <v-btn value="calendar" size="small">
          <v-icon>mdi-calendar-month</v-icon>
        </v-btn>
      </v-btn-toggle>
      <v-btn
        v-if="canDelete && selectedIds.length"
        color="error"
        variant="outlined"
        class="mr-2"
        prepend-icon="mdi-delete-sweep"
        @click="massDeleteDialog = true"
      >
        Delete ({{ selectedIds.length }})
      </v-btn>
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/activities/create"
      >
        Create Activity
      </v-btn>
    </div>

    <!-- Table View -->
    <v-data-table
      v-if="viewMode === 'table'"
      v-model="selectedIds"
      :headers="headers"
      :items="store.activities"
      :loading="store.loading"
      item-value="id"
      :show-select="canDelete"
    >
      <template #item.type="{ item }">
        <v-chip :color="typeColor(item.type)" size="small">{{ item.type }}</v-chip>
      </template>
      <template #item.is_done="{ item }">
        <v-icon :color="item.is_done ? 'success' : 'grey'" size="small">
          {{ item.is_done ? 'mdi-check-circle' : 'mdi-circle-outline' }}
        </v-icon>
      </template>
      <template #item.schedule_from="{ item }">
        {{ item.schedule_from ? new Date(item.schedule_from).toLocaleString() : '-' }}
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          :href="`/admin/activities/${item.id}/edit`"
        />
        <v-btn
          v-if="canDelete"
          icon="mdi-delete"
          size="small"
          variant="text"
          color="error"
          @click="confirmDelete(item)"
        />
      </template>
    </v-data-table>

    <!-- Calendar View -->
    <v-card v-if="viewMode === 'calendar'" rounded="lg" elevation="0">
      <v-card-text class="pa-4">
        <!-- Month Navigation -->
        <div class="d-flex align-center justify-center mb-4">
          <v-btn icon variant="text" @click="prevMonth">
            <v-icon>mdi-chevron-left</v-icon>
          </v-btn>
          <span class="text-h6 font-weight-medium mx-4" style="min-width: 180px; text-align: center">
            {{ monthLabel }}
          </span>
          <v-btn icon variant="text" @click="nextMonth">
            <v-icon>mdi-chevron-right</v-icon>
          </v-btn>
          <v-btn size="small" variant="tonal" class="ml-2" @click="goToToday">Today</v-btn>
        </div>

        <!-- Weekday Headers -->
        <div class="calendar-grid mb-1">
          <div
            v-for="day in weekDays"
            :key="day"
            class="calendar-header text-caption font-weight-medium text-center text-medium-emphasis"
          >
            {{ day }}
          </div>
        </div>

        <!-- Calendar Cells -->
        <div class="calendar-grid">
          <div
            v-for="(cell, i) in calendarCells"
            :key="i"
            class="calendar-cell rounded"
            :class="{
              'bg-surface': cell.inMonth,
              'text-disabled': !cell.inMonth,
              'today-cell': cell.isToday,
            }"
          >
            <div class="text-caption font-weight-medium pa-1">{{ cell.day }}</div>
            <div class="cell-activities">
              <div
                v-for="act in cell.activities"
                :key="act.id"
                class="activity-chip text-caption px-1 mb-1 rounded"
                :class="`bg-${typeColor(act.type)}`"
                :title="act.title || act.type"
                @click="goToActivity(act)"
              >
                <span class="text-white text-truncate d-block" style="font-size: 11px">
                  {{ formatTime(act.schedule_from) }} {{ act.title || act.type }}
                </span>
              </div>
              <div
                v-if="cell.overflow > 0"
                class="text-caption text-primary cursor-pointer px-1"
              >
                +{{ cell.overflow }} more
              </div>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>

    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Activity</v-card-title>
        <v-card-text>
          Are you sure you want to delete this activity?
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" @click="doDelete" :loading="deleting">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="massDeleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Selected Activities</v-card-title>
        <v-card-text>
          Are you sure you want to delete {{ selectedIds.length }} selected activity(ies)? This action cannot be undone.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="massDeleteDialog = false">Cancel</v-btn>
          <v-btn color="error" @click="doMassDelete" :loading="massDeleting">Delete All</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useActivitiesStore, type Activity } from "@/stores/admin/activities";
import { post } from "@/api/client";
import { activityTypeColors } from "@/utils/activityColors";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("activities.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("activities.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("activities.delete") || data.permission_type === "all");

const store = useActivitiesStore();
store.hydrate(data);

const viewMode = ref("table");

function typeColor(type: string): string {
  return activityTypeColors[type] || "#E0E0E0";
}

const headers = [
  { title: "ID", key: "id", width: "70px" },
  { title: "Title", key: "title" },
  { title: "Type", key: "type", width: "100px" },
  { title: "Done", key: "is_done", width: "60px" },
  { title: "Scheduled", key: "schedule_from" },
  { title: "Created By", key: "user_name" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

// --- Calendar Logic ---
const today = new Date();
const calMonth = ref(today.getMonth());
const calYear = ref(today.getFullYear());
const weekDays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MAX_PER_CELL = 3;

const monthLabel = computed(() => {
  const d = new Date(calYear.value, calMonth.value, 1);
  return d.toLocaleDateString("en-US", { month: "long", year: "numeric" });
});

function prevMonth() {
  if (calMonth.value === 0) {
    calMonth.value = 11;
    calYear.value--;
  } else {
    calMonth.value--;
  }
}

function nextMonth() {
  if (calMonth.value === 11) {
    calMonth.value = 0;
    calYear.value++;
  } else {
    calMonth.value++;
  }
}

function goToToday() {
  calMonth.value = today.getMonth();
  calYear.value = today.getFullYear();
}

interface CalendarCell {
  day: number;
  inMonth: boolean;
  isToday: boolean;
  dateStr: string;
  activities: Activity[];
  overflow: number;
}

const calendarCells = computed((): CalendarCell[] => {
  const year = calYear.value;
  const month = calMonth.value;
  const firstDay = new Date(year, month, 1).getDay();
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const daysInPrevMonth = new Date(year, month, 0).getDate();

  // Build activity lookup by date string "YYYY-MM-DD"
  const actByDate: Record<string, Activity[]> = {};
  for (const act of store.activities) {
    if (!act.schedule_from) continue;
    const d = act.schedule_from.substring(0, 10); // "YYYY-MM-DD"
    if (!actByDate[d]) actByDate[d] = [];
    actByDate[d].push(act);
  }

  const cells: CalendarCell[] = [];

  // Previous month padding
  for (let i = firstDay - 1; i >= 0; i--) {
    const day = daysInPrevMonth - i;
    const m = month === 0 ? 11 : month - 1;
    const y = month === 0 ? year - 1 : year;
    const dateStr = `${y}-${String(m + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    const all = actByDate[dateStr] || [];
    cells.push({
      day,
      inMonth: false,
      isToday: false,
      dateStr,
      activities: all.slice(0, MAX_PER_CELL),
      overflow: Math.max(0, all.length - MAX_PER_CELL),
    });
  }

  // Current month
  const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, "0")}-${String(today.getDate()).padStart(2, "0")}`;
  for (let d = 1; d <= daysInMonth; d++) {
    const dateStr = `${year}-${String(month + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    const all = actByDate[dateStr] || [];
    cells.push({
      day: d,
      inMonth: true,
      isToday: dateStr === todayStr,
      dateStr,
      activities: all.slice(0, MAX_PER_CELL),
      overflow: Math.max(0, all.length - MAX_PER_CELL),
    });
  }

  // Next month padding (fill to 6 rows = 42 cells)
  const remaining = 42 - cells.length;
  for (let d = 1; d <= remaining; d++) {
    const m = month === 11 ? 0 : month + 1;
    const y = month === 11 ? year + 1 : year;
    const dateStr = `${y}-${String(m + 1).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
    const all = actByDate[dateStr] || [];
    cells.push({
      day: d,
      inMonth: false,
      isToday: false,
      dateStr,
      activities: all.slice(0, MAX_PER_CELL),
      overflow: Math.max(0, all.length - MAX_PER_CELL),
    });
  }

  return cells;
});

function formatTime(dt: string | null): string {
  if (!dt) return "";
  const d = new Date(dt);
  return d.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit", hour12: true });
}

function goToActivity(act: Activity) {
  if (canEdit.value) {
    window.location.href = `/admin/activities/${act.id}/edit`;
  }
}

// --- Delete Logic ---
const deleteDialog = ref(false);
const deletingActivity = ref<Activity | null>(null);
const deleting = ref(false);

function confirmDelete(activity: Activity) {
  deletingActivity.value = activity;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingActivity.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingActivity.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}

const selectedIds = ref<number[]>([]);
const massDeleteDialog = ref(false);
const massDeleting = ref(false);

async function doMassDelete() {
  if (!selectedIds.value.length) return;
  massDeleting.value = true;
  try {
    await post("/admin/api/activities/mass-delete", { ids: selectedIds.value });
    massDeleteDialog.value = false;
    selectedIds.value = [];
    store.fetchAll();
  } finally {
    massDeleting.value = false;
  }
}
</script>

<style scoped>
.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}
.calendar-header {
  padding: 4px 0;
}
.calendar-cell {
  min-height: 90px;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
  overflow: hidden;
}
.today-cell {
  border-color: rgb(var(--v-theme-primary)) !important;
  border-width: 2px !important;
}
.activity-chip {
  cursor: pointer;
  line-height: 1.3;
  opacity: 0.9;
}
.activity-chip:hover {
  opacity: 1;
}
</style>
