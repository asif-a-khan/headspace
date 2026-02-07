import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Lead {
  id: number;
  title: string;
  description: string | null;
  lead_value: string | null;
  status: boolean | null;
  lost_reason: string | null;
  closed_at: string | null;
  expected_close_date: string | null;
  user_id: number | null;
  person_id: number | null;
  lead_source_id: number | null;
  lead_type_id: number | null;
  lead_pipeline_id: number | null;
  lead_pipeline_stage_id: number | null;
  created_at: string;
  updated_at: string;
  person_name?: string | null;
  user_name?: string | null;
  source_name?: string | null;
  type_name?: string | null;
  stage_name?: string | null;
  pipeline_name?: string | null;
}

export interface KanbanCard {
  id: number;
  title: string;
  lead_value: string | null;
  lead_pipeline_stage_id: number | null;
  person_name: string | null;
  created_at: string;
}

export const useLeadsStore = defineStore("admin-leads", () => {
  const leads = ref<Lead[]>([]);
  const kanbanCards = ref<KanbanCard[]>([]);
  const loading = ref(false);

  function hydrate(data: { leads?: Lead[] }) {
    if (data.leads) leads.value = data.leads;
  }

  async function fetchAll(pipelineId?: number) {
    loading.value = true;
    try {
      const url = pipelineId
        ? `/admin/api/leads?pipeline_id=${pipelineId}`
        : "/admin/api/leads";
      const res = await get<{ data: Lead[] }>(url);
      leads.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function fetchKanban(pipelineId?: number) {
    loading.value = true;
    try {
      const url = pipelineId
        ? `/admin/api/leads/kanban?pipeline_id=${pipelineId}`
        : "/admin/api/leads/kanban";
      const res = await get<{ data: KanbanCard[] }>(url);
      kanbanCards.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Lead }>("/admin/api/leads", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Lead }>(`/admin/api/leads/${id}`, form);
  }

  async function moveToStage(id: number, stageId: number) {
    return put<{ data: Lead }>(`/admin/api/leads/${id}/stage`, {
      lead_pipeline_stage_id: stageId,
    });
  }

  async function remove(id: number) {
    await del(`/admin/api/leads/${id}`);
    leads.value = leads.value.filter((l) => l.id !== id);
    kanbanCards.value = kanbanCards.value.filter((c) => c.id !== id);
  }

  return { leads, kanbanCards, loading, hydrate, fetchAll, fetchKanban, create, update, moveToStage, remove };
});
