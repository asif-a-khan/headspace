import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Group {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export const useGroupsStore = defineStore("admin-groups", () => {
  const groups = ref<Group[]>([]);
  const loading = ref(false);

  function hydrate(data: { groups?: Group[] }) {
    if (data.groups) groups.value = data.groups;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      groups.value = await get<Group[]>("/admin/api/settings/groups");
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<Group>("/admin/api/settings/groups", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<Group>(`/admin/api/settings/groups/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/groups/${id}`);
    groups.value = groups.value.filter((g) => g.id !== id);
  }

  return { groups, loading, hydrate, fetchAll, create, update, remove };
});
