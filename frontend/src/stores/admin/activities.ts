import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Activity {
  id: number;
  title: string | null;
  type: string;
  comment: string | null;
  additional: Record<string, unknown> | null;
  location: string | null;
  schedule_from: string | null;
  schedule_to: string | null;
  is_done: boolean;
  user_id: number;
  created_at: string;
  updated_at: string;
  user_name?: string | null;
}

export const useActivitiesStore = defineStore("admin-activities", () => {
  const activities = ref<Activity[]>([]);
  const loading = ref(false);

  function hydrate(data: { activities?: Activity[] }) {
    if (data.activities) activities.value = data.activities;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Activity[] }>("/admin/api/activities");
      activities.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Activity }>("/admin/api/activities", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Activity }>(`/admin/api/activities/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/activities/${id}`);
    activities.value = activities.value.filter((a) => a.id !== id);
  }

  return { activities, loading, hydrate, fetchAll, create, update, remove };
});
