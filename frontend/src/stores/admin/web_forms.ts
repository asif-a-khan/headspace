import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface WebForm {
  id: number;
  form_id: string;
  title: string;
  description: string | null;
  submit_button_label: string;
  submit_success_action: string;
  submit_success_content: string;
  create_lead: boolean;
  background_color: string | null;
  form_background_color: string | null;
  form_title_color: string | null;
  form_submit_button_color: string | null;
  attribute_label_color: string | null;
  created_at: string;
  updated_at: string;
}

export const useWebFormsStore = defineStore("webForms", () => {
  const items = ref<WebForm[]>([]);

  function hydrate(data: WebForm[]) {
    items.value = data;
  }

  async function fetchAll() {
    const res = await get<{ data: WebForm[] }>("/admin/api/settings/web-forms");
    items.value = res.data;
  }

  async function create(payload: Record<string, unknown>) {
    const res = await post<{ data: WebForm; message: string }>("/admin/api/settings/web-forms", payload);
    return res;
  }

  async function update(id: number, payload: Record<string, unknown>) {
    const res = await put<{ data: WebForm; message: string }>(`/admin/api/settings/web-forms/${id}`, payload);
    return res;
  }

  async function remove(id: number) {
    await del<{ message: string }>(`/admin/api/settings/web-forms/${id}`);
    items.value = items.value.filter((f) => f.id !== id);
  }

  return { items, hydrate, fetchAll, create, update, remove };
});
