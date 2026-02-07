import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Pipeline {
  id: number;
  name: string;
  is_default: boolean;
  rotten_days: number;
  created_at: string;
  updated_at: string;
}

export interface Stage {
  id: number;
  code: string;
  name: string;
  is_user_defined: boolean;
  created_at: string;
  updated_at: string;
}

export interface PipelineStageDetail {
  id: number;
  probability: number;
  sort_order: number;
  lead_stage_id: number;
  lead_pipeline_id: number;
  stage_code: string;
  stage_name: string;
}

export const usePipelinesStore = defineStore("admin-pipelines", () => {
  const pipelines = ref<Pipeline[]>([]);
  const stages = ref<Stage[]>([]);
  const loading = ref(false);

  function hydrate(data: { pipelines?: Pipeline[]; stages?: Stage[] }) {
    if (data.pipelines) pipelines.value = data.pipelines;
    if (data.stages) stages.value = data.stages;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Pipeline[]; stages: Stage[] }>(
        "/admin/api/settings/pipelines",
      );
      pipelines.value = res.data;
      stages.value = res.stages;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Pipeline }>(
      "/admin/api/settings/pipelines",
      form,
    );
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Pipeline }>(
      `/admin/api/settings/pipelines/${id}`,
      form,
    );
  }

  async function remove(id: number) {
    await del(`/admin/api/settings/pipelines/${id}`);
    pipelines.value = pipelines.value.filter((p) => p.id !== id);
  }

  return { pipelines, stages, loading, hydrate, fetchAll, create, update, remove };
});
