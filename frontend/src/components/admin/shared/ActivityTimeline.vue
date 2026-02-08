<template>
  <v-card variant="outlined" class="activity-timeline">
    <!-- Tab bar -->
    <div class="activity-tabs border-b px-2">
      <v-tabs v-model="activeTab" density="compact" show-arrows>
        <v-tab v-for="tab in allTabs" :key="tab.name" :value="tab.name" class="text-none">
          {{ tab.label }}
          <v-badge
            v-if="tab.name !== 'all' && getTabCount(tab.name) > 0"
            :content="getTabCount(tab.name)"
            color="primary"
            inline
            class="ml-1"
          />
        </v-tab>
      </v-tabs>
    </div>

    <!-- Extra tab slots (Description, Products, Quotes) -->
    <template v-if="isExtraTab">
      <slot :name="activeTab" />
    </template>

    <!-- Activity list -->
    <template v-else>
      <div class="activity-list pa-4" style="max-height: 600px; overflow-y: auto">
        <template v-if="filteredActivities.length">
          <div
            v-for="activity in filteredActivities"
            :key="activity.id"
            class="d-flex ga-3 mb-4"
          >
            <!-- Color-coded icon -->
            <v-avatar
              :color="getActivityColor(activity.type)"
              size="36"
              class="flex-shrink-0 mt-1"
            >
              <v-icon size="18" :color="getIconTextColor(activity.type)">
                {{ getActivityIcon(activity.type) }}
              </v-icon>
            </v-avatar>

            <!-- Content -->
            <div class="flex-grow-1" style="min-width: 0">
              <div class="d-flex align-center ga-2 mb-1">
                <span class="text-body-2 font-weight-medium">
                  {{ activity.title || activityTypeLabel(activity.type) }}
                </span>
                <v-chip
                  :color="getActivityColor(activity.type)"
                  size="x-small"
                  variant="tonal"
                >
                  {{ activity.type }}
                </v-chip>
                <v-spacer />
                <v-checkbox-btn
                  v-if="activity.schedule_from"
                  :model-value="activity.is_done"
                  density="compact"
                  hide-details
                  class="flex-shrink-0"
                  disabled
                />
              </div>

              <div v-if="activity.comment" class="text-body-2 text-medium-emphasis mb-1" style="white-space: pre-line">
                {{ activity.comment }}
              </div>

              <div class="d-flex align-center ga-2 text-caption text-medium-emphasis">
                <span v-if="activity.schedule_from">
                  <v-icon size="x-small" class="mr-1">mdi-clock-outline</v-icon>
                  {{ formatDateTime(activity.schedule_from) }}
                  <span v-if="activity.schedule_to"> — {{ formatDateTime(activity.schedule_to) }}</span>
                </span>
                <span v-else>
                  {{ formatDateTime(activity.created_at) }}
                </span>
                <span v-if="activity.user_name">
                  <v-icon size="x-small" class="mr-1">mdi-account</v-icon>
                  {{ activity.user_name }}
                </span>
                <span v-if="activity.location">
                  <v-icon size="x-small" class="mr-1">mdi-map-marker</v-icon>
                  {{ activity.location }}
                </span>
              </div>
            </div>
          </div>
        </template>

        <!-- Empty state -->
        <div v-else class="text-center py-12">
          <v-icon size="64" color="grey-lighten-1" class="mb-3">
            {{ emptyIcon }}
          </v-icon>
          <div class="text-body-1 text-medium-emphasis">{{ emptyTitle }}</div>
          <div class="text-body-2 text-medium-emphasis">{{ emptyDescription }}</div>
        </div>
      </div>
    </template>
  </v-card>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { getActivityColor, getActivityIcon } from "@/utils/activityColors";
import type { Activity } from "@/stores/admin/activities";

interface ExtraTab {
  name: string;
  label: string;
}

const props = defineProps<{
  activities: Activity[];
  extraTabs?: ExtraTab[];
}>();

const standardTabs = [
  { name: "all", label: "All" },
  { name: "planned", label: "Planned" },
  { name: "note", label: "Notes" },
  { name: "call", label: "Calls" },
  { name: "meeting", label: "Meetings" },
  { name: "file", label: "Files" },
  { name: "email", label: "Emails" },
];

const allTabs = computed(() => [
  ...standardTabs,
  ...(props.extraTabs || []),
]);

const activeTab = ref("all");

const extraTabNames = computed(() => (props.extraTabs || []).map((t) => t.name));
const isExtraTab = computed(() => extraTabNames.value.includes(activeTab.value));

const filteredActivities = computed(() => {
  if (isExtraTab.value) return [];
  const tab = activeTab.value;
  if (tab === "all") return props.activities;
  if (tab === "planned") {
    return props.activities.filter((a) => !a.is_done && a.schedule_from);
  }
  return props.activities.filter((a) => a.type === tab);
});

function getTabCount(tab: string): number {
  if (extraTabNames.value.includes(tab)) return 0;
  if (tab === "all") return props.activities.length;
  if (tab === "planned") {
    return props.activities.filter((a) => !a.is_done && a.schedule_from).length;
  }
  return props.activities.filter((a) => a.type === tab).length;
}

function getIconTextColor(type: string): string {
  const darkTextTypes: Record<string, string> = {
    email: "#1B5E20",
    note: "#E65100",
    call: "#006064",
    meeting: "#0D47A1",
    lunch: "#0D47A1",
    file: "#1B5E20",
    system: "#F57F17",
    task: "#1B5E20",
  };
  return darkTextTypes[type] || "#424242";
}

function activityTypeLabel(type: string): string {
  return type.charAt(0).toUpperCase() + type.slice(1);
}

function formatDateTime(dt: string): string {
  if (!dt) return "";
  const d = new Date(dt);
  return d.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

const emptyMessages: Record<string, { icon: string; title: string; description: string }> = {
  all: { icon: "mdi-calendar-blank-outline", title: "No activities yet", description: "Activities will appear here once created" },
  planned: { icon: "mdi-calendar-check-outline", title: "No planned activities", description: "Scheduled activities will appear here" },
  note: { icon: "mdi-note-text-outline", title: "No notes", description: "Notes will appear here" },
  call: { icon: "mdi-phone-outline", title: "No calls", description: "Call activities will appear here" },
  meeting: { icon: "mdi-calendar-clock", title: "No meetings", description: "Meeting activities will appear here" },
  file: { icon: "mdi-file-outline", title: "No files", description: "File attachments will appear here" },
  email: { icon: "mdi-email-outline", title: "No emails", description: "Emails will appear here" },
};

const emptyIcon = computed(() => emptyMessages[activeTab.value]?.icon || "mdi-calendar-blank-outline");
const emptyTitle = computed(() => emptyMessages[activeTab.value]?.title || "Nothing here");
const emptyDescription = computed(() => emptyMessages[activeTab.value]?.description || "");
</script>

<style scoped>
.activity-tabs :deep(.v-tab) {
  min-width: auto;
  font-size: 0.8125rem;
}
.activity-list > div:nth-child(even) {
  background-color: rgba(var(--v-theme-on-surface), 0.03);
  border-radius: 8px;
  padding: 8px;
}
</style>
