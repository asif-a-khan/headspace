<template>
  <div>
    <h1 class="text-h5 mb-4">My Account</h1>

    <!-- Profile Information -->
    <v-card max-width="700" class="mb-6">
      <v-card-title class="text-h6">Profile Information</v-card-title>
      <v-card-text>
        <v-alert v-if="profileError" type="error" density="compact" class="mb-4">
          {{ profileError }}
        </v-alert>
        <v-alert v-if="profileSuccess" type="success" density="compact" class="mb-4">
          {{ profileSuccess }}
        </v-alert>
        <v-form @submit.prevent="updateProfile">
          <v-text-field
            v-model="profileForm.first_name"
            label="First Name"
            required
            :rules="[rules.required]"
          />
          <v-text-field
            v-model="profileForm.last_name"
            label="Last Name"
            required
            class="mt-2"
            :rules="[rules.required]"
          />
          <v-text-field
            v-model="profileForm.email"
            label="Email"
            type="email"
            required
            class="mt-2"
            :rules="[rules.required, rules.email]"
          />
          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="profileLoading">
              Save Profile
            </v-btn>
          </div>
        </v-form>
      </v-card-text>
    </v-card>

    <!-- Change Password -->
    <v-card max-width="700">
      <v-card-title class="text-h6">Change Password</v-card-title>
      <v-card-text>
        <v-alert v-if="passwordError" type="error" density="compact" class="mb-4">
          {{ passwordError }}
        </v-alert>
        <v-alert v-if="passwordSuccess" type="success" density="compact" class="mb-4">
          {{ passwordSuccess }}
        </v-alert>
        <v-form @submit.prevent="updatePassword">
          <v-text-field
            v-model="passwordForm.current_password"
            label="Current Password"
            type="password"
            required
            :rules="[rules.required]"
          />
          <v-text-field
            v-model="passwordForm.password"
            label="New Password"
            type="password"
            required
            class="mt-2"
            :rules="[rules.required, rules.minLength]"
          />
          <v-text-field
            v-model="passwordForm.password_confirmation"
            label="Confirm New Password"
            type="password"
            required
            class="mt-2"
            :rules="[rules.required, rules.passwordMatch]"
          />
          <div class="d-flex gap-2 mt-4">
            <v-btn type="submit" color="primary" :loading="passwordLoading">
              Update Password
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

const data = (window as any).__INITIAL_DATA__ || {};

// --- Profile form ---
const profileError = ref("");
const profileSuccess = ref("");
const profileLoading = ref(false);

const profileForm = reactive({
  first_name: data.account?.first_name || "",
  last_name: data.account?.last_name || "",
  email: data.account?.email || "",
});

async function updateProfile() {
  profileError.value = "";
  profileSuccess.value = "";
  profileLoading.value = true;
  try {
    await put("/admin/api/account", profileForm);
    profileSuccess.value = "Profile updated successfully.";
  } catch (e: any) {
    profileError.value = e.message || "Failed to update profile.";
  } finally {
    profileLoading.value = false;
  }
}

// --- Password form ---
const passwordError = ref("");
const passwordSuccess = ref("");
const passwordLoading = ref(false);

const passwordForm = reactive({
  current_password: "",
  password: "",
  password_confirmation: "",
});

async function updatePassword() {
  if (passwordForm.password !== passwordForm.password_confirmation) {
    passwordError.value = "New passwords do not match.";
    return;
  }
  passwordError.value = "";
  passwordSuccess.value = "";
  passwordLoading.value = true;
  try {
    await put("/admin/api/account/password", passwordForm);
    passwordSuccess.value = "Password updated successfully.";
    passwordForm.current_password = "";
    passwordForm.password = "";
    passwordForm.password_confirmation = "";
  } catch (e: any) {
    passwordError.value = e.message || "Failed to update password.";
  } finally {
    passwordLoading.value = false;
  }
}

// --- Validation rules ---
const rules = {
  required: (v: string) => !!v || "This field is required.",
  email: (v: string) => /.+@.+\..+/.test(v) || "Invalid email address.",
  minLength: (v: string) => v.length >= 6 || "Must be at least 6 characters.",
  passwordMatch: (v: string) =>
    v === passwordForm.password || "Passwords do not match.",
};
</script>
