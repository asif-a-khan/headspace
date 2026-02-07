import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface QuoteItem {
  id?: number;
  sku: string | null;
  name: string | null;
  quantity: number;
  price: number;
  discount_percent: number;
  discount_amount: number;
  tax_percent: number;
  tax_amount: number;
  total: number;
  product_id: number | null;
  is_delete?: boolean;
}

export interface Quote {
  id: number;
  subject: string;
  description: string | null;
  billing_address: Record<string, string> | null;
  shipping_address: Record<string, string> | null;
  discount_percent: string | null;
  discount_amount: string | null;
  tax_amount: string | null;
  adjustment_amount: string | null;
  sub_total: string | null;
  grand_total: string | null;
  expired_at: string | null;
  person_id: number | null;
  user_id: number | null;
  created_at: string;
  updated_at: string;
  person_name?: string | null;
  user_name?: string | null;
}

export const useQuotesStore = defineStore("admin-quotes", () => {
  const quotes = ref<Quote[]>([]);
  const loading = ref(false);

  function hydrate(data: { quotes?: Quote[] }) {
    if (data.quotes) quotes.value = data.quotes;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Quote[] }>("/admin/api/quotes");
      quotes.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Quote }>("/admin/api/quotes", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Quote }>(`/admin/api/quotes/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/quotes/${id}`);
    quotes.value = quotes.value.filter((q) => q.id !== id);
  }

  return { quotes, loading, hydrate, fetchAll, create, update, remove };
});
