import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Warehouse {
  id: number;
  name: string;
  description: string | null;
  contact_name: string | null;
  contact_emails: any[];
  contact_numbers: any[];
  contact_address: Record<string, string> | null;
  created_at: string;
  updated_at: string;
}

export const useWarehousesStore = defineStore("admin-warehouses", () => {
  const warehouses = ref<Warehouse[]>([]);
  const loading = ref(false);

  function hydrate(data: { warehouses?: Warehouse[] }) {
    if (data.warehouses) warehouses.value = data.warehouses;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Warehouse[] }>("/admin/api/settings/warehouses");
      warehouses.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Warehouse }>("/admin/api/settings/warehouses", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Warehouse }>(`/admin/api/settings/warehouses/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/warehouses/${id}`);
    warehouses.value = warehouses.value.filter((w) => w.id !== id);
  }

  return { warehouses, loading, hydrate, fetchAll, create, update, remove };
});
