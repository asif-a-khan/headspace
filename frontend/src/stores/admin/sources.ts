import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Source {
  id: number;
  name: string;
  created_at: string;
  updated_at: string;
}

export const useSourcesStore = defineStore("admin-sources", () => {
  const sources = ref<Source[]>([]);
  const loading = ref(false);

  function hydrate(data: { sources?: Source[] }) {
    if (data.sources) sources.value = data.sources;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Source[] }>("/admin/api/settings/sources");
      sources.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    const res = await post<{ data: Source }>("/admin/api/settings/sources", form);
    sources.value.push(res.data);
    return res;
  }

  async function update(id: number, form: Record<string, unknown>) {
    const res = await put<{ data: Source }>(`/admin/api/settings/sources/${id}`, form);
    const idx = sources.value.findIndex((s) => s.id === id);
    if (idx !== -1) sources.value[idx] = res.data;
    return res;
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/sources/${id}`);
    sources.value = sources.value.filter((s) => s.id !== id);
  }

  return { sources, loading, hydrate, fetchAll, create, update, remove };
});
