<template>
  <div>
    <div class="text-subtitle-2 mb-1">Tags</div>
    <div class="d-flex flex-wrap ga-1 mb-2">
      <v-chip
        v-for="tag in selectedTags"
        :key="tag.id"
        :color="tag.color || '#6366F1'"
        closable
        size="small"
        @click:close="removeTag(tag)"
      >
        {{ tag.name }}
      </v-chip>
    </div>

    <v-autocomplete
      v-model="searchModel"
      :items="availableTags"
      item-title="name"
      item-value="id"
      label="Add tag..."
      density="compact"
      hide-details
      clearable
      return-object
      @update:model-value="addTag"
    >
      <template #item="{ item, props }">
        <v-list-item v-bind="props">
          <template #prepend>
            <v-icon :color="item.raw.color || '#6366F1'" size="small">mdi-circle</v-icon>
          </template>
        </v-list-item>
      </template>
      <template #append-inner>
        <v-btn
          icon="mdi-plus"
          size="x-small"
          variant="text"
          @click.stop="showCreateDialog = true"
        />
      </template>
    </v-autocomplete>

    <!-- Create new tag dialog -->
    <v-dialog v-model="showCreateDialog" max-width="360">
      <v-card>
        <v-card-title>Create Tag</v-card-title>
        <v-card-text>
          <v-text-field v-model="newTagName" label="Name" density="compact" class="mb-2" />
          <v-text-field v-model="newTagColor" label="Color" type="color" density="compact" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showCreateDialog = false">Cancel</v-btn>
          <v-btn color="primary" :loading="creating" @click="createAndAdd">Create</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useTagsStore, type Tag } from "@/stores/admin/tags";

const props = defineProps<{
  modelValue: Tag[];
  entityType: string;
  entityId?: number;
}>();

const emit = defineEmits<{
  "update:modelValue": [tags: Tag[]];
}>();

const store = useTagsStore();
if (!store.tags.length) {
  store.fetchAll();
}

const searchModel = ref<Tag | null>(null);

const selectedTags = computed(() => props.modelValue);

const availableTags = computed(() => {
  const selectedIds = new Set(props.modelValue.map((t) => t.id));
  return store.tags.filter((t) => !selectedIds.has(t.id));
});

async function addTag(tag: Tag | null) {
  if (!tag) return;
  const updated = [...props.modelValue, tag];
  emit("update:modelValue", updated);

  if (props.entityId) {
    await store.attach(props.entityType, props.entityId, tag.id);
  }
  searchModel.value = null;
}

async function removeTag(tag: Tag) {
  const updated = props.modelValue.filter((t) => t.id !== tag.id);
  emit("update:modelValue", updated);

  if (props.entityId) {
    await store.detach(props.entityType, props.entityId, tag.id);
  }
}

const showCreateDialog = ref(false);
const newTagName = ref("");
const newTagColor = ref("#6366F1");
const creating = ref(false);

async function createAndAdd() {
  if (!newTagName.value) return;
  creating.value = true;
  try {
    const tag = await store.create({ name: newTagName.value, color: newTagColor.value });
    const updated = [...props.modelValue, tag];
    emit("update:modelValue", updated);
    if (props.entityId) {
      await store.attach(props.entityType, props.entityId, tag.id);
    }
    showCreateDialog.value = false;
    newTagName.value = "";
    newTagColor.value = "#6366F1";
  } finally {
    creating.value = false;
  }
}
</script>
