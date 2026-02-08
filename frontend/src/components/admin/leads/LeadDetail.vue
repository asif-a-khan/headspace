<template>
  <div>
    <div class="lead-detail-layout d-flex ga-4" style="align-items: flex-start">
      <!-- LEFT Sidebar (sticky) -->
      <div class="lead-sidebar">
        <v-card variant="outlined" class="lead-sidebar-card">
          <!-- Tags Section -->
          <div class="pa-4 border-b">
            <div v-if="tags.length" class="d-flex flex-wrap ga-1 mb-2">
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
            <div v-if="lead.rotten_days && lead.rotten_days > 0" class="mb-2">
              <v-chip color="error" size="small" variant="tonal" prepend-icon="mdi-alert-circle">
                Rotten for {{ lead.rotten_days }} day{{ lead.rotten_days === 1 ? '' : 's' }}
              </v-chip>
            </div>

            <!-- Title -->
            <h3 class="font-weight-bold mb-1" style="font-size: 1.125rem; line-height: 1.4">{{ lead.title }}</h3>
            <div v-if="lead.lead_value" class="text-body-2 text-success font-weight-medium mb-2">
              ${{ Number(lead.lead_value).toLocaleString() }}
            </div>

            <!-- Activity Action Buttons -->
            <div class="d-flex flex-wrap ga-2 mt-2">
              <v-btn
                v-if="canComposeMail"
                size="small"
                variant="tonal"
                color="success"
                prepend-icon="mdi-email"
                :href="`/admin/mail/compose?lead_id=${lead.id}`"
              >
                Mail
              </v-btn>
              <v-btn
                v-if="canCreateActivity"
                size="small"
                variant="tonal"
                prepend-icon="mdi-file-plus"
                :href="`/admin/activities/create?lead_id=${lead.id}&type=file`"
              >
                File
              </v-btn>
              <v-btn
                v-if="canCreateActivity"
                size="small"
                variant="tonal"
                color="warning"
                prepend-icon="mdi-note-plus"
                :href="`/admin/activities/create?lead_id=${lead.id}&type=note`"
              >
                Note
              </v-btn>
              <v-btn
                v-if="canCreateActivity"
                size="small"
                variant="tonal"
                color="info"
                prepend-icon="mdi-calendar-plus"
                :href="`/admin/activities/create?lead_id=${lead.id}&type=meeting`"
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
                      :href="`/admin/leads/${lead.id}/edit`"
                      target="_blank"
                      @click.stop
                    />
                  </div>
                </v-expansion-panel-title>
                <v-expansion-panel-text>
                  <div class="d-flex flex-column ga-3">
                    <div>
                      <div class="text-caption text-medium-emphasis">Value</div>
                      <div class="text-body-2">{{ lead.lead_value ? `$${Number(lead.lead_value).toLocaleString()}` : '-' }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Expected Close Date</div>
                      <div class="text-body-2">{{ lead.expected_close_date ? formatDate(lead.expected_close_date) : '-' }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Source</div>
                      <div class="text-body-2">{{ sourceName || '-' }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Type</div>
                      <div class="text-body-2">{{ typeName || '-' }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Pipeline</div>
                      <div class="text-body-2">{{ pipelineName || '-' }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Assigned To</div>
                      <div class="text-body-2">{{ userName || '-' }}</div>
                    </div>
                    <div v-if="lead.closed_at">
                      <div class="text-caption text-medium-emphasis">Closed At</div>
                      <div class="text-body-2">{{ formatDateTime(lead.closed_at) }}</div>
                    </div>
                    <div v-if="lead.lost_reason">
                      <div class="text-caption text-medium-emphasis">Lost Reason</div>
                      <div class="text-body-2">{{ lead.lost_reason }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Created</div>
                      <div class="text-body-2">{{ formatDateTime(lead.created_at) }}</div>
                    </div>
                    <div>
                      <div class="text-caption text-medium-emphasis">Updated</div>
                      <div class="text-body-2">{{ formatDateTime(lead.updated_at) }}</div>
                    </div>
                  </div>
                </v-expansion-panel-text>
              </v-expansion-panel>
            </v-expansion-panels>
          </div>

          <!-- Contact Person -->
          <div v-if="person" class="pa-4">
            <div class="d-flex align-center mb-3">
              <v-avatar color="primary" variant="tonal" size="36" class="mr-2">
                <span class="text-body-2 font-weight-bold">
                  {{ person.name ? person.name.charAt(0).toUpperCase() : '?' }}
                </span>
              </v-avatar>
              <div style="min-width: 0">
                <a :href="`/admin/contacts/persons/${person.id}`" class="text-body-2 font-weight-medium text-decoration-none d-block text-truncate">
                  {{ person.name }}
                </a>
                <div v-if="person.job_title" class="text-caption text-medium-emphasis text-truncate">
                  {{ person.job_title }}
                </div>
              </div>
            </div>

            <div v-if="personEmails.length" class="mb-2">
              <div v-for="(email, i) in personEmails" :key="i" class="text-body-2 mb-1">
                <v-icon size="x-small" class="mr-1">mdi-email-outline</v-icon>
                <a :href="`mailto:${email.value}`" class="text-decoration-none">{{ email.value }}</a>
              </div>
            </div>

            <div v-if="personPhones.length" class="mb-2">
              <div v-for="(phone, i) in personPhones" :key="i" class="text-body-2 mb-1">
                <v-icon size="x-small" class="mr-1">mdi-phone-outline</v-icon>
                <a :href="`tel:${phone.value}`" class="text-decoration-none">{{ phone.value }}</a>
              </div>
            </div>

            <!-- Organization -->
            <div v-if="organization" class="mt-3 pt-3 border-t">
              <div class="text-caption text-medium-emphasis mb-1">Organization</div>
              <a :href="`/admin/contacts/organizations/${organization.id}`" class="text-body-2 font-weight-medium text-decoration-none">
                {{ organization.name }}
              </a>
              <div v-if="organization.address" class="text-caption text-medium-emphasis mt-1">
                {{ organization.address }}
              </div>
            </div>
          </div>
        </v-card>
      </div>

      <!-- RIGHT Content Area -->
      <div class="lead-content" style="flex: 1; min-width: 0">
        <!-- Stage Progress Bar (Krayin arrow style) -->
        <v-card class="mb-4">
          <v-card-text class="d-flex align-center flex-wrap ga-2 py-3 px-4">
            <div class="stage-bar d-flex align-center">
              <div
                v-for="(stage, i) in stages"
                :key="stage.id"
                class="stage-item"
                :class="{
                  'stage-active': stage.id === lead.lead_pipeline_stage_id && lead.status === null,
                  'stage-passed': lead.status === null && getStageOrder(stage.id) < getCurrentStageOrder(),
                  'stage-won': lead.status === true,
                  'stage-lost': lead.status === false && getStageOrder(stage.id) <= getCurrentStageOrder(),
                  'stage-first': i === 0,
                  'stage-last': i === stages.length - 1,
                }"
                @click="moveToStage(stage.id)"
              >
                <span class="stage-label">{{ stage.stage_name }}</span>
              </div>
            </div>

            <v-divider vertical class="mx-2" />

            <template v-if="lead.status === null">
              <v-menu>
                <template #activator="{ props: menuProps }">
                  <v-btn
                    v-bind="menuProps"
                    size="small"
                    variant="tonal"
                    append-icon="mdi-chevron-down"
                  >
                    Won/Lost
                  </v-btn>
                </template>
                <v-list density="compact">
                  <v-list-item @click="markWon">
                    <template #prepend>
                      <v-icon color="success">mdi-trophy</v-icon>
                    </template>
                    <v-list-item-title>Won</v-list-item-title>
                  </v-list-item>
                  <v-list-item @click="showLostDialog = true">
                    <template #prepend>
                      <v-icon color="error">mdi-close-circle</v-icon>
                    </template>
                    <v-list-item-title>Lost</v-list-item-title>
                  </v-list-item>
                </v-list>
              </v-menu>
            </template>

            <v-chip v-if="lead.status === true" color="success" variant="elevated">
              <v-icon start size="small">mdi-trophy</v-icon>
              Won
            </v-chip>
            <v-chip v-if="lead.status === false" color="error" variant="elevated">
              <v-icon start size="small">mdi-close-circle</v-icon>
              Lost
            </v-chip>
          </v-card-text>
        </v-card>

        <!-- Activity Timeline with Extra Tabs -->
        <ActivityTimeline
          :activities="activities"
          :extra-tabs="extraTabs"
        >
          <!-- Description Tab -->
          <template #description>
            <div class="pa-4">
              <div v-if="lead.description" class="text-body-1" style="white-space: pre-line">
                {{ lead.description }}
              </div>
              <div v-else class="text-center py-8 text-medium-emphasis">
                <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-text-box-outline</v-icon>
                <div>No description</div>
              </div>
            </div>
          </template>

          <!-- Products Tab -->
          <template #products>
            <div class="pa-4">
              <div class="d-flex align-center mb-3">
                <span class="text-subtitle-2 font-weight-medium">
                  Products ({{ leadProducts.length }})
                </span>
                <v-spacer />
                <v-btn
                  v-if="canEdit"
                  size="small"
                  variant="text"
                  prepend-icon="mdi-plus"
                  @click="showAddProductDialog = true"
                >
                  Add Product
                </v-btn>
              </div>
              <v-table v-if="leadProducts.length" density="compact">
                <thead>
                  <tr>
                    <th>SKU</th>
                    <th>Product</th>
                    <th class="text-right">Price</th>
                    <th class="text-right">Qty</th>
                    <th class="text-right">Amount</th>
                    <th v-if="canEdit" width="50"></th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="lp in leadProducts" :key="lp.id">
                    <td class="text-caption">{{ lp.product_sku }}</td>
                    <td>{{ lp.product_name }}</td>
                    <td class="text-right">${{ Number(lp.price).toLocaleString() }}</td>
                    <td class="text-right">{{ lp.quantity }}</td>
                    <td class="text-right font-weight-medium">${{ Number(lp.amount).toLocaleString() }}</td>
                    <td v-if="canEdit">
                      <v-btn
                        icon="mdi-close"
                        size="x-small"
                        variant="text"
                        color="error"
                        @click="removeProduct(lp.id)"
                      />
                    </td>
                  </tr>
                </tbody>
                <tfoot v-if="leadProducts.length > 1">
                  <tr>
                    <td colspan="4" class="text-right font-weight-bold">Total</td>
                    <td class="text-right font-weight-bold">${{ productsTotal }}</td>
                    <td v-if="canEdit"></td>
                  </tr>
                </tfoot>
              </v-table>
              <div v-else class="text-center text-medium-emphasis py-8">
                <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-package-variant</v-icon>
                <div>No products attached</div>
              </div>
            </div>
          </template>

          <!-- Quotes Tab -->
          <template #quotes>
            <div class="pa-4">
              <div class="d-flex align-center mb-3">
                <span class="text-subtitle-2 font-weight-medium">
                  Quotes ({{ leadQuotes.length }})
                </span>
                <v-spacer />
                <v-btn
                  v-if="canEdit"
                  size="small"
                  variant="text"
                  prepend-icon="mdi-plus"
                  :href="`/admin/quotes/create?lead_id=${lead.id}`"
                  class="mr-1"
                >
                  Create
                </v-btn>
                <v-btn
                  v-if="canEdit"
                  size="small"
                  variant="text"
                  prepend-icon="mdi-link-plus"
                  @click="showLinkQuoteDialog = true"
                >
                  Link
                </v-btn>
              </div>
              <v-list v-if="leadQuotes.length" density="compact">
                <v-list-item
                  v-for="q in leadQuotes"
                  :key="q.id"
                  :href="`/admin/quotes/${q.id}/edit`"
                >
                  <v-list-item-title>{{ q.subject }}</v-list-item-title>
                  <v-list-item-subtitle>
                    <span v-if="q.grand_total">Total: ${{ Number(q.grand_total).toLocaleString() }}</span>
                    <span v-if="q.expired_at" class="ml-2">Expires: {{ formatDate(q.expired_at) }}</span>
                  </v-list-item-subtitle>
                  <template v-if="canEdit" #append>
                    <v-btn
                      icon="mdi-link-off"
                      size="x-small"
                      variant="text"
                      color="error"
                      @click.prevent="unlinkQuote(q.id)"
                    />
                  </template>
                </v-list-item>
              </v-list>
              <div v-else class="text-center text-medium-emphasis py-8">
                <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-file-document-outline</v-icon>
                <div>No quotes linked</div>
              </div>
            </div>
          </template>
        </ActivityTimeline>
      </div>
    </div>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Lead</v-card-title>
        <v-card-text>
          Are you sure you want to delete "{{ lead.title }}"? This action cannot be undone.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" :loading="deleting" @click="doDelete">Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Lost Reason Dialog -->
    <v-dialog v-model="showLostDialog" max-width="440">
      <v-card>
        <v-card-title>Mark as Lost</v-card-title>
        <v-card-text>
          <v-textarea
            v-model="lostReason"
            label="Lost Reason (optional)"
            rows="3"
            hide-details
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showLostDialog = false">Cancel</v-btn>
          <v-btn color="error" :loading="markingLost" @click="markLost">Mark Lost</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Add Product Dialog -->
    <v-dialog v-model="showAddProductDialog" max-width="500">
      <v-card>
        <v-card-title>Add Product</v-card-title>
        <v-card-text>
          <v-autocomplete
            v-model="newProduct.product_id"
            :items="allProducts"
            item-title="name"
            item-value="id"
            label="Product"
            density="compact"
            class="mb-3"
            hide-details
            @update:model-value="onProductSelect"
          />
          <v-row>
            <v-col cols="6">
              <v-text-field
                v-model.number="newProduct.quantity"
                label="Quantity"
                type="number"
                min="1"
                density="compact"
                hide-details
              />
            </v-col>
            <v-col cols="6">
              <v-text-field
                v-model.number="newProduct.price"
                label="Price"
                type="number"
                min="0"
                step="0.01"
                density="compact"
                hide-details
              />
            </v-col>
          </v-row>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showAddProductDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            :loading="addingProduct"
            :disabled="!newProduct.product_id"
            @click="addProduct"
          >
            Add
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Link Quote Dialog -->
    <v-dialog v-model="showLinkQuoteDialog" max-width="500">
      <v-card>
        <v-card-title>Link Quote</v-card-title>
        <v-card-text>
          <v-combobox
            v-model="selectedQuoteSearch"
            :items="quoteSearchResults"
            :loading="quoteSearchLoading"
            item-title="label"
            item-value="id"
            label="Search quotes by subject..."
            density="compact"
            hide-details
            return-object
            clearable
            @update:search="onQuoteSearch"
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showLinkQuoteDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            :loading="linkingQuote"
            :disabled="!selectedQuoteSearch?.id"
            @click="doLinkQuote"
          >
            Link
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Snackbar -->
    <v-snackbar v-model="errorSnackbar" color="error" :timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { get, post, put, del } from "@/api/client";
import ActivityTimeline from "@/components/admin/shared/ActivityTimeline.vue";

const data = window.__INITIAL_DATA__ || {};

const lead = data.lead || {};
const person = data.person || null;
const organization = data.organization || null;
const activities: any[] = data.activities || [];
const tags: any[] = data.tags || [];
const stages: any[] = data.stages || [];
const pipelineName: string = data.pipeline_name || "";
const sourceName: string = data.source_name || null;
const typeName: string = data.type_name || null;
const userName: string = data.user_name || null;

const leadProducts = ref<any[]>(data.products || []);
const leadQuotes = ref<any[]>(data.quotes || []);

const permissions: string[] = data.permissions || [];
const canEdit = computed(() => permissions.includes("leads.edit") || data.permission_type === "all");
const canDelete = computed(() => permissions.includes("leads.delete") || data.permission_type === "all");
const canComposeMail = computed(() => permissions.includes("mail.compose") || data.permission_type === "all");
const canCreateActivity = computed(() => permissions.includes("activities.create") || data.permission_type === "all");

const extraTabs = [
  { name: "description", label: "Description" },
  { name: "products", label: "Products" },
  { name: "quotes", label: "Quotes" },
];

const productsTotal = computed(() => {
  const total = leadProducts.value.reduce((sum: number, lp: any) => sum + Number(lp.amount || 0), 0);
  return total.toLocaleString();
});

// --- Contact person helpers ---
const personEmails = computed(() => {
  if (!person?.emails) return [];
  if (typeof person.emails === "string") {
    try { return JSON.parse(person.emails); } catch { return []; }
  }
  return person.emails;
});

const personPhones = computed(() => {
  if (!person?.contact_numbers) return [];
  if (typeof person.contact_numbers === "string") {
    try { return JSON.parse(person.contact_numbers); } catch { return []; }
  }
  return person.contact_numbers;
});

// --- Stage helpers ---
function getStageOrder(stageId: number): number {
  const s = stages.find((st: any) => st.id === stageId);
  return s?.sort_order ?? 999;
}

function getCurrentStageOrder(): number {
  return getStageOrder(lead.lead_pipeline_stage_id);
}

// --- Formatters ---
function formatDate(value: string): string {
  if (!value) return "-";
  return new Date(value).toLocaleDateString();
}

function formatDateTime(value: string): string {
  if (!value) return "-";
  return new Date(value).toLocaleString();
}

// --- Stage move (Krayin-aligned: auto won/lost on stage code) ---
const pendingStageId = ref<number | null>(null);

async function moveToStage(stageId: number) {
  if (stageId === lead.lead_pipeline_stage_id) return;
  const target = stages.find((s: any) => s.id === stageId);
  if (!target) return;

  if (target.stage_code === "lost") {
    pendingStageId.value = stageId;
    showLostDialog.value = true;
    return;
  }
  // For 'won' or normal stages — proceed directly via update_stage
  try {
    await put(`/admin/api/leads/${lead.id}/stage`, { lead_pipeline_stage_id: stageId });
    window.location.reload();
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to move stage.";
    errorSnackbar.value = true;
  }
}

// --- Won / Lost ---
const markingWon = ref(false);
const markingLost = ref(false);
const showLostDialog = ref(false);
const lostReason = ref("");

async function markWon() {
  markingWon.value = true;
  try {
    const wonStage = stages.find((s: any) => s.stage_code === "won");
    if (wonStage) {
      await put(`/admin/api/leads/${lead.id}/stage`, { lead_pipeline_stage_id: wonStage.id });
    } else {
      await put(`/admin/api/leads/${lead.id}/status`, { status: "won" });
    }
    window.location.reload();
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to mark as won.";
    errorSnackbar.value = true;
  } finally {
    markingWon.value = false;
  }
}

async function markLost() {
  markingLost.value = true;
  try {
    const targetId = pendingStageId.value;
    if (targetId) {
      // Stage bar click path
      await put(`/admin/api/leads/${lead.id}/stage`, {
        lead_pipeline_stage_id: targetId,
        lost_reason: lostReason.value || null,
      });
    } else {
      // Dropdown path — find lost stage and use update_stage
      const lostStage = stages.find((s: any) => s.stage_code === "lost");
      if (lostStage) {
        await put(`/admin/api/leads/${lead.id}/stage`, {
          lead_pipeline_stage_id: lostStage.id,
          lost_reason: lostReason.value || null,
        });
      } else {
        await put(`/admin/api/leads/${lead.id}/status`, {
          status: "lost",
          lost_reason: lostReason.value || null,
        });
      }
    }
    window.location.reload();
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to mark as lost.";
    errorSnackbar.value = true;
  } finally {
    markingLost.value = false;
    pendingStageId.value = null;
  }
}

// --- Products ---
const showAddProductDialog = ref(false);
const addingProduct = ref(false);
const allProducts = ref<any[]>([]);
const newProduct = ref({ product_id: null as number | null, quantity: 1, price: 0 });

async function loadAllProducts() {
  try {
    const res = await get<{ data: any[] }>("/admin/api/products");
    allProducts.value = res.data;
  } catch { /* ignore */ }
}
loadAllProducts();

function onProductSelect(productId: number | null) {
  if (!productId) return;
  const p = allProducts.value.find((x) => x.id === productId);
  if (p) {
    newProduct.value.price = Number(p.price) || 0;
  }
}

async function addProduct() {
  addingProduct.value = true;
  try {
    await post(`/admin/api/leads/${lead.id}/products`, {
      product_id: newProduct.value.product_id,
      quantity: newProduct.value.quantity,
      price: newProduct.value.price,
    });
    showAddProductDialog.value = false;
    newProduct.value = { product_id: null, quantity: 1, price: 0 };
    const res = await get<{ data: any[] }>(`/admin/api/leads/${lead.id}/products`);
    leadProducts.value = res.data;
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to add product.";
    errorSnackbar.value = true;
  } finally {
    addingProduct.value = false;
  }
}

async function removeProduct(lineId: number) {
  try {
    await del(`/admin/api/leads/${lead.id}/products/${lineId}`);
    leadProducts.value = leadProducts.value.filter((lp) => lp.id !== lineId);
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to remove product.";
    errorSnackbar.value = true;
  }
}

// --- Quotes ---
const showLinkQuoteDialog = ref(false);
const linkingQuote = ref(false);
const quoteSearchResults = ref<any[]>([]);
const quoteSearchLoading = ref(false);
const selectedQuoteSearch = ref<any>(null);
let quoteSearchTimer: ReturnType<typeof setTimeout> | null = null;

function onQuoteSearch(val: string) {
  if (quoteSearchTimer) clearTimeout(quoteSearchTimer);
  if (!val || val.length < 2) {
    quoteSearchResults.value = [];
    return;
  }
  quoteSearchTimer = setTimeout(async () => {
    quoteSearchLoading.value = true;
    try {
      const res = await get<{ data: any[] }>(`/admin/api/quotes/search?q=${encodeURIComponent(val)}`);
      quoteSearchResults.value = res.data.map((q: any) => ({
        id: q.id,
        label: `#${q.id} - ${q.subject}${q.grand_total ? ` ($${Number(q.grand_total).toLocaleString()})` : ""}`,
      }));
    } catch { /* ignore */ }
    quoteSearchLoading.value = false;
  }, 300);
}

async function doLinkQuote() {
  const quoteId = selectedQuoteSearch.value?.id;
  if (!quoteId) return;
  linkingQuote.value = true;
  try {
    await post(`/admin/api/leads/${lead.id}/quotes`, { quote_id: quoteId });
    showLinkQuoteDialog.value = false;
    selectedQuoteSearch.value = null;
    quoteSearchResults.value = [];
    const res = await get<{ data: any[] }>(`/admin/api/leads/${lead.id}/quotes`);
    leadQuotes.value = res.data;
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to link quote.";
    errorSnackbar.value = true;
  } finally {
    linkingQuote.value = false;
  }
}

async function unlinkQuote(quoteId: number) {
  try {
    await del(`/admin/api/leads/${lead.id}/quotes/${quoteId}`);
    leadQuotes.value = leadQuotes.value.filter((q) => q.id !== quoteId);
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to unlink quote.";
    errorSnackbar.value = true;
  }
}

// --- Delete ---
const deleteDialog = ref(false);
const deleting = ref(false);

async function doDelete() {
  deleting.value = true;
  try {
    await del(`/admin/api/leads/${lead.id}`);
    window.location.href = "/admin/leads";
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to delete lead.";
    errorSnackbar.value = true;
  } finally {
    deleting.value = false;
  }
}

// --- Error state ---
const errorSnackbar = ref(false);
const errorMessage = ref("");
</script>

<style scoped>
.lead-detail-layout {
  flex-wrap: wrap;
}
.lead-sidebar {
  width: 394px;
  min-width: 394px;
  max-width: 394px;
  flex-shrink: 0;
}
.lead-sidebar-card {
  position: sticky;
  top: 73px;
  align-self: flex-start;
}
.border-b {
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
.border-t {
  border-top: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
/* Stage bar - Krayin arrow-connected style */
.stage-bar {
  gap: 0;
  flex-wrap: nowrap;
  overflow-x: auto;
}
.stage-item {
  position: relative;
  height: 28px;
  display: flex;
  align-items: center;
  padding: 0 16px 0 20px;
  background: #E5E7EB;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s;
  clip-path: polygon(0 0, calc(100% - 10px) 0, 100% 50%, calc(100% - 10px) 100%, 0 100%, 10px 50%);
}
.stage-item.stage-first {
  padding-left: 12px;
  border-radius: 4px 0 0 4px;
  clip-path: polygon(0 0, calc(100% - 10px) 0, 100% 50%, calc(100% - 10px) 100%, 0 100%);
}
.stage-item.stage-last {
  border-radius: 0 4px 4px 0;
  clip-path: polygon(0 0, 100% 0, 100% 100%, 0 100%, 10px 50%);
}
.stage-item.stage-first.stage-last {
  clip-path: none;
  border-radius: 4px;
  padding-left: 12px;
}
.stage-label {
  font-size: 12px;
  font-weight: 500;
  color: #4B5563;
  position: relative;
  z-index: 1;
}
.stage-item.stage-active {
  background: #10B981;
}
.stage-item.stage-active .stage-label {
  color: #fff;
}
.stage-item.stage-passed {
  background: #6EE7B7;
}
.stage-item.stage-passed .stage-label {
  color: #065F46;
}
.stage-item.stage-won {
  background: #10B981;
}
.stage-item.stage-won .stage-label {
  color: #fff;
}
.stage-item.stage-lost {
  background: #EF4444;
}
.stage-item.stage-lost .stage-label {
  color: #fff;
}
.stage-item:hover {
  filter: brightness(0.95);
}
@media (max-width: 960px) {
  .lead-sidebar {
    width: 100%;
    min-width: 100%;
    max-width: 100%;
  }
  .lead-sidebar-card {
    position: static;
  }
}
</style>
