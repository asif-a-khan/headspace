import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Person {
  id: number;
  name: string;
  emails: Array<{ value: string; label: string }>;
  contact_numbers: Array<{ value: string; label: string }>;
  job_title: string | null;
  organization_id: number | null;
  user_id: number | null;
  created_at: string;
  updated_at: string;
  organization_name?: string | null;
  user_name?: string | null;
}

export const usePersonsStore = defineStore("admin-persons", () => {
  const persons = ref<Person[]>([]);
  const loading = ref(false);

  function hydrate(data: { persons?: Person[] }) {
    if (data.persons) persons.value = data.persons;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Person[] }>("/admin/api/contacts/persons");
      persons.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Person }>("/admin/api/contacts/persons", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Person }>(`/admin/api/contacts/persons/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/contacts/persons/${id}`);
    persons.value = persons.value.filter((p) => p.id !== id);
  }

  return { persons, loading, hydrate, fetchAll, create, update, remove };
});
