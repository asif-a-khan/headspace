import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface User {
  id: number;
  first_name: string;
  last_name: string;
  email: string;
  image: string | null;
  status: boolean;
  role_id: number;
  view_permission: string;
  created_at: string;
  updated_at: string;
}

export const useUsersStore = defineStore("admin-users", () => {
  const users = ref<User[]>([]);
  const loading = ref(false);

  function hydrate(data: { users?: User[] }) {
    if (data.users) users.value = data.users;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      users.value = await get<User[]>("/admin/api/settings/users");
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<User>("/admin/api/settings/users", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<User>(`/admin/api/settings/users/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/users/${id}`);
    users.value = users.value.filter((u) => u.id !== id);
  }

  return { users, loading, hydrate, fetchAll, create, update, remove };
});
