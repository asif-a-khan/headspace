import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Organization {
  id: number;
  name: string;
  address: Record<string, string> | null;
  user_id: number | null;
  created_at: string;
  updated_at: string;
  user_name?: string | null;
}

export const useOrganizationsStore = defineStore("admin-organizations", () => {
  const organizations = ref<Organization[]>([]);
  const loading = ref(false);

  function hydrate(data: { organizations?: Organization[] }) {
    if (data.organizations) organizations.value = data.organizations;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Organization[] }>("/admin/api/contacts/organizations");
      organizations.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Organization }>("/admin/api/contacts/organizations", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Organization }>(`/admin/api/contacts/organizations/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/contacts/organizations/${id}`);
    organizations.value = organizations.value.filter((o) => o.id !== id);
  }

  return { organizations, loading, hydrate, fetchAll, create, update, remove };
});
