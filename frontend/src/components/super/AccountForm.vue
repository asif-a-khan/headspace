<template>
  <div>
    <h1 class="text-h5 mb-4">My Account</h1>
    <v-card max-width="700">
      <v-card-text>
        <v-alert v-if="error" type="error" density="compact" class="mb-4">
          {{ error }}
        </v-alert>
        <v-alert v-if="success" type="success" density="compact" class="mb-4">
          {{ success }}
        </v-alert>
        <v-form @submit.prevent="submit">
          <v-text-field v-model="form.first_name" label="First Name" required />
          <v-text-field v-model="form.last_name" label="Last Name" required class="mt-2" />
          <v-text-field v-model="form.email" label="Email" type="email" required class="mt-2" />
          <v-divider class="my-4" />
          <p class="text-body-2 text-medium-emphasis mb-2">Change Password (optional)</p>
          <v-text-field
            v-model="form.current_password"
            label="Current Password"
            type="password"
            hint="Required to save changes"
            persistent-hint
          />
          <v-text-field
            v-model="form.new_password"
            label="New Password"
            type="password"
            class="mt-2"
          />
          <v-text-field
            v-model="form.new_password_confirmation"
            label="Confirm New Password"
            type="password"
            class="mt-2"
          />
          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="loading">
              Update
            </v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import { put } from "@/api/client";

const data = window.__INITIAL_DATA__ || {};
const error = ref("");
const success = ref("");
const loading = ref(false);

const form = reactive({
  first_name: data.account?.first_name || "",
  last_name: data.account?.last_name || "",
  email: data.account?.email || "",
  current_password: "",
  new_password: "",
  new_password_confirmation: "",
});

async function submit() {
  if (form.new_password && form.new_password !== form.new_password_confirmation) {
    error.value = "New passwords do not match";
    return;
  }
  error.value = "";
  success.value = "";
  loading.value = true;
  try {
    await put("/super/api/account", form);
    success.value = "Account updated successfully";
    form.current_password = "";
    form.new_password = "";
    form.new_password_confirmation = "";
  } catch (e: any) {
    error.value = e.message || "Failed to update";
  } finally {
    loading.value = false;
  }
}
</script>
