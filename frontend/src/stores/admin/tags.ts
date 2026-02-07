import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Tag {
  id: number;
  name: string;
  color: string | null;
  user_id: number;
  created_at: string;
  updated_at: string;
}

export const useTagsStore = defineStore("admin-tags", () => {
  const tags = ref<Tag[]>([]);
  const loading = ref(false);

  function hydrate(data: { tags?: Tag[] }) {
    if (data.tags) tags.value = data.tags;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Tag[] }>("/admin/api/tags");
      tags.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: { name: string; color?: string }) {
    const res = await post<{ data: Tag }>("/admin/api/tags", form);
    tags.value.push(res.data);
    return res.data;
  }

  async function update(id: number, form: { name: string; color?: string }) {
    return put<{ data: Tag }>(`/admin/api/tags/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/tags/${id}`);
    tags.value = tags.value.filter((t) => t.id !== id);
  }

  async function attach(entityType: string, entityId: number, tagId: number) {
    await post("/admin/api/tags/attach", { entity_type: entityType, entity_id: entityId, tag_id: tagId });
  }

  async function detach(entityType: string, entityId: number, tagId: number) {
    await post("/admin/api/tags/detach", { entity_type: entityType, entity_id: entityId, tag_id: tagId });
  }

  return { tags, loading, hydrate, fetchAll, create, update, remove, attach, detach };
});
