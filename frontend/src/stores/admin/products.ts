import { defineStore } from "pinia";
import { ref } from "vue";
import { get, post, put, del } from "@/api/client";

export interface Product {
  id: number;
  sku: string;
  name: string;
  description: string | null;
  price: string;
  quantity: number;
  created_at: string;
  updated_at: string;
}

export const useProductsStore = defineStore("admin-products", () => {
  const products = ref<Product[]>([]);
  const loading = ref(false);

  function hydrate(data: { products?: Product[] }) {
    if (data.products) products.value = data.products;
  }

  async function fetchAll() {
    loading.value = true;
    try {
      const res = await get<{ data: Product[] }>("/admin/api/products");
      products.value = res.data;
    } finally {
      loading.value = false;
    }
  }

  async function create(form: Record<string, unknown>) {
    return post<{ data: Product }>("/admin/api/products", form);
  }

  async function update(id: number, form: Record<string, unknown>) {
    return put<{ data: Product }>(`/admin/api/products/${id}`, form);
  }

  async function remove(id: number) {
    await del(`/admin/api/products/${id}`);
    products.value = products.value.filter((p) => p.id !== id);
  }

  return { products, loading, hydrate, fetchAll, create, update, remove };
});
