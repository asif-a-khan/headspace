import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Role {
  id: number;
  name: string;
  description: string | null;
  permission_type: string;
  permissions: string[];
  created_at: string;
  updated_at: string;
}

export const useRolesStore = defineStore("admin-roles", () => {
  const roles = ref<Role[]>([]);
  const loading = ref(false);

  function hydrate(data: { roles?: Role[] }) {
    if (data.roles) roles.value = data.roles;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      roles.value = await get<Role[]>("/admin/api/settings/roles");
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<Role>("/admin/api/settings/roles", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<Role>(`/admin/api/settings/roles/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/roles/${id}`);
    roles.value = roles.value.filter((r) => r.id !== id);
  }

  return { roles, loading, hydrate, fetchAll, create, update, remove };
});
