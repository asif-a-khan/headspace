<template>
  <div>
    <h1 class="text-h5 mb-4">{{ isEdit ? "Edit Lead" : "Create Lead" }}</h1>

    <v-card>
      <v-tabs v-model="activeTab" color="primary">
        <v-tab value="details">Details</v-tab>
        <v-tab value="contact">Contact Person</v-tab>
        <v-tab value="products">Products ({{ formProducts.length }})</v-tab>
      </v-tabs>

      <v-card-text>
        <v-form ref="formRef" @submit.prevent="submit">
          <!-- Details Tab -->
          <div v-show="activeTab === 'details'">
            <v-text-field
              v-model="form.title"
              label="Title"
              :rules="[rules.required]"
              class="mb-4"
            />

            <v-textarea
              v-model="form.description"
              label="Description"
              rows="3"
              class="mb-4"
            />

            <div class="d-flex ga-3 mb-3">
              <v-text-field
                v-model="form.lead_value"
                label="Lead Value"
                type="number"
                step="0.01"
                prefix="$"
              />
              <v-text-field
                v-model="form.expected_close_date"
                label="Expected Close Date"
                type="date"
              />
            </div>

            <div class="d-flex ga-3 mb-3">
              <v-select
                v-model="form.lead_pipeline_id"
                :items="pipelineItems"
                item-title="name"
                item-value="id"
                label="Pipeline"
                :rules="[rules.required]"
                @update:model-value="onPipelineChange"
              />
              <v-select
                v-model="form.lead_pipeline_stage_id"
                :items="filteredStages"
                item-title="stage_name"
                item-value="id"
                label="Stage"
                :rules="[rules.required]"
              />
            </div>

            <div class="d-flex ga-3 mb-3">
              <v-select
                v-model="form.lead_source_id"
                :items="sourceItems"
                item-title="name"
                item-value="id"
                label="Source"
                clearable
              />
              <v-select
                v-model="form.lead_type_id"
                :items="typeItems"
                item-title="name"
                item-value="id"
                label="Type"
                clearable
              />
            </div>

            <v-select
              v-model="form.user_id"
              :items="userItems"
              item-title="label"
              item-value="id"
              label="Assigned To"
              clearable
              class="mb-4"
            />
          </div>

          <!-- Contact Person Tab -->
          <div v-show="activeTab === 'contact'">
            <div class="d-flex align-center mb-4">
              <v-combobox
                v-model="selectedPerson"
                :items="personSearchResults"
                item-title="label"
                item-value="value"
                label="Search Contact Person"
                density="compact"
                clearable
                return-object
                :loading="personSearching"
                @update:search="onPersonSearch"
                @update:model-value="onPersonSelected"
                no-filter
                hide-no-data
                style="max-width: 400px"
                class="mr-3"
              />
              <v-btn
                variant="tonal"
                size="small"
                prepend-icon="mdi-plus"
                @click="showCreatePersonDialog = true"
              >
                Create New Person
              </v-btn>
            </div>

            <!-- Selected person info -->
            <v-card v-if="selectedPersonData" variant="outlined" class="mb-4">
              <v-card-text>
                <div class="text-subtitle-1 font-weight-medium mb-1">{{ selectedPersonData.name }}</div>
                <div v-if="selectedPersonData.job_title" class="text-body-2 text-medium-emphasis mb-2">
                  {{ selectedPersonData.job_title }}
                </div>
                <div v-if="selectedPersonData.organization_name" class="text-body-2 mb-2">
                  <v-icon size="x-small" class="mr-1">mdi-domain</v-icon>
                  {{ selectedPersonData.organization_name }}
                </div>

                <div v-if="personEmails.length" class="mb-2">
                  <div class="text-caption text-medium-emphasis mb-1">Email</div>
                  <div v-for="(email, i) in personEmails" :key="i" class="text-body-2">
                    <v-icon size="x-small" class="mr-1">mdi-email-outline</v-icon>
                    {{ email.value }}
                    <v-chip v-if="email.label" size="x-small" variant="outlined" class="ml-1">{{ email.label }}</v-chip>
                  </div>
                </div>

                <div v-if="personPhones.length">
                  <div class="text-caption text-medium-emphasis mb-1">Phone</div>
                  <div v-for="(phone, i) in personPhones" :key="i" class="text-body-2">
                    <v-icon size="x-small" class="mr-1">mdi-phone-outline</v-icon>
                    {{ phone.value }}
                    <v-chip v-if="phone.label" size="x-small" variant="outlined" class="ml-1">{{ phone.label }}</v-chip>
                  </div>
                </div>
              </v-card-text>
            </v-card>

            <div v-else class="text-center text-medium-emphasis py-8">
              <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-account-search</v-icon>
              <div>Search for an existing contact or create a new one</div>
            </div>
          </div>

          <!-- Products Tab -->
          <div v-show="activeTab === 'products'">
            <div class="d-flex align-center mb-4">
              <span class="text-subtitle-1">Lead Products</span>
              <v-spacer />
              <v-btn size="small" variant="tonal" prepend-icon="mdi-plus" @click="addProductRow">
                Add Product
              </v-btn>
            </div>

            <v-table v-if="formProducts.length" density="compact">
              <thead>
                <tr>
                  <th>Product</th>
                  <th style="width: 80px">Qty</th>
                  <th style="width: 120px">Price</th>
                  <th style="width: 120px">Amount</th>
                  <th style="width: 50px"></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(item, i) in formProducts" :key="i">
                  <td>
                    <v-autocomplete
                      v-model="item.product_id"
                      :items="productOptions"
                      item-title="label"
                      item-value="id"
                      density="compact"
                      hide-details
                      clearable
                      @update:model-value="onProductSelect(item)"
                    />
                  </td>
                  <td>
                    <v-text-field
                      v-model.number="item.quantity"
                      type="number"
                      density="compact"
                      hide-details
                      min="1"
                      @update:model-value="calcAmount(item)"
                    />
                  </td>
                  <td>
                    <v-text-field
                      v-model.number="item.price"
                      type="number"
                      step="0.01"
                      density="compact"
                      hide-details
                      prefix="$"
                      @update:model-value="calcAmount(item)"
                    />
                  </td>
                  <td class="font-weight-medium">
                    ${{ (item.amount || 0).toFixed(2) }}
                  </td>
                  <td>
                    <v-btn icon="mdi-close" size="x-small" variant="text" color="error" @click="removeProductRow(i)" />
                  </td>
                </tr>
              </tbody>
              <tfoot v-if="formProducts.length > 1">
                <tr>
                  <td colspan="3" class="text-right font-weight-bold">Total</td>
                  <td class="font-weight-bold">${{ productsTotal }}</td>
                  <td></td>
                </tr>
              </tfoot>
            </v-table>

            <div v-else class="text-center text-medium-emphasis py-8">
              <v-icon size="48" color="grey-lighten-1" class="mb-2">mdi-package-variant</v-icon>
              <div>No products added. Click "Add Product" to start.</div>
            </div>
          </div>
        </v-form>
      </v-card-text>

      <v-card-actions class="px-4 pb-4">
        <v-btn href="/admin/leads" variant="outlined">Cancel</v-btn>
        <v-spacer />
        <v-btn color="primary" :loading="saving" @click="submit">
          {{ isEdit ? "Update" : "Create" }}
        </v-btn>
      </v-card-actions>
    </v-card>

    <!-- Create Person Dialog -->
    <v-dialog v-model="showCreatePersonDialog" max-width="500">
      <v-card>
        <v-card-title>Create New Person</v-card-title>
        <v-card-text>
          <v-text-field
            v-model="newPerson.name"
            label="Name"
            :rules="[rules.required]"
            class="mb-4"
          />
          <v-text-field
            v-model="newPerson.email"
            label="Email"
            type="email"
            class="mb-4"
          />
          <v-text-field
            v-model="newPerson.phone"
            label="Phone"
            class="mb-4"
          />
          <v-select
            v-model="newPerson.organization_id"
            :items="organizationItems"
            item-title="name"
            item-value="id"
            label="Organization"
            clearable
          />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showCreatePersonDialog = false">Cancel</v-btn>
          <v-btn color="primary" :loading="creatingPerson" :disabled="!newPerson.name" @click="createPerson">
            Create
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="errorSnackbar" color="error" :timeout="4000">
      {{ errorMessage }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { useLeadsStore } from "@/stores/admin/leads";
import { get, post } from "@/api/client";

const data = window.__INITIAL_DATA__ || {};
const store = useLeadsStore();
const isEdit = computed(() => !!data.lead);

const activeTab = ref("details");

const pipelineItems = computed(() => data.pipelines || []);
const allStages = computed(() => data.stages || []);
const sourceItems = computed(() => data.sources || []);
const typeItems = computed(() => data.types || []);
const organizationItems = computed(() => data.organizations || []);
const userItems = computed(() =>
  (data.users || []).map((u: any) => ({
    id: u.id,
    label: `${u.first_name} ${u.last_name}`,
  }))
);

const productOptions = computed(() =>
  (data.all_products || []).map((p: any) => ({
    id: p.id,
    label: `${p.sku} - ${p.name}`,
    price: Number(p.price),
  }))
);

const lead = data.lead;
const form = reactive({
  title: lead?.title || "",
  description: lead?.description || "",
  lead_value: lead?.lead_value || "",
  expected_close_date: lead?.expected_close_date || "",
  person_id: lead?.person_id || null as number | null,
  lead_pipeline_id: lead?.lead_pipeline_id || null,
  lead_pipeline_stage_id: lead?.lead_pipeline_stage_id || null,
  lead_source_id: lead?.lead_source_id || null,
  lead_type_id: lead?.lead_type_id || null,
  user_id: lead?.user_id || null,
});

// If no pipeline selected, pick default
if (!form.lead_pipeline_id && pipelineItems.value.length) {
  const defaultPipeline = pipelineItems.value.find((p: any) => p.is_default) || pipelineItems.value[0];
  form.lead_pipeline_id = defaultPipeline.id;
}

const filteredStages = computed(() =>
  allStages.value.filter((s: any) => s.lead_pipeline_id === form.lead_pipeline_id)
);

if (!form.lead_pipeline_stage_id && filteredStages.value.length) {
  form.lead_pipeline_stage_id = filteredStages.value[0].id;
}

function onPipelineChange() {
  const stages = filteredStages.value;
  form.lead_pipeline_stage_id = stages.length ? stages[0].id : null;
}

// --- Contact Person ---
interface PersonSearchItem { label: string; value: number }

const personSearchResults = ref<PersonSearchItem[]>([]);
const personSearching = ref(false);
const selectedPerson = ref<PersonSearchItem | null>(null);
const selectedPersonData = ref<any>(null);
let personSearchTimer: ReturnType<typeof setTimeout> | null = null;

// Initialize selected person from existing lead data
if (form.person_id) {
  const existingPerson = (data.persons || []).find((p: any) => p.id === form.person_id);
  if (existingPerson) {
    selectedPerson.value = { label: existingPerson.name, value: existingPerson.id };
    selectedPersonData.value = {
      ...existingPerson,
      organization_name: (data.organizations || []).find((o: any) => o.id === existingPerson.organization_id)?.name,
    };
  }
}

function onPersonSearch(val: string) {
  if (personSearchTimer) clearTimeout(personSearchTimer);
  if (!val || val.length < 2) {
    personSearchResults.value = [];
    return;
  }
  personSearchTimer = setTimeout(async () => {
    personSearching.value = true;
    try {
      const res = await get<{ data: Array<{ id: number; name: string }> }>(
        `/admin/api/contacts/persons/search?q=${encodeURIComponent(val)}`
      );
      personSearchResults.value = res.data.map((p) => ({ label: p.name, value: p.id }));
    } catch {
      personSearchResults.value = [];
    } finally {
      personSearching.value = false;
    }
  }, 300);
}

async function onPersonSelected(item: PersonSearchItem | string | null) {
  if (!item || typeof item === "string") {
    form.person_id = null;
    selectedPersonData.value = null;
    return;
  }
  form.person_id = item.value;
  // Fetch full person details
  try {
    const res = await get<{ data: any }>(`/admin/api/contacts/persons/${item.value}`);
    const person = res.data;
    const orgName = (data.organizations || []).find((o: any) => o.id === person.organization_id)?.name;
    selectedPersonData.value = { ...person, organization_name: orgName };
  } catch {
    selectedPersonData.value = { name: item.label };
  }
}

const personEmails = computed(() => {
  if (!selectedPersonData.value?.emails) return [];
  const e = selectedPersonData.value.emails;
  if (typeof e === "string") { try { return JSON.parse(e); } catch { return []; } }
  return Array.isArray(e) ? e.filter((x: any) => x.value) : [];
});

const personPhones = computed(() => {
  if (!selectedPersonData.value?.contact_numbers) return [];
  const c = selectedPersonData.value.contact_numbers;
  if (typeof c === "string") { try { return JSON.parse(c); } catch { return []; } }
  return Array.isArray(c) ? c.filter((x: any) => x.value) : [];
});

// --- Create Person Dialog ---
const showCreatePersonDialog = ref(false);
const creatingPerson = ref(false);
const newPerson = reactive({
  name: "",
  email: "",
  phone: "",
  organization_id: null as number | null,
});

async function createPerson() {
  creatingPerson.value = true;
  try {
    const emails = newPerson.email ? [{ value: newPerson.email, label: "work" }] : [];
    const phones = newPerson.phone ? [{ value: newPerson.phone, label: "work" }] : [];
    const res = await post<{ data: any }>("/admin/api/contacts/persons", {
      name: newPerson.name,
      emails,
      contact_numbers: phones,
      organization_id: newPerson.organization_id,
    });
    const created = res.data;
    form.person_id = created.id;
    const orgName = (data.organizations || []).find((o: any) => o.id === created.organization_id)?.name;
    selectedPerson.value = { label: created.name, value: created.id };
    selectedPersonData.value = { ...created, organization_name: orgName };
    showCreatePersonDialog.value = false;
    newPerson.name = "";
    newPerson.email = "";
    newPerson.phone = "";
    newPerson.organization_id = null;
  } catch (err: any) {
    errorMessage.value = err.message || "Failed to create person.";
    errorSnackbar.value = true;
  } finally {
    creatingPerson.value = false;
  }
}

// --- Products ---
interface ProductLine {
  product_id: number | null;
  quantity: number;
  price: number;
  amount: number;
}

const formProducts = ref<ProductLine[]>(
  (data.lead_products || []).map((lp: any) => ({
    product_id: lp.product_id,
    quantity: lp.quantity,
    price: Number(lp.price),
    amount: Number(lp.amount),
  }))
);

function addProductRow() {
  formProducts.value.push({ product_id: null, quantity: 1, price: 0, amount: 0 });
}

function removeProductRow(index: number) {
  formProducts.value.splice(index, 1);
}

function onProductSelect(item: ProductLine) {
  const product = productOptions.value.find((p: any) => p.id === item.product_id);
  if (product) {
    item.price = product.price;
    calcAmount(item);
  }
}

function calcAmount(item: ProductLine) {
  item.amount = (item.quantity || 0) * (item.price || 0);
}

const productsTotal = computed(() => {
  const total = formProducts.value.reduce((sum, p) => sum + (p.amount || 0), 0);
  return total.toFixed(2);
});

// --- Form submission ---
const rules = {
  required: (v: any) => !!v || "Required",
};

const formRef = ref<any>(null);
const saving = ref(false);
const errorSnackbar = ref(false);
const errorMessage = ref("");

async function submit() {
  // Switch to details tab for validation
  activeTab.value = "details";
  await new Promise((r) => setTimeout(r, 50));
  const { valid } = await formRef.value?.validate();
  if (!valid) return;

  saving.value = true;
  try {
    const products = formProducts.value
      .filter((p) => p.product_id)
      .map((p) => ({
        product_id: p.product_id!,
        quantity: p.quantity,
        price: p.price,
      }));

    const payload = {
      title: form.title,
      description: form.description || null,
      lead_value: form.lead_value ? Number(form.lead_value) : null,
      expected_close_date: form.expected_close_date || null,
      person_id: form.person_id || null,
      lead_pipeline_id: form.lead_pipeline_id,
      lead_pipeline_stage_id: form.lead_pipeline_stage_id,
      lead_source_id: form.lead_source_id || null,
      lead_type_id: form.lead_type_id || null,
      user_id: form.user_id || null,
      products,
    };

    if (isEdit.value) {
      await store.update(lead.id, payload);
    } else {
      await store.create(payload);
    }
    window.location.href = "/admin/leads";
  } catch (err: any) {
    errorMessage.value = err.message || "An error occurred.";
    errorSnackbar.value = true;
  } finally {
    saving.value = false;
  }
}
</script>
