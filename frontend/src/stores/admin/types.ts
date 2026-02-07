import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface LeadType {
  id: number;
  name: string;
  created_at: string;
  updated_at: string;
}

export const useTypesStore = defineStore("admin-types", () => {
  const types = ref<LeadType[]>([]);
  const loading = ref(false);

  function hydrate(data: { types?: LeadType[] }) {
    if (data.types) types.value = data.types;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: LeadType[] }>("/admin/api/settings/types");
      types.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    const res = await post<{ data: LeadType }>("/admin/api/settings/types", form);
    types.value.push(res.data);
    return res;
  }

  async function update(id: number, form: Record<string, unknown>) {
    const res = await put<{ data: LeadType }>(`/admin/api/settings/types/${id}`, form);
    const idx = types.value.findIndex((t) => t.id === id);
    if (idx !== -1) types.value[idx] = res.data;
    return res;
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/types/${id}`);
    types.value = types.value.filter((t) => t.id !== id);
  }

  return { types, loading, hydrate, fetchAll, create, update, remove };
});
