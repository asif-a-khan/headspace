import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Tenant {
  id: number;
  name: string;
  email: string | null;
  domain: string;
  cname: string | null;
  description: string | null;
  is_active: boolean;
  schema_name: string;
  created_at: string;
  updated_at: string;
}

export interface TenantForm {
  name: string;
  email: string;
  domain: string;
  cname: string;
  description: string;
  is_active: boolean;
}

export const useTenantsStore = defineStore("super-tenants", () => {
  const tenants = ref<Tenant[]>([]);
  const loading = ref(false);

  function hydrate(data: { tenants?: Tenant[] }) {
    if (data.tenants) tenants.value = data.tenants;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      tenants.value = await get<Tenant[]>("/super/api/tenants");
    } finally {
      loading.value = false;
    }
  }

  async function create(form: TenantForm) {
    return post<Tenant>("/super/api/tenants", form);
  }

  async function update(id: number, form: TenantForm) {
    return put<Tenant>(`/super/api/tenants/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/super/api/tenants/${id}`);
    tenants.value = tenants.value.filter((t) => t.id !== id);
  }

  return { tenants, loading, hydrate, fetchAll, create, update, remove };
});
