<template>
  <v-card width="400" class="pa-4">
    <v-card-title class="text-center text-h5 mb-2">Headspace</v-card-title>
    <v-card-subtitle class="text-center mb-4">Super Admin Login</v-card-subtitle>
    <v-form @submit.prevent="submit">
      <v-alert v-if="error" type="error" density="compact" class="mb-4">
        {{ error }}
      </v-alert>
      <v-text-field
        v-model="form.email"
        label="Email"
        type="email"
        prepend-inner-icon="mdi-email"
        required
        autofocus
      />
      <v-text-field
        v-model="form.password"
        label="Password"
        type="password"
        prepend-inner-icon="mdi-lock"
        required
        class="mt-2"
      />
      <v-btn
        type="submit"
        color="primary"
        block
        size="large"
        :loading="loading"
        class="mt-4"
      >
        Sign In
      </v-btn>
    </v-form>
  </v-card>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { post } from "@/api/client";

const form = reactive({ email: "", password: "" });
const error = ref("");
const loading = ref(false);

async function submit() {
  error.value = "";
  loading.value = true;
  try {
    await post("/super/api/login", form);
    window.location.href = "/super/tenants";
  } catch (e: any) {
    error.value = e.message || "Login failed";
  } finally {
    loading.value = false;
  }
}
</script>
