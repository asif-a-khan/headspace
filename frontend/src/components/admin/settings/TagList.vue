<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Tags</h1>
      <v-spacer />
      <v-btn
        v-if="canCreate"
        color="primary"
        prepend-icon="mdi-plus"
        @click="openDialog()"
      >
        Create Tag
      </v-btn>
    </div>

    <v-data-table
      :headers="headers"
      :items="store.tags"
      :loading="store.loading"
      item-value="id"
    >
      <template #item.color="{ item }">
        <v-chip :color="item.color || '#6366F1'" size="small" variant="flat">
          <span class="text-white">{{ item.color || '#6366F1' }}</span>
        </v-chip>
      </template>
      <template #item.actions="{ item }">
        <v-btn
          v-if="canEdit"
          icon="mdi-pencil"
          size="small"
          variant="text"
          @click="openDialog(item)"
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

    <!-- Create/Edit dialog -->
    <v-dialog v-model="formDialog" max-width="400">
      <v-card>
        <v-card-title>{{ editingItem ? "Edit Tag" : "Create Tag" }}</v-card-title>
        <v-card-text>
          <v-text-field v-model="formName" label="Name" required autofocus class="mb-3" />
          <div class="d-flex align-center">
            <label class="text-body-2 mr-3">Color</label>
            <input type="color" v-model="formColor" style="width: 48px; height: 36px; border: none; cursor: pointer;" />
            <v-chip :color="formColor" size="small" variant="flat" class="ml-3">
              <span class="text-white">Preview</span>
            </v-chip>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="formDialog = false">Cancel</v-btn>
          <v-btn color="primary" @click="saveItem" :loading="saving">Save</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Tag</v-card-title>
        <v-card-text>Are you sure you want to delete "{{ deletingItem?.name }}"?</v-card-text>
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
import { useTagsStore, type Tag } from "@/stores/admin/tags";

const data = window.__INITIAL_DATA__ || {};
const permissions: string[] = data.permissions || [];
const canCreate = computed(() => permissions.includes("tags.create") || data.permission_type === "all");
const canEdit = computed(() => permissions.includes("tags.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("tags.delete") || data.permission_type === "all");

const store = useTagsStore();
store.hydrate(data);

const headers = [
  { title: "ID", key: "id", width: "80px" },
  { title: "Name", key: "name" },
  { title: "Color", key: "color", width: "140px" },
  { title: "Actions", key: "actions", sortable: false, width: "120px" },
];

// Form dialog
const formDialog = ref(false);
const formName = ref("");
const formColor = ref("#6366F1");
const editingItem = ref<Tag | null>(null);
const saving = ref(false);

function openDialog(item?: Tag) {
  editingItem.value = item || null;
  formName.value = item?.name || "";
  formColor.value = item?.color || "#6366F1";
  formDialog.value = true;
}

async function saveItem() {
  saving.value = true;
  try {
    if (editingItem.value) {
      await store.update(editingItem.value.id, { name: formName.value, color: formColor.value });
      // Update local list
      const idx = store.tags.findIndex((t) => t.id === editingItem.value!.id);
      if (idx >= 0) {
        store.tags[idx].name = formName.value;
        store.tags[idx].color = formColor.value;
      }
    } else {
      await store.create({ name: formName.value, color: formColor.value });
    }
    formDialog.value = false;
  } finally {
    saving.value = false;
  }
}

// Delete dialog
const deleteDialog = ref(false);
const deletingItem = ref<Tag | null>(null);
const deleting = ref(false);

function confirmDelete(item: Tag) {
  deletingItem.value = item;
  deleteDialog.value = true;
}

async function doDelete() {
  if (!deletingItem.value) return;
  deleting.value = true;
  try {
    await store.remove(deletingItem.value.id);
    deleteDialog.value = false;
  } finally {
    deleting.value = false;
  }
}
</script>
