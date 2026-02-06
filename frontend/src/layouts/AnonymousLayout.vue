<template>
  <v-app>
    <v-main class="bg-background d-flex align-center justify-center" style="min-height: 100vh">
      <div class="d-flex flex-column align-center">
        <div class="mb-6 d-flex align-center">
          <svg width="40" height="40" viewBox="0 0 32 32" class="mr-3">
            <rect width="32" height="32" rx="6" fill="#6366F1"/>
            <text x="16" y="23" text-anchor="middle" fill="white" font-size="20" font-weight="700" font-family="Inter, sans-serif">H</text>
          </svg>
          <span class="text-h4 font-weight-bold text-on-background">Headspace</span>
        </div>
        <slot />
      </div>
    </v-main>
  </v-app>
</template>
