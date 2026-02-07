<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Activities</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        href="/admin/activities/create"
      >
        Create Activity
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.activities"
      :loading="store.loading"
      item-value="id"
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useActivitiesStore, type Activity } from "@/stores/admin/activities";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("activities.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("activities.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("activities.delete") || data.permission_type === "all");

const store = useActivitiesStore();
store.hydrate(data);

function typeColor(type: string): string {
  switch (type) {
    case "call": return "blue";
    case "meeting": return "purple";
    case "note": return "orange";
    case "task": return "green";
    default: return "grey";
  }
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
</script>
