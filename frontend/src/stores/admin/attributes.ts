import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Attribute {
  id: number;
  code: string;
  name: string;
  type: string;
  entity_type: string;
  sort_order: number;
  validation: string | null;
  is_required: boolean;
  is_unique: boolean;
  quick_add: boolean;
  is_user_defined: boolean;
  created_at: string;
  updated_at: string;
}

export interface AttributeOption {
  id?: number;
  name: string;
  sort_order: number;
  is_delete?: boolean;
}

export const useAttributesStore = defineStore("admin-attributes", () => {
  const attributes = ref<Attribute[]>([]);
  const loading = ref(false);

  function hydrate(data: { attributes?: Attribute[] }) {
    if (data.attributes) attributes.value = data.attributes;
  }

  async function fetchAll(entityType?: string) {
    loading.value = true;
    try {
      const url = entityType
        ? `/admin/api/settings/attributes?entity_type=${entityType}`
        : "/admin/api/settings/attributes";
      const res = await get<{ data: Attribute[] }>(url);
      attributes.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Attribute }>("/admin/api/settings/attributes", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Attribute }>(
      `/admin/api/settings/attributes/${id}`,
      form,
    );
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/attributes/${id}`);
    attributes.value = attributes.value.filter((a) => a.id !== id);
  }

  return { attributes, loading, hydrate, fetchAll, create, update, remove };
});
