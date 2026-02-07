<template>
  <v-card width="360" rounded="lg" elevation="1" :border="false">
    <v-card-text class="pa-5 pb-3">
      <p class="text-h6 font-weight-bold text-on-surface">{{ companyName }}</p>
      <p class="text-body-2 text-secondary">Sign in to your account</p>
    </v-card-text>

    <v-form @submit.prevent="submit">
      <v-card-text class="px-5 pt-0 pb-2">
        <v-alert v-if="error" type="error" density="compact" class="mb-4" variant="tonal">
          {{ error }}
        </v-alert>
        <v-text-field
          v-model="form.email"
          label="Email"
          type="email"
          prepend-inner-icon="mdi-email-outline"
          required
          autofocus
          class="mb-2"
        />
        <v-text-field
          v-model="form.password"
          label="Password"
          :type="showPassword ? 'text' : 'password'"
          prepend-inner-icon="mdi-lock-outline"
          :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
          @click:append-inner="showPassword = !showPassword"
          required
        />
      </v-card-text>

      <v-card-actions class="px-5 pb-5 pt-0 d-flex justify-end">
        <v-btn
          type="submit"
          color="primary"
          :loading="loading"
          rounded="md"
        >
          Sign In
        </v-btn>
      </v-card-actions>
    </v-form>
  </v-card>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { post } from "@/api/client";

const data = (window as any).__INITIAL_DATA__ || {};
const companyName = computed(() => data.company_name || "Headspace");

const form = reactive({ email: "", password: "" });
const error = ref("");
const loading = ref(false);
const showPassword = ref(false);

async function submit() {
  error.value = "";
  loading.value = true;
  try {
    await post("/admin/api/login", form);
    window.location.href = "/admin/dashboard";
  } catch (e: any) {
    error.value = e.message || "Login failed";
  } finally {
    loading.value = false;
  }
}
</script>
