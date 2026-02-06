import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Agent {
  id: number;
  first_name: string;
  last_name: string;
  email: string;
  image: string | null;
  status: boolean;
  role_id: number;
  created_at: string;
  updated_at: string;
}

export interface AgentForm {
  first_name: string;
  last_name: string;
  email: string;
  password: string;
  password_confirmation: string;
  status: boolean;
  role_id: number | null;
}

export interface Role {
  id: number;
  name: string;
}

export const useAgentsStore = defineStore("super-agents", () => {
  const agents = ref<Agent[]>([]);
  const roles = ref<Role[]>([]);
  const loading = ref(false);

  function hydrate(data: { agents?: Agent[]; roles?: Role[] }) {
    if (data.agents) agents.value = data.agents;
    if (data.roles) roles.value = data.roles;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      agents.value = await get<Agent[]>("/super/api/agents");
    } finally {
      loading.value = false;
    }
  }

  async function create(form: AgentForm) {
    return post<Agent>("/super/api/agents", form);
  }

  async function update(id: number, form: Partial<AgentForm>) {
    return put<Agent>(`/super/api/agents/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/super/api/agents/${id}`);
    agents.value = agents.value.filter((a) => a.id !== id);
  }

  return { agents, roles, loading, hydrate, fetchAll, create, update, remove };
});
