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

export interface RoleForm {
  name: string;
  description: string;
  permission_type: string;
  permissions: string[];
}

export interface AclNode {
  key: string;
  name: string;
  depth: number;
}

export const useRolesStore = defineStore("super-roles", () => {
  const roles = ref<Role[]>([]);
  const acl = ref<AclNode[]>([]);
  const loading = ref(false);

  function hydrate(data: { roles?: Role[]; acl?: AclNode[] }) {
    if (data.roles) roles.value = data.roles;
    if (data.acl) acl.value = data.acl;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      roles.value = await get<Role[]>("/super/api/roles");
    } finally {
      loading.value = false;
    }
  }

  async function create(form: RoleForm) {
    return post<Role>("/super/api/roles", form);
  }

  async function update(id: number, form: RoleForm) {
    return put<Role>(`/super/api/roles/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/super/api/roles/${id}`);
    roles.value = roles.value.filter((r) => r.id !== id);
  }

  return { roles, acl, loading, hydrate, fetchAll, create, update, remove };
});
