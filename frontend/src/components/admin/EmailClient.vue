<template>
  <div>
    <div class="d-flex align-center mb-4">
      <h1 class="text-h5">Mail</h1>
      <v-spacer />
      <v-btn
        v-if="imapEnabled"
        variant="outlined"
        prepend-icon="mdi-sync"
        class="mr-2"
        :loading="syncing"
        @click="triggerSync()"
      >
        Sync
      </v-btn>
      <v-btn color="primary" prepend-icon="mdi-pencil" @click="openCompose()">
        Compose
      </v-btn>
    </div>

    <v-row>
      <!-- Folder sidebar -->
      <v-col cols="12" md="3">
        <v-card>
          <v-list density="compact" nav>
            <v-list-item
              v-for="f in folders"
              :key="f.key"
              :prepend-icon="f.icon"
              :title="f.label"
              :active="currentFolder === f.key"
              @click="selectFolder(f.key)"
            >
              <template #append>
                <v-badge v-if="folderCounts[f.key]" :content="folderCounts[f.key]" color="primary" inline />
              </template>
            </v-list-item>
          </v-list>
          <v-divider />
          <v-list-item
            v-if="imapEnabled && imapLastSyncAt"
            prepend-icon="mdi-clock-outline"
            density="compact"
            class="text-medium-emphasis"
          >
            <v-list-item-title class="text-caption">Last sync</v-list-item-title>
            <v-list-item-subtitle class="text-caption">{{ formatSyncTime(imapLastSyncAt) }}</v-list-item-subtitle>
          </v-list-item>
          <v-list-item
            v-if="!smtpConfigured"
            prepend-icon="mdi-alert-circle"
            title="SMTP Not Configured"
            subtitle="Go to Settings > Configuration"
            density="compact"
            class="text-warning"
          />
        </v-card>
      </v-col>

      <!-- Email list / detail view -->
      <v-col cols="12" md="9">
        <!-- Detail view -->
        <v-card v-if="selectedEmail">
          <v-card-title class="d-flex align-center">
            <v-btn icon variant="text" @click="selectedEmail = null" class="mr-2">
              <v-icon>mdi-arrow-left</v-icon>
            </v-btn>
            {{ selectedEmail.subject || '(No Subject)' }}
          </v-card-title>
          <v-divider />
          <v-card-text>
            <div class="d-flex align-center mb-2">
              <strong>{{ selectedEmail.from_name || selectedEmail.from_address }}</strong>
              <span class="text-medium-emphasis ml-2">&lt;{{ selectedEmail.from_address }}&gt;</span>
              <v-spacer />
              <span class="text-medium-emphasis text-caption">
                {{ new Date(selectedEmail.created_at).toLocaleString() }}
              </span>
            </div>
            <div class="text-medium-emphasis text-body-2 mb-3">
              To: {{ formatRecipients(selectedEmail.reply_to) }}
              <span v-if="selectedEmail.cc?.length"> | CC: {{ formatRecipients(selectedEmail.cc) }}</span>
            </div>
            <v-divider class="mb-3" />
            <div v-html="selectedEmail.body" class="email-body" />

            <!-- Thread replies -->
            <template v-if="emailReplies.length">
              <v-divider class="my-4" />
              <div v-for="reply in emailReplies" :key="reply.id" class="mb-4">
                <div class="d-flex align-center mb-1">
                  <strong>{{ reply.from_name || reply.from_address }}</strong>
                  <v-spacer />
                  <span class="text-caption text-medium-emphasis">
                    {{ new Date(reply.created_at).toLocaleString() }}
                  </span>
                </div>
                <div v-html="reply.body" class="email-body" />
              </div>
            </template>
          </v-card-text>
          <v-card-actions>
            <v-btn prepend-icon="mdi-reply" @click="openReply()">Reply</v-btn>
            <v-btn
              prepend-icon="mdi-delete"
              color="error"
              variant="text"
              @click="trashEmail(selectedEmail.id)"
            >
              {{ selectedEmail.folder === 'trash' ? 'Delete Permanently' : 'Move to Trash' }}
            </v-btn>
          </v-card-actions>
        </v-card>

        <!-- List view -->
        <v-card v-else>
          <v-list lines="two">
            <v-list-item v-if="loading" class="text-center">
              <v-progress-circular indeterminate color="primary" />
            </v-list-item>
            <v-list-item v-else-if="!emails.length" class="text-center text-medium-emphasis">
              No emails in {{ currentFolder }}.
            </v-list-item>
            <v-list-item
              v-for="email in emails"
              :key="email.id"
              @click="viewEmail(email.id)"
              :class="{ 'font-weight-bold': !email.is_read }"
            >
              <template #prepend>
                <v-icon :color="email.is_read ? 'grey' : 'primary'" class="mr-2">
                  {{ email.is_read ? 'mdi-email-open' : 'mdi-email' }}
                </v-icon>
              </template>
              <v-list-item-title>{{ email.subject || '(No Subject)' }}</v-list-item-title>
              <v-list-item-subtitle>
                <span v-if="currentFolder === 'sent'">To: {{ formatRecipients(email.reply_to) }}</span>
                <span v-else>{{ email.from_name || email.from_address }}</span>
                <span v-if="email.person_name" class="ml-2 text-primary">{{ email.person_name }}</span>
                <span v-if="email.lead_title" class="ml-2 text-info">{{ email.lead_title }}</span>
              </v-list-item-subtitle>
              <template #append>
                <span class="text-caption text-medium-emphasis">
                  {{ new Date(email.created_at).toLocaleDateString() }}
                </span>
              </template>
            </v-list-item>
          </v-list>
        </v-card>
      </v-col>
    </v-row>

    <!-- Compose dialog -->
    <v-dialog v-model="composeDialog" max-width="700" persistent>
      <v-card>
        <v-card-title class="d-flex align-center">
          {{ isReply ? 'Reply' : 'New Email' }}
          <v-spacer />
          <v-btn icon variant="text" @click="composeDialog = false">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-card-title>
        <v-divider />
        <v-card-text>
          <v-text-field
            v-model="compose.to"
            label="To (comma-separated)"
            variant="outlined"
            density="compact"
            class="mb-2"
          />
          <v-text-field
            v-model="compose.cc"
            label="CC (optional)"
            variant="outlined"
            density="compact"
            class="mb-2"
          />
          <v-text-field
            v-model="compose.subject"
            label="Subject"
            variant="outlined"
            density="compact"
            class="mb-2"
          />
          <v-textarea
            v-model="compose.body"
            label="Message"
            variant="outlined"
            rows="10"
            class="mb-2"
          />
        </v-card-text>
        <v-card-actions>
          <v-btn @click="sendEmail(true)" :loading="sending" variant="text">
            Save Draft
          </v-btn>
          <v-spacer />
          <v-btn @click="composeDialog = false">Cancel</v-btn>
          <v-btn color="primary" prepend-icon="mdi-send" @click="sendEmail(false)" :loading="sending">
            Send
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="snackbar" :color="snackbarColor" :timeout="3000">
      {{ snackbarText }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";

const data = window.__INITIAL_DATA__ || {};
const smtpConfigured = data.smtp_configured || false;
const imapEnabled = data.imap_enabled || false;
const imapLastSyncAt = ref<string>(data.imap_last_sync_at || "");
const folderCounts = reactive<Record<string, number>>(data.folder_counts || {});

interface EmailItem {
  id: number;
  subject: string;
  from_address: string;
  from_name: string;
  reply_to: string[];
  folder: string;
  is_read: boolean;
  person_name?: string;
  lead_title?: string;
  user_name?: string;
  created_at: string;
}

interface EmailDetail {
  id: number;
  subject: string;
  body: string;
  from_address: string;
  from_name: string;
  reply_to: string[];
  cc: string[];
  bcc: string[];
  folder: string;
  is_read: boolean;
  person_id?: number;
  lead_id?: number;
  parent_id?: number;
  created_at: string;
}

const folders = [
  { key: "inbox", label: "Inbox", icon: "mdi-inbox" },
  { key: "sent", label: "Sent", icon: "mdi-send" },
  { key: "draft", label: "Drafts", icon: "mdi-file-edit" },
  { key: "trash", label: "Trash", icon: "mdi-delete" },
];

const currentFolder = ref("inbox");
const emails = ref<EmailItem[]>([]);
const loading = ref(false);
const selectedEmail = ref<EmailDetail | null>(null);
const emailReplies = ref<EmailDetail[]>([]);

const composeDialog = ref(false);
const isReply = ref(false);
const sending = ref(false);
const compose = reactive({
  to: "",
  cc: "",
  subject: "",
  body: "",
  parentId: null as number | null,
});

const syncing = ref(false);

const snackbar = ref(false);
const snackbarText = ref("");
const snackbarColor = ref("success");

function getCsrf(): string {
  return document.querySelector('meta[name="csrf-token"]')?.getAttribute("content") || "";
}

async function fetchEmails() {
  loading.value = true;
  try {
    const resp = await fetch(`/admin/api/emails?folder=${currentFolder.value}`, {
      headers: { "X-CSRF-TOKEN": getCsrf() },
    });
    const result = await resp.json();
    emails.value = result.data || [];
  } catch {
    emails.value = [];
  } finally {
    loading.value = false;
  }
}

function selectFolder(folder: string) {
  currentFolder.value = folder;
  selectedEmail.value = null;
  fetchEmails();
}

async function viewEmail(id: number) {
  try {
    const resp = await fetch(`/admin/api/emails/${id}`, {
      headers: { "X-CSRF-TOKEN": getCsrf() },
    });
    const result = await resp.json();
    selectedEmail.value = result.data;
    emailReplies.value = result.replies || [];
    // Mark as read in list
    const idx = emails.value.findIndex((e) => e.id === id);
    if (idx >= 0) emails.value[idx].is_read = true;
  } catch {
    snackbarText.value = "Failed to load email.";
    snackbarColor.value = "error";
    snackbar.value = true;
  }
}

function openCompose() {
  isReply.value = false;
  compose.to = "";
  compose.cc = "";
  compose.subject = "";
  compose.body = "";
  compose.parentId = null;
  composeDialog.value = true;
}

function openReply() {
  if (!selectedEmail.value) return;
  isReply.value = true;
  compose.to = selectedEmail.value.from_address;
  compose.cc = "";
  compose.subject = selectedEmail.value.subject.startsWith("Re:")
    ? selectedEmail.value.subject
    : `Re: ${selectedEmail.value.subject}`;
  compose.body = `\n\n--- Original Message ---\n${selectedEmail.value.body}`;
  compose.parentId = selectedEmail.value.id;
  composeDialog.value = true;
}

async function sendEmail(isDraft: boolean) {
  sending.value = true;
  try {
    const toList = compose.to
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    const ccList = compose.cc
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);

    const resp = await fetch("/admin/api/emails", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-CSRF-TOKEN": getCsrf(),
      },
      body: JSON.stringify({
        subject: compose.subject,
        body: compose.body,
        to: toList,
        cc: ccList.length ? ccList : undefined,
        is_draft: isDraft,
        parent_id: compose.parentId,
      }),
    });
    const result = await resp.json();

    if (resp.ok) {
      snackbarText.value = result.message;
      snackbarColor.value = "success";
      composeDialog.value = false;
      fetchEmails();
    } else {
      snackbarText.value = result.error || "Failed to send.";
      snackbarColor.value = "error";
    }
    snackbar.value = true;
  } catch {
    snackbarText.value = "Failed to send email.";
    snackbarColor.value = "error";
    snackbar.value = true;
  } finally {
    sending.value = false;
  }
}

