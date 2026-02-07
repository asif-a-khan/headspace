<template>
  <div>
    <h1 class="text-h5 font-weight-bold mb-2">Dashboard</h1>
    <p class="text-body-1 text-secondary mb-6">Welcome to {{ companyName }}</p>

    <v-row>
      <v-col cols="12" md="4">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center mb-3">
              <v-avatar color="primary" size="40" class="mr-3">
                <v-icon color="white">mdi-account-group</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">0</p>
                <p class="text-body-2 text-secondary">Total Leads</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center mb-3">
              <v-avatar color="success" size="40" class="mr-3">
                <v-icon color="white">mdi-handshake</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">0</p>
                <p class="text-body-2 text-secondary">Won Deals</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" md="4">
        <v-card rounded="lg" elevation="0">
          <v-card-text class="pa-5">
            <div class="d-flex align-center mb-3">
              <v-avatar color="info" size="40" class="mr-3">
                <v-icon color="white">mdi-calendar-check</v-icon>
              </v-avatar>
              <div>
                <p class="text-h6 font-weight-bold">0</p>
                <p class="text-body-2 text-secondary">Activities</p>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const data = (window as any).__INITIAL_DATA__ || {};
const companyName = computed(() => data.company_name || "Headspace");
</script>
