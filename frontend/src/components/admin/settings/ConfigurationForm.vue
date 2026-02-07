<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />
    <h1 class="text-h5 mb-4">Configuration</h1>

    <v-row>
      <v-col cols="12" md="6">
        <v-card>
          <v-card-title>General Settings</v-card-title>
          <v-card-text>
            <v-select
              v-model="config['general.currency_symbol']"
              :items="currencyOptions"
              item-title="label"
              item-value="value"
              label="Currency Symbol"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-select
              v-model="config['general.date_format']"
              :items="dateFormatOptions"
              item-title="label"
              item-value="value"
              label="Date Format"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-select
              v-model="config['general.timezone']"
              :items="timezoneOptions"
              label="Timezone"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-select
              v-model="config['general.locale']"
              :items="localeOptions"
              item-title="label"
              item-value="value"
              label="Locale"
              variant="outlined"
              density="compact"
            />
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="6">
        <v-card>
          <v-card-title>Appearance</v-card-title>
          <v-card-text>
            <div class="mb-3">
              <label class="text-body-2 d-block mb-1">Brand Color</label>
              <div class="d-flex align-center ga-3">
                <v-text-field
                  v-model="config['appearance.brand_color']"
                  variant="outlined"
                  density="compact"
                  hide-details
                  style="max-width: 160px"
                />
                <div
                  :style="{
                    width: '36px',
                    height: '36px',
                    borderRadius: '4px',
                    backgroundColor: config['appearance.brand_color'] || '#6366F1',
                    border: '1px solid rgba(0,0,0,0.2)',
                  }"
                />
              </div>
            </div>
          </v-card-text>
        </v-card>

        <v-card class="mt-4">
          <v-card-title>Preview</v-card-title>
          <v-card-text>
            <p class="text-body-2">
              Currency: <strong>{{ config['general.currency_symbol'] || '$' }}1,234.56</strong>
            </p>
            <p class="text-body-2">
              Date format: <strong>{{ previewDate }}</strong>
            </p>
            <p class="text-body-2">
              Timezone: <strong>{{ config['general.timezone'] || 'UTC' }}</strong>
            </p>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <v-row class="mt-2">
      <v-col cols="12" md="6">
        <v-card>
          <v-card-title>Email / SMTP</v-card-title>
          <v-card-text>
            <v-text-field
              v-model="config['email.smtp.host']"
              label="SMTP Host"
              placeholder="smtp.gmail.com"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-text-field
              v-model="config['email.smtp.port']"
              label="SMTP Port"
              placeholder="587"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-select
              v-model="config['email.smtp.encryption']"
              :items="encryptionOptions"
              label="Encryption"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-text-field
              v-model="config['email.smtp.username']"
              label="Username"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-text-field
              v-model="config['email.smtp.password']"
              label="Password"
              type="password"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-text-field
              v-model="config['email.smtp.from_address']"
              label="From Address"
              placeholder="no-reply@example.com"
              variant="outlined"
              density="compact"
              class="mb-3"
            />
            <v-text-field
              v-model="config['email.smtp.from_name']"
              label="From Name"
              placeholder="Company Name"
              variant="outlined"
              density="compact"
            />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <div class="d-flex mt-4">
      <v-spacer />
      <v-btn color="primary" @click="save" :loading="saving" prepend-icon="mdi-content-save">
        Save Configuration
      </v-btn>
    </div>

    <v-snackbar v-model="snackbar" :color="snackbarColor" :timeout="3000">
      {{ snackbarText }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from "vue";

const data = window.__INITIAL_DATA__ || {};
const config = reactive<Record<string, string>>(data.config || {});

const breadcrumbs = [
  { title: "Settings", href: "/admin/settings" },
  { title: "Configuration", disabled: true },
];

const currencyOptions = [
  { label: "$ (USD)", value: "$" },
  { label: "\u20AC (EUR)", value: "\u20AC" },
  { label: "\u00A3 (GBP)", value: "\u00A3" },
  { label: "\u00A5 (JPY/CNY)", value: "\u00A5" },
  { label: "\u20B9 (INR)", value: "\u20B9" },
  { label: "R$ (BRL)", value: "R$" },
  { label: "$ (CAD/AUD)", value: "$" },
  { label: "CHF", value: "CHF" },
  { label: "kr (SEK/NOK/DKK)", value: "kr" },
  { label: "z\u0142 (PLN)", value: "z\u0142" },
];

const dateFormatOptions = [
  { label: "YYYY-MM-DD (2026-02-07)", value: "YYYY-MM-DD" },
  { label: "MM/DD/YYYY (02/07/2026)", value: "MM/DD/YYYY" },
  { label: "DD/MM/YYYY (07/02/2026)", value: "DD/MM/YYYY" },
  { label: "DD.MM.YYYY (07.02.2026)", value: "DD.MM.YYYY" },
  { label: "MMM DD, YYYY (Feb 07, 2026)", value: "MMM DD, YYYY" },
];

const timezoneOptions = [
  "UTC",
  "America/New_York",
  "America/Chicago",
  "America/Denver",
  "America/Los_Angeles",
  "America/Sao_Paulo",
  "Europe/London",
  "Europe/Paris",
  "Europe/Berlin",
  "Europe/Moscow",
  "Asia/Dubai",
  "Asia/Kolkata",
  "Asia/Shanghai",
  "Asia/Tokyo",
  "Australia/Sydney",
  "Pacific/Auckland",
];

const localeOptions = [
  { label: "English", value: "en" },
  { label: "Spanish", value: "es" },
  { label: "French", value: "fr" },
  { label: "German", value: "de" },
  { label: "Portuguese", value: "pt" },
  { label: "Japanese", value: "ja" },
  { label: "Chinese", value: "zh" },
  { label: "Arabic", value: "ar" },
  { label: "Hindi", value: "hi" },
];

const encryptionOptions = ["tls", "ssl", "none"];

const previewDate = computed(() => {
  const now = new Date();
  const fmt = config["general.date_format"] || "YYYY-MM-DD";
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  switch (fmt) {
    case "MM/DD/YYYY": return `${m}/${d}/${y}`;
    case "DD/MM/YYYY": return `${d}/${m}/${y}`;
    case "DD.MM.YYYY": return `${d}.${m}.${y}`;
    case "MMM DD, YYYY": return `${months[now.getMonth()]} ${d}, ${y}`;
    default: return `${y}-${m}-${d}`;
  }
});

const saving = ref(false);
const snackbar = ref(false);
const snackbarText = ref("");
const snackbarColor = ref("success");

async function save() {
  saving.value = true;
  try {
    const csrfMeta = document.querySelector('meta[name="csrf-token"]');
    const resp = await fetch("/admin/api/settings/config", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        "X-CSRF-TOKEN": csrfMeta?.getAttribute("content") || "",
      },
      body: JSON.stringify({ config }),
    });
    const result = await resp.json();
    snackbarText.value = result.message || "Saved.";
    snackbarColor.value = resp.ok ? "success" : "error";
    snackbar.value = true;
  } catch {
    snackbarText.value = "Failed to save configuration.";
    snackbarColor.value = "error";
    snackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