async function trashEmail(id: number) {
  try {
    const resp = await fetch(`/admin/api/emails/${id}`, {
      method: "DELETE",
      headers: { "X-CSRF-TOKEN": getCsrf() },
    });
    const result = await resp.json();
    snackbarText.value = result.message || "Done.";
    snackbarColor.value = "success";
    snackbar.value = true;
    selectedEmail.value = null;
    fetchEmails();
  } catch {
    snackbarText.value = "Failed to delete.";
    snackbarColor.value = "error";
    snackbar.value = true;
  }
}

function formatRecipients(list: string[] | undefined): string {
  if (!list || !Array.isArray(list)) return "";
  return list.join(", ");
}

async function triggerSync() {
  syncing.value = true;
  try {
    const resp = await fetch("/admin/api/emails/sync", {
      method: "POST",
      headers: { "X-CSRF-TOKEN": getCsrf() },
    });
    const result = await resp.json();
    if (resp.ok) {
      snackbarText.value = result.message || "Sync started.";
      snackbarColor.value = "success";
      // Refresh the email list after a short delay to give sync time
      setTimeout(() => {
        fetchEmails();
        imapLastSyncAt.value = new Date().toISOString();
      }, 5000);
    } else {
      snackbarText.value = result.error || "Sync failed.";
      snackbarColor.value = "error";
    }
    snackbar.value = true;
  } catch {
    snackbarText.value = "Failed to trigger sync.";
    snackbarColor.value = "error";
    snackbar.value = true;
  } finally {
    syncing.value = false;
  }
}

function formatSyncTime(isoStr: string): string {
  if (!isoStr) return "Never";
  try {
    return new Date(isoStr).toLocaleString();
  } catch {
    return isoStr;
  }
}

onMounted(() => {
  fetchEmails();
});
</script>

<style scoped>
.email-body {
  max-width: 100%;
  overflow-wrap: break-word;
}
.email-body :deep(img) {
  max-width: 100%;
}
</style>
