<template>
  <div>
    <div class="person-detail-layout d-flex ga-4" style="align-items: flex-start">
      <!-- LEFT Sidebar (sticky) -->
      <div class="person-sidebar">
        <v-card variant="outlined" class="person-sidebar-card">
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

            <!-- Person Name + Job Title -->
            <div class="d-flex align-center mb-3">
              <v-avatar color="primary" variant="tonal" size="40" class="mr-3">
                <span class="text-body-1 font-weight-bold">{{ initials }}</span>
              </v-avatar>
              <div style="min-width: 0">
                <h3 class="font-weight-bold text-truncate" style="font-size: 1.125rem; line-height: 1.4">{{ person.name }}</h3>
                <div v-if="person.job_title" class="text-body-2 text-medium-emphasis text-truncate">
                  {{ person.job_title }}
                </div>
              </div>
            </div>

            <!-- Activity Action Buttons -->
            <div class="d-flex flex-wrap ga-2">
              <v-btn
                v-if="canComposeMail"
                size="small"
                variant="tonal"
                color="success"
                prepend-icon="mdi-email"
                :href="`/admin/mail/compose?person_id=${person.id}`"
              >
                Mail
              </v-btn>
              <v-btn
                v-if="canCreateActivity"
                size="small"
                variant="tonal"
                prepend-icon="mdi-file-plus"
                :href="`/admin/activities/create?person_id=${person.id}&type=file`"
              >
                File
              </v-btn>
              <v-btn
                v-if="canCreateActivity"
                size="small"
                variant="tonal"
                color="warning"
                prepend-icon="mdi-note-plus"
                :href="`/admin/activities/create?person_id=${person.id}&type=note`"
              >
                Note
              </v-btn>
              <v-btn
                v-if="canCreateActivity"
                size="small"
                variant="tonal"
                color="info"
                prepend-icon="mdi-calendar-plus"
                :href="`/admin/activities/create?person_id=${person.id}&type=meeting`"
              >
                Activity
              </v-btn>
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
                      :href="`/admin/contacts/persons/${person.id}/edit`"
                      target="_blank"
                      @click.stop
                    />
                  </div>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
                  <div class="d-flex flex-column ga-3">
                    <div v-if="organization">
                      <div class="text-caption text-medium-emphasis">Organization</div>
                      <a :href="`/admin/contacts/organizations/${organization.id}`" class="text-body-2 text-decoration-none">
                        {{ organization.name }}
                      </a>
                    </div>
                    <div v-if="emails.length">
                      <div class="text-caption text-medium-emphasis">Email</div>
                      <div v-for="(email, i) in emails" :key="i" class="text-body-2">
                        {{ email.value }}
                        <v-chip v-if="email.label" size="x-small" variant="outlined" class="ml-1">{{ email.label }}</v-chip>
                      </div>
                    </div>
                    <div v-if="phones.length">
                      <div class="text-caption text-medium-emphasis">Phone</div>
                      <div v-for="(phone, i) in phones" :key="i" class="text-body-2">
                        {{ phone.value }}
                        <v-chip v-if="phone.label" size="x-small" variant="outlined" class="ml-1">{{ phone.label }}</v-chip>
                      </div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Assigned To</div>
                      <div class="text-body-2">{{ userName || '-' }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Created</div>
                      <div class="text-body-2">{{ formatDateTime(person.created_at) }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Updated</div>
                      <div class="text-body-2">{{ formatDateTime(person.updated_at) }}</div>
                    </div>
                  </div>
                </v-expansion-panel-text>
              </v-expansion-panel>
            </v-expansion-panels>
          </div>

          <!-- Organization Card -->
          <div v-if="organization" class="pa-4">
            <div class="text-subtitle-2 font-weight-medium mb-2">Organization</div>
            <a :href="`/admin/contacts/organizations/${organization.id}`" class="text-body-2 font-weight-medium text-decoration-none">
              {{ organization.name }}
            </a>
            <div v-if="orgAddress" class="text-caption text-medium-emphasis mt-1">
              {{ orgAddress }}
            </div>
          </div>
        </v-card>
      </div>

      <!-- RIGHT Content Area -->
      <div style="flex: 1; min-width: 0">
        <!-- Leads (collapsible section above timeline) -->
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
                  <span v-if="lead.pipeline_name" class="ml-2">{{ lead.pipeline_name }}</span>
                  <span v-if="lead.stage_name" class="ml-1">/ {{ lead.stage_name }}</span>
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
        <v-card-title>Delete Person</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ person.name }}"? This action cannot be undone.
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

const person = data.person || {};
const organization = data.organization || null;
const userName: string | null = data.user_name || null;
const activities: any[] = data.activities || [];
const leads: any[] = data.leads || [];
const tags: any[] = data.tags || [];

const permissions: string[] = data.permissions || [];
const canEdit = computed(() => permissions.includes("contacts.persons.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("contacts.persons.delete") || data.permission_type === "all");
const canComposeMail = computed(() => permissions.includes("mail.compose") || data.permission_type === "all");
const canCreateActivity = computed(() => permissions.includes("activities.create") || data.permission_type === "all");

const initials = computed(() => {
  const parts = (person.name || "").split(" ");
  return parts.map((p: string) => p.charAt(0).toUpperCase()).slice(0, 2).join("");
});

const emails = computed(() => {
  if (!person.emails) return [];
  if (typeof person.emails === "string") {
    try { return JSON.parse(person.emails); } catch { return []; }
  }
  return Array.isArray(person.emails) ? person.emails : [];
});

const phones = computed(() => {
  if (!person.contact_numbers) return [];
  if (typeof person.contact_numbers === "string") {
    try { return JSON.parse(person.contact_numbers); } catch { return []; }
  }
  return Array.isArray(person.contact_numbers) ? person.contact_numbers : [];
});

const orgAddress = computed(() => {
  if (!organization?.address) return "";
  const addr = typeof organization.address === "string" ? JSON.parse(organization.address) : organization.address;
  const parts = [addr.address, addr.city, addr.state, addr.postcode, addr.country].filter(Boolean);
  return parts.join(", ");
});

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
    await del(`/admin/api/contacts/persons/${person.id}`);
    window.location.href = "/admin/contacts/persons";
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to delete person.";
    errorSnackbar.value = true;
  } finally {
    deleting.value = false;
  }
}
</script>

<style scoped>
.person-detail-layout {
  flex-wrap: wrap;
}
.person-sidebar {
  width: 394px;
  min-width: 394px;
  max-width: 394px;
  flex-shrink: 0;
}
.person-sidebar-card {
  position: sticky;
  top: 73px;
  align-self: flex-start;
}
.border-b {
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
@media (max-width: 960px) {
  .person-sidebar {
    width: 100%;
    min-width: 100%;
    max-width: 100%;
  }
  .person-sidebar-card {
    position: static;
  }
}
</style>
