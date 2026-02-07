<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />

    <v-row>
      <!-- Left Column -->
      <v-col cols="12" md="8">
        <!-- Person Info Card -->
        <v-card class="mb-4">
          <v-card-text>
            <div class="d-flex align-center mb-4">
              <v-avatar color="primary" size="48" class="mr-3">
                <span class="text-h6 text-white">{{ initials }}</span>
              </v-avatar>
              <div>
                <h1 class="text-h5 font-weight-bold">{{ person.name }}</h1>
                <div v-if="person.job_title" class="text-body-2 text-medium-emphasis">
                  {{ person.job_title }}
                </div>
              </div>
              <v-spacer />
            </div>

            <v-divider class="mb-4" />

            <v-row dense>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Organization</div>
                <div class="text-body-1">
                  <a v-if="person.organization_id" :href="`/admin/contacts/organizations/${person.organization_id}/edit`" class="text-decoration-none">
                    {{ organizationName || '-' }}
                  </a>
                  <span v-else>-</span>
                </div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Assigned To</div>
                <div class="text-body-1">{{ userName || '-' }}</div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Created</div>
                <div class="text-body-2">{{ formatDateTime(person.created_at) }}</div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Updated</div>
                <div class="text-body-2">{{ formatDateTime(person.updated_at) }}</div>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>

        <!-- Leads Section -->
        <v-card class="mb-4">
          <v-card-title class="d-flex align-center">
            <v-icon start>mdi-flag</v-icon>
            Leads
            <v-chip size="x-small" variant="tonal" class="ml-2">{{ leads.length }}</v-chip>
          </v-card-title>
          <v-card-text v-if="leads.length">
            <v-list density="compact">
              <v-list-item
                v-for="lead in leads"
                :key="lead.id"
                :href="`/admin/leads/${lead.id}`"
              >
                <v-list-item-title class="d-flex align-center ga-2">
                  {{ lead.title }}
                  <v-chip v-if="lead.status === null" color="info" size="x-small">Open</v-chip>
                  <v-chip v-else-if="lead.status" color="success" size="x-small">Won</v-chip>
                  <v-chip v-else color="error" size="x-small">Lost</v-chip>
                </v-list-item-title>
                <v-list-item-subtitle>
                  <span v-if="lead.lead_value">${{ Number(lead.lead_value).toLocaleString() }}</span>
                  <span v-if="lead.pipeline_name" class="ml-2">{{ lead.pipeline_name }}</span>
                  <span v-if="lead.stage_name" class="ml-1">/ {{ lead.stage_name }}</span>
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-card-text>
          <v-card-text v-else class="text-center text-medium-emphasis py-8">
            <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-flag-outline</v-icon>
            <div>No leads linked</div>
          </v-card-text>
        </v-card>

        <!-- Activities Section -->
        <v-card class="mb-4">
          <v-card-title class="d-flex align-center">
            <v-icon start>mdi-calendar-check</v-icon>
            Activities
            <v-chip size="x-small" variant="tonal" class="ml-2">{{ activities.length }}</v-chip>
          </v-card-title>
          <v-card-text v-if="activities.length">
            <v-list lines="two" density="compact">
              <v-list-item
                v-for="activity in activities"
                :key="activity.id"
              >
                <template #prepend>
                  <v-icon
                    :color="activity.is_done ? 'success' : 'grey'"
                    size="small"
                  >
                    {{ activity.is_done ? 'mdi-check-circle' : 'mdi-circle-outline' }}
                  </v-icon>
                </template>
                <v-list-item-title class="d-flex align-center ga-2">
                  {{ activity.title }}
                  <v-chip :color="activityTypeColor(activity.type)" size="x-small">
                    {{ activity.type }}
                  </v-chip>
                </v-list-item-title>
                <v-list-item-subtitle>
                  <span v-if="activity.schedule_from">
                    {{ formatDateTime(activity.schedule_from) }}
                    <span v-if="activity.schedule_to"> - {{ formatDateTime(activity.schedule_to) }}</span>
                  </span>
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-card-text>
          <v-card-text v-else class="text-center text-medium-emphasis py-8">
            <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-calendar-blank</v-icon>
            <div>No activities yet</div>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- Right Column: Sidebar -->
      <v-col cols="12" md="4">
        <!-- Action Buttons -->
        <v-card class="mb-4">
          <v-card-text class="d-flex flex-column ga-2">
            <v-btn
              v-if="canEdit"
              color="primary"
              variant="elevated"
              block
              prepend-icon="mdi-pencil"
              :href="`/admin/contacts/persons/${person.id}/edit`"
            >
              Edit Person
            </v-btn>
            <v-btn
              v-if="canDelete"
              color="error"
              variant="outlined"
              block
              prepend-icon="mdi-delete"
              @click="deleteDialog = true"
            >
              Delete Person
            </v-btn>
          </v-card-text>
        </v-card>

        <!-- Contact Info -->
        <v-card class="mb-4">
          <v-card-title>
            <v-icon start>mdi-card-account-details</v-icon>
            Contact Info
          </v-card-title>
          <v-card-text>
            <div v-if="emails.length" class="mb-3">
              <div class="text-caption text-medium-emphasis mb-1">Email</div>
              <div v-for="(email, i) in emails" :key="i" class="text-body-2 mb-1">
                <v-icon size="x-small" class="mr-1">mdi-email-outline</v-icon>
                <a :href="`mailto:${email.value}`" class="text-decoration-none">{{ email.value }}</a>
                <v-chip v-if="email.label" size="x-small" variant="outlined" class="ml-1">{{ email.label }}</v-chip>
              </div>
            </div>

            <div v-if="phones.length">
              <div class="text-caption text-medium-emphasis mb-1">Phone</div>
              <div v-for="(phone, i) in phones" :key="i" class="text-body-2 mb-1">
                <v-icon size="x-small" class="mr-1">mdi-phone-outline</v-icon>
                <a :href="`tel:${phone.value}`" class="text-decoration-none">{{ phone.value }}</a>
                <v-chip v-if="phone.label" size="x-small" variant="outlined" class="ml-1">{{ phone.label }}</v-chip>
              </div>
            </div>

            <div v-if="!emails.length && !phones.length" class="text-body-2 text-medium-emphasis">
              No contact info
            </div>
          </v-card-text>
        </v-card>

        <!-- Tags Section -->
        <v-card class="mb-4">
          <v-card-title>
            <v-icon start>mdi-tag-multiple</v-icon>
            Tags
          </v-card-title>
          <v-card-text>
            <div v-if="tags.length" class="d-flex flex-wrap ga-1">
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
            <div v-else class="text-body-2 text-medium-emphasis">
              No tags
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

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

const data = window.__INITIAL_DATA__ || {};

const person = data.person || {};
const organizationName: string | null = data.organization_name || null;
const userName: string | null = data.user_name || null;
const activities: any[] = data.activities || [];
const leads: any[] = data.leads || [];
const tags: any[] = data.tags || [];

const permissions: string[] = data.permissions || [];
const canEdit = computed(() => permissions.includes("contacts.persons.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("contacts.persons.delete") || data.permission_type === "all");

const breadcrumbs = computed(() => [
  { title: "Persons", href: "/admin/contacts/persons" },
  { title: person.name || "Person", disabled: true },
]);

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

function formatDateTime(value: string): string {
  if (!value) return "-";
  return new Date(value).toLocaleString();
}

function activityTypeColor(type: string): string {
  switch (type) {
    case "call": return "blue";
    case "meeting": return "purple";
    case "note": return "orange";
    case "task": return "green";
    case "email": return "indigo";
    default: return "grey";
  }
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
