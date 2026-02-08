<template>
  <div>
    <div class="org-detail-layout d-flex ga-4" style="align-items: flex-start">
      <!-- LEFT Sidebar (sticky) -->
      <div class="org-sidebar">
        <v-card variant="outlined" class="org-sidebar-card">
          <!-- Tags Section -->
          <div class="pa-4 border-b">
            <div v-if="tags.length" class="d-flex flex-wrap ga-1 mb-3">
              <v-chip
                v-for="tag in tags"
                :key="tag.id"
                :color="tag.color || '#6366F1'"
                size="small"
                variant="tonal"
              >
                {{ tag.name }}
              </v-chip>
            </div>

            <!-- Organization Name -->
            <div class="d-flex align-center mb-3">
              <v-avatar color="primary" variant="tonal" size="40" class="mr-3">
                <v-icon>mdi-domain</v-icon>
              </v-avatar>
              <div style="min-width: 0">
                <h3 class="font-weight-bold text-truncate" style="font-size: 1.125rem; line-height: 1.4">{{ organization.name }}</h3>
              </div>
            </div>
          </div>

          <!-- Attributes Accordion -->
          <div class="pa-4 border-b">
            <v-expansion-panels variant="accordion" flat>
              <v-expansion-panel>
                <v-expansion-panel-title class="px-0 py-2">
                  <div class="d-flex align-center w-100">
                    <span class="text-subtitle-2 font-weight-medium">Attributes</span>
                    <v-spacer />
                    <v-btn
                      v-if="canEdit"
                      icon="mdi-pencil"
                      size="x-small"
                      variant="text"
                      :href="`/admin/contacts/organizations/${organization.id}/edit`"
                      target="_blank"
                      @click.stop
                    />
                  </div>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
                  <div class="d-flex flex-column ga-3">
                    <div v-if="address">
                      <div class="text-caption text-medium-emphasis">Address</div>
                      <div class="text-body-2">{{ address }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Assigned To</div>
                      <div class="text-body-2">{{ userName || '-' }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Created</div>
                      <div class="text-body-2">{{ formatDateTime(organization.created_at) }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Updated</div>
                      <div class="text-body-2">{{ formatDateTime(organization.updated_at) }}</div>
                    </div>
                  </div>
                </v-expansion-panel-text>
              </v-expansion-panel>
            </v-expansion-panels>
          </div>
        </v-card>
      </div>

      <!-- RIGHT Content Area -->
      <div style="flex: 1; min-width: 0">
        <!-- Persons linked to this org -->
        <v-card v-if="persons.length" class="mb-4" variant="outlined">
          <v-card-title class="d-flex align-center py-2 px-4">
            <v-icon start size="small">mdi-account-group</v-icon>
            <span class="text-subtitle-2">Persons</span>
            <v-chip size="x-small" variant="tonal" class="ml-2">{{ persons.length }}</v-chip>
          </v-card-title>
          <v-card-text class="pa-0">
            <v-list density="compact">
              <v-list-item
                v-for="person in persons"
                :key="person.id"
                :href="`/admin/contacts/persons/${person.id}`"
                density="compact"
              >
                <template #prepend>
                  <v-avatar color="primary" variant="tonal" size="28" class="mr-2">
                    <span class="text-caption font-weight-bold">{{ personInitials(person.name) }}</span>
                  </v-avatar>
                </template>
                <v-list-item-title class="text-body-2">{{ person.name }}</v-list-item-title>
                <v-list-item-subtitle v-if="person.job_title" class="text-caption">
                  {{ person.job_title }}
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-card-text>
        </v-card>

        <!-- Leads linked to persons in this org -->
        <v-card v-if="leads.length" class="mb-4" variant="outlined">
          <v-card-title class="d-flex align-center py-2 px-4">
            <v-icon start size="small">mdi-flag</v-icon>
            <span class="text-subtitle-2">Leads</span>
            <v-chip size="x-small" variant="tonal" class="ml-2">{{ leads.length }}</v-chip>
          </v-card-title>
          <v-card-text class="pa-0">
            <v-list density="compact">
              <v-list-item
                v-for="lead in leads"
                :key="lead.id"
                :href="`/admin/leads/${lead.id}`"
                density="compact"
              >
                <v-list-item-title class="d-flex align-center ga-2 text-body-2">
                  {{ lead.title }}
                  <v-chip v-if="lead.status === null" color="info" size="x-small">Open</v-chip>
                  <v-chip v-else-if="lead.status" color="success" size="x-small">Won</v-chip>
                  <v-chip v-else color="error" size="x-small">Lost</v-chip>
                </v-list-item-title>
                <v-list-item-subtitle class="text-caption">
                  <span v-if="lead.lead_value">${{ Number(lead.lead_value).toLocaleString() }}</span>
                  <span v-if="lead.person_name" class="ml-2">{{ lead.person_name }}</span>
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-card-text>
        </v-card>

        <!-- Activity Timeline -->
        <ActivityTimeline :activities="activities" />
      </div>
    </div>

    <!-- Delete Dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Organization</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ organization.name }}"? This action cannot be undone.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" :loading="deleting" @click="doDelete">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="errorSnackbar" color="error" :timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { del } from "@/api/client";
import ActivityTimeline from "@/components/admin/shared/ActivityTimeline.vue";

const data = window.__INITIAL_DATA__ || {};

const organization = data.organization || {};
const persons: any[] = data.persons || [];
const leads: any[] = data.leads || [];
const activities: any[] = data.activities || [];
const tags: any[] = data.tags || [];
const userName: string | null = data.user_name || null;

const permissions: string[] = data.permissions || [];
const canEdit = computed(() => permissions.includes("contacts.organizations.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("contacts.organizations.delete") || data.permission_type === "all");

const address = computed(() => {
  if (!organization.address) return "";
  const addr = typeof organization.address === "string" ? JSON.parse(organization.address) : organization.address;
  const parts = [addr.address, addr.city, addr.state, addr.postcode, addr.country].filter(Boolean);
  return parts.join(", ");
});

function personInitials(name: string): string {
  const parts = (name || "").split(" ");
  return parts.map((p: string) => p.charAt(0).toUpperCase()).slice(0, 2).join("");
}

function formatDateTime(value: string): string {
  if (!value) return "-";
  return new Date(value).toLocaleString();
}

// --- Delete ---
const deleteDialog = ref(false);
const deleting = ref(false);
const errorSnackbar = ref(false);
const errorMessage = ref("");

async function doDelete() {
  deleting.value = true;
  try {
    await del(`/admin/api/contacts/organizations/${organization.id}`);
    window.location.href = "/admin/contacts/organizations";
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to delete organization.";
    errorSnackbar.value = true;
  } finally {
    deleting.value = false;
  }
}
</script>

<style scoped>
.org-detail-layout {
  flex-wrap: wrap;
}
.org-sidebar {
  width: 394px;
  min-width: 394px;
  max-width: 394px;
  flex-shrink: 0;
}
.org-sidebar-card {
  position: sticky;
  top: 73px;
  align-self: flex-start;
}
.border-b {
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
@media (max-width: 960px) {
  .org-sidebar {
    width: 100%;
    min-width: 100%;
    max-width: 100%;
  }
  .org-sidebar-card {
    position: static;
  }
}
</style>
