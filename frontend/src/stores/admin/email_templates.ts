import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface EmailTemplate {
  id: number;
  name: string;
  subject: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export const useEmailTemplatesStore = defineStore("admin-email-templates", () => {
  const templates = ref<EmailTemplate[]>([]);
  const loading = ref(false);

  function hydrate(data: { email_templates?: EmailTemplate[] }) {
    if (data.email_templates) templates.value = data.email_templates;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: EmailTemplate[] }>("/admin/api/settings/email-templates");
      templates.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: EmailTemplate }>("/admin/api/settings/email-templates", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: EmailTemplate }>(`/admin/api/settings/email-templates/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/email-templates/${id}`);
    templates.value = templates.value.filter((t) => t.id !== id);
  }

  return { templates, loading, hydrate, fetchAll, create, update, remove };
});
