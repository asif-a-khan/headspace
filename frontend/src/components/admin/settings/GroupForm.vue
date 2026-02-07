<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Group" : "Create Group" }}</h1>
    <v-card max-width="700">
      <v-card-text>
        <v-alert v-if="error" type="error" density="compact" class="mb-4">
          {{ error }}
        </v-alert>
        <v-form @submit.prevent="submit">
          <v-text-field v-model="form.name" label="Name" required />
          <v-textarea v-model="form.description" label="Description" rows="2" class="mt-2" />
          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="loading">
              {{ isEdit ? "Update" : "Create" }}
            </v-btn>
            <v-btn href="/admin/settings/groups" variant="text">Cancel</v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { useGroupsStore } from "@/stores/admin/groups";

const data = window.__INITIAL_DATA__ || {};

const store = useGroupsStore();
store.hydrate(data);
const isEdit = !!data.group;
const error = ref("");
const loading = ref(false);

const form = reactive({
  name: data.group?.name || "",
  description: data.group?.description || "",
});

async function submit() {
  error.value = "";
  loading.value = true;
  try {
    if (isEdit) {
      await store.update(data.group.id, form);
    } else {
      await store.create(form);
    }
    window.location.href = "/admin/settings/groups";
  } catch (e: any) {
    error.value = e.message || "Failed to save";
  } finally {
    loading.value = false;
  }
}
</script>
