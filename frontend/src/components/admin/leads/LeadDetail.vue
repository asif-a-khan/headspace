<template>
  <div>
    <v-breadcrumbs :items="breadcrumbs" class="px-0 pt-0" />

    <!-- Stage Progress Bar -->
    <v-card class="mb-4">
      <v-card-text class="d-flex align-center flex-wrap ga-1 py-3">
        <v-chip
          v-for="stage in stages"
          :key="stage.id"
          :color="stage.id === lead.lead_pipeline_stage_id ? 'primary' : 'default'"
          :variant="stage.id === lead.lead_pipeline_stage_id ? 'elevated' : 'tonal'"
          class="font-weight-medium"
          @click="moveToStage(stage.id)"
        >
          {{ stage.stage_name }}
          <span class="text-caption ml-1">({{ stage.probability }}%)</span>
        </v-chip>

        <v-divider vertical class="mx-2" />

        <v-btn
          v-if="lead.status === null"
          color="success"
          size="small"
          variant="tonal"
          prepend-icon="mdi-trophy"
          :loading="markingWon"
          @click="markWon"
        >
          Won
        </v-btn>
        <v-btn
          v-if="lead.status === null"
          color="error"
          size="small"
          variant="tonal"
          prepend-icon="mdi-close-circle"
          class="ml-1"
          @click="showLostDialog = true"
        >
          Lost
        </v-btn>

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

    <v-row>
      <!-- Left Column: Lead Info -->
      <v-col cols="12" md="8">
        <!-- Lead Info Card -->
        <v-card class="mb-4">
          <v-card-text>
            <div class="d-flex align-center mb-4">
              <div>
                <h1 class="text-h5 font-weight-bold">{{ lead.title }}</h1>
                <div class="text-caption text-medium-emphasis mt-1">
                  Lead #{{ lead.id }}
                  <span v-if="pipelineName" class="ml-2">
                    <v-icon size="x-small" class="mr-1">mdi-pipe</v-icon>{{ pipelineName }}
                  </span>
                </div>
              </div>
              <v-spacer />
              <v-chip
                v-if="lead.status === null"
                color="info"
                variant="tonal"
                size="small"
              >
                Open
              </v-chip>
              <v-chip
                v-else-if="lead.status === true"
                color="success"
                variant="tonal"
                size="small"
              >
                Won
              </v-chip>
              <v-chip
                v-else
                color="error"
                variant="tonal"
                size="small"
              >
                Lost
              </v-chip>
            </div>

            <div v-if="lead.description" class="text-body-1 mb-4">
              {{ lead.description }}
            </div>

            <v-divider class="mb-4" />

            <v-row dense>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Value</div>
                <div class="text-body-1 font-weight-medium">
                  {{ lead.lead_value ? `$${Number(lead.lead_value).toLocaleString()}` : '-' }}
                </div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Expected Close Date</div>
                <div class="text-body-1">
                  {{ lead.expected_close_date ? formatDate(lead.expected_close_date) : '-' }}
                </div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Source</div>
                <div class="text-body-1">{{ sourceName || '-' }}</div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Type</div>
                <div class="text-body-1">{{ typeName || '-' }}</div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Assigned To</div>
                <div class="text-body-1">{{ userName || '-' }}</div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Pipeline</div>
                <div class="text-body-1">{{ pipelineName || '-' }}</div>
              </v-col>
              <v-col v-if="lead.closed_at" cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Closed At</div>
                <div class="text-body-1">{{ formatDateTime(lead.closed_at) }}</div>
              </v-col>
              <v-col v-if="lead.lost_reason" cols="12">
                <div class="text-caption text-medium-emphasis">Lost Reason</div>
                <div class="text-body-1">{{ lead.lost_reason }}</div>
              </v-col>
            </v-row>

            <v-divider class="my-4" />

            <v-row dense>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Created</div>
                <div class="text-body-2">{{ formatDateTime(lead.created_at) }}</div>
              </v-col>
              <v-col cols="12" sm="6">
                <div class="text-caption text-medium-emphasis">Updated</div>
                <div class="text-body-2">{{ formatDateTime(lead.updated_at) }}</div>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>

        <!-- Products Section -->
        <v-card class="mb-4">
          <v-card-title class="d-flex align-center">
            <v-icon start>mdi-package-variant</v-icon>
            Products
            <v-chip size="x-small" variant="tonal" class="ml-2">{{ leadProducts.length }}</v-chip>
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
          </v-card-title>
          <v-card-text v-if="leadProducts.length">
            <v-table density="compact">
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
          </v-card-text>
          <v-card-text v-else class="text-center text-medium-emphasis py-8">
            <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-package-variant</v-icon>
            <div>No products attached</div>
          </v-card-text>
        </v-card>

        <!-- Quotes Section -->
        <v-card class="mb-4">
          <v-card-title class="d-flex align-center">
            <v-icon start>mdi-file-document-outline</v-icon>
            Quotes
            <v-chip size="x-small" variant="tonal" class="ml-2">{{ leadQuotes.length }}</v-chip>
            <v-spacer />
            <v-btn
              v-if="canEdit"
              size="small"
              variant="text"
              prepend-icon="mdi-link-plus"
              @click="showLinkQuoteDialog = true"
            >
              Link Quote
            </v-btn>
          </v-card-title>
          <v-card-text v-if="leadQuotes.length">
            <v-list density="compact">
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
          </v-card-text>
          <v-card-text v-else class="text-center text-medium-emphasis py-8">
            <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-file-document-outline</v-icon>
            <div>No quotes linked</div>
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
                  <span v-if="activity.comment" class="ml-2 text-medium-emphasis">
                    {{ activity.comment }}
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
              :href="`/admin/leads/${lead.id}/edit`"
            >
              Edit Lead
            </v-btn>
            <v-btn
              v-if="canDelete"
              color="error"
              variant="outlined"
              block
              prepend-icon="mdi-delete"
              @click="deleteDialog = true"
            >
              Delete Lead
            </v-btn>
          </v-card-text>
        </v-card>

        <!-- Contact Person Card -->
        <v-card v-if="person" class="mb-4">
          <v-card-title>
            <v-icon start>mdi-account</v-icon>
            Contact Person
          </v-card-title>
          <v-card-text>
            <div class="text-subtitle-1 font-weight-medium mb-1">{{ person.name }}</div>
            <div v-if="person.job_title" class="text-body-2 text-medium-emphasis mb-3">
              {{ person.job_title }}
            </div>

            <div v-if="personEmails.length" class="mb-2">
              <div class="text-caption text-medium-emphasis mb-1">Email</div>
              <div v-for="(email, i) in personEmails" :key="i" class="text-body-2">
                <v-icon size="x-small" class="mr-1">mdi-email-outline</v-icon>
                <a :href="`mailto:${email.value}`" class="text-decoration-none">{{ email.value }}</a>
                <v-chip v-if="email.label" size="x-small" variant="outlined" class="ml-1">{{ email.label }}</v-chip>
              </div>
            </div>

            <div v-if="personPhones.length" class="mb-2">
              <div class="text-caption text-medium-emphasis mb-1">Phone</div>
              <div v-for="(phone, i) in personPhones" :key="i" class="text-body-2">
                <v-icon size="x-small" class="mr-1">mdi-phone-outline</v-icon>
                <a :href="`tel:${phone.value}`" class="text-decoration-none">{{ phone.value }}</a>
                <v-chip v-if="phone.label" size="x-small" variant="outlined" class="ml-1">{{ phone.label }}</v-chip>
              </div>
            </div>

            <v-btn
              variant="text"
              size="small"
              color="primary"
              :href="`/admin/contacts/persons/${person.id}/edit`"
              class="mt-2 px-0"
            >
              View Contact
              <v-icon end size="small">mdi-arrow-right</v-icon>
            </v-btn>
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
    <v-dialog v-model="showLinkQuoteDialog" max-width="440">
      <v-card>
        <v-card-title>Link Quote</v-card-title>
        <v-card-text>
          <v-text-field
            v-model.number="linkQuoteId"
            label="Quote ID"
            type="number"
            density="compact"
            hide-details
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showLinkQuoteDialog = false">Cancel</v-btn>
          <v-btn
            color="primary"
            :loading="linkingQuote"
            :disabled="!linkQuoteId"
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

const data = window.__INITIAL_DATA__ || {};

const lead = data.lead || {};
const person = data.person || null;
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

const breadcrumbs = computed(() => [
  { title: "Leads", href: "/admin/leads" },
  { title: `Lead #${lead.id}`, disabled: true },
]);

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

// --- Formatters ---
function formatDate(value: string): string {
  if (!value) return "-";
  return new Date(value).toLocaleDateString();
}

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

// --- Stage move ---
async function moveToStage(stageId: number) {
  if (stageId === lead.lead_pipeline_stage_id) return;
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
    await put(`/admin/api/leads/${lead.id}/status`, { status: "won" });
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
    await put(`/admin/api/leads/${lead.id}/status`, {
      status: "lost",
      lost_reason: lostReason.value || null,
    });
    window.location.reload();
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to mark as lost.";
    errorSnackbar.value = true;
  } finally {
    markingLost.value = false;
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
    // Reload products
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
const linkQuoteId = ref<number | null>(null);

async function doLinkQuote() {
  if (!linkQuoteId.value) return;
  linkingQuote.value = true;
  try {
    await post(`/admin/api/leads/${lead.id}/quotes`, { quote_id: linkQuoteId.value });
    showLinkQuoteDialog.value = false;
    linkQuoteId.value = null;
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
