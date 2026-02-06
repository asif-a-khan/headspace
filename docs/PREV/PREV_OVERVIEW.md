# Headspace CRM - System Overview

This document explains how all entities in the CRM interact and the complete business flow.

---

## Core Entity Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                     CONTACT MANAGEMENT                          │
│                                                                 │
│   Organization (Company)                                        │
│        │                                                        │
│        └──► Person (Contact)                                    │
│              │  - name, emails[], contact_numbers[]             │
│              │  - job_title                                     │
│              │  - assigned to User                              │
│              │                                                  │
│              ├──► Leads (sales opportunities)                   │
│              ├──► Quotes (proposals)                            │
│              ├──► Emails (communication)                        │
│              └──► Activities (calls, meetings, notes)           │
└─────────────────────────────────────────────────────────────────┘
```

---

## Sales Pipeline Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      LEAD LIFECYCLE                             │
│                                                                 │
│  Source ──► Lead ──► Pipeline ──► Stages ──► Won/Lost          │
│  (where     (the     (which       (progress   (outcome)         │
│   from)     deal)    process)     steps)                        │
│                                                                 │
│  Example Flow:                                                  │
│  ┌─────────┐    ┌───────────┐    ┌─────────┐    ┌─────────┐    │
│  │ Website │───►│Qualification│──►│Proposal │───►│   Won   │   │
│  │ Referral│    │   (20%)    │    │  (50%)  │    │  (100%) │   │
│  │Cold Call│    └───────────┘    └─────────┘    └─────────┘    │
│  └─────────┘                                    ┌─────────┐    │
│                                                 │  Lost   │    │
│                                                 │(reason) │    │
│                                                 └─────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Entity Descriptions

| Entity | Purpose | Key Relationships |
|--------|---------|-------------------|
| **Person** | Individual contact (customer/prospect) | Belongs to Organization, assigned to User |
| **Organization** | Company/business entity | Has many Persons |
| **Lead** | Sales opportunity with $ value | Has Person, User (owner), Pipeline, Stage, Products, Quotes |
| **Pipeline** | Sales process workflow | Has Stages, tracks "rotten days" (aging) |
| **Stage** | Step in pipeline (e.g., Qualification, Proposal) | Has probability % for forecasting |
| **Quote** | Formal proposal with pricing | Has QuoteItems, linked to Lead(s) and Person |
| **Product** | Catalog item with SKU/price | Tracked in Warehouses, added to Leads/Quotes |
| **Warehouse** | Physical inventory location | Has Locations (shelves/bins) |
| **Activity** | Logged action (call, meeting, note) | Linked to Leads, Persons, Products |
| **User** | System user/employee | Has Role (permissions), assigned Leads/Persons |
| **Tag** | Color-coded label | Applied to Leads, Persons, Products, Emails |

---

## Complete Business Flow

### 1. Contact Creation
```
Create Organization (company) → Create Person (contact) → Assign to User
```

### 2. Lead Management
```
Create Lead
  ├─ Link to Person (who's the customer?)
  ├─ Set Type (Prospect, Customer, Partner)
  ├─ Set Source (Website, Referral, Cold Call)
  ├─ Choose Pipeline (sales process)
  ├─ Set Initial Stage (first step)
  ├─ Add Products (optional line items)
  ├─ Set Lead Value ($ forecast)
  └─ Assign to User (salesperson)
```

### 3. Working the Lead
```
Log Activities
  ├─ Calls (with participants)
  ├─ Meetings (scheduled, with attendees)
  ├─ Notes (internal comments)
  └─ Tasks (to-do items)

Send/Receive Emails (threaded conversations)

Progress through Stages
  └─ Each stage has probability % (for forecasting)
```

### 4. Quote Generation
```
Create Quote from Lead
  ├─ Copy Person as recipient
  ├─ Add QuoteItems (products with quantity, price, discounts, tax)
  ├─ Set billing/shipping addresses
  ├─ Calculate totals (sub_total, tax, grand_total)
  └─ Set expiration date
```

### 5. Lead Closure
```
Move to Won Stage
  ├─ Auto-sets closed_at timestamp
  └─ Lead becomes part of customer base

OR

Move to Lost Stage
  ├─ Set lost_reason (why did we lose?)
  └─ Auto-sets closed_at timestamp
```

---

## Database Relationships

### Pivot Tables (Many-to-Many)

| Pivot Table | Connects | Purpose |
|-------------|----------|---------|
| `lead_quotes` | Lead ↔ Quote | Associates quotes with leads |
| `lead_tags` | Lead ↔ Tag | Tags for leads |
| `lead_activities` | Lead ↔ Activity | Activities logged to leads |
| `lead_products` | Lead ↔ Product | Products in leads (qty, price) |
| `person_activities` | Person ↔ Activity | Activities involving persons |
| `person_tags` | Person ↔ Tag | Tags for persons |
| `product_tags` | Product ↔ Tag | Product categorization |
| `product_inventories` | Product ↔ Warehouse | Stock tracking |
| `warehouse_tags` | Warehouse ↔ Tag | Warehouse tags |
| `email_tags` | Email ↔ Tag | Email categorization |
| `user_groups` | User ↔ Group | User group membership |

### Key Foreign Keys

| Table | Foreign Key | References |
|-------|-------------|------------|
| `persons` | `organization_id` | `organizations.id` |
| `persons` | `user_id` | `users.id` |
| `leads` | `person_id` | `persons.id` |
| `leads` | `user_id` | `users.id` |
| `leads` | `lead_pipeline_id` | `lead_pipelines.id` |
| `leads` | `lead_pipeline_stage_id` | `lead_pipeline_stages.id` |
| `leads` | `lead_source_id` | `lead_sources.id` |
| `leads` | `lead_type_id` | `lead_types.id` |
| `quotes` | `person_id` | `persons.id` |
| `quotes` | `user_id` | `users.id` |
| `quote_items` | `quote_id` | `quotes.id` |
| `quote_items` | `product_id` | `products.id` |
| `users` | `role_id` | `roles.id` |

---

## Inventory System

```
Product (SKU, name, base price)
    │
    └──► ProductInventory (stock tracking)
            ├─ Warehouse (which warehouse?)
            ├─ Location (which shelf/bin?)
            ├─ in_stock (total quantity)
            ├─ allocated (reserved)
            └─ onHand = in_stock - allocated (available)
```

### Warehouse Structure
```
Warehouse
  ├─ name, description
  ├─ contact_name
  ├─ contact_emails[] (JSON array)
  ├─ contact_numbers[] (JSON array)
  ├─ contact_address (JSON)
  │
  └──► Locations (shelves/bins)
        ├─ name (unique per warehouse)
        └──► ProductInventory
```

---

## User & Permissions

```
Role (defines permissions)
  │
  ├─ permission_type: 'all' (full access) or 'custom'
  ├─ permissions: JSON array of specific permissions
  │
  └──► User (has one role)
        ├─ name, email, password
        ├─ api_token
        ├─ status (active/inactive)
        └─ Groups (via user_groups pivot)
```

### Permission Check
```php
$user->hasPermission('leads.create')  // Check specific permission
```

---

## Activity Logging

Activities are polymorphic - can be attached to multiple entity types:

```
Activity (call, meeting, note, task)
  │
  ├─ title, type, comment
  ├─ schedule_from, schedule_to (calendar)
  ├─ is_done (completion status)
  ├─ location
  │
  ├──► Participants
  │     ├─ Users (internal team)
  │     └─ Persons (external contacts)
  │
  ├──► Files (attachments)
  │
  └──► Linked to:
        ├─ Leads (lead_activities)
        ├─ Persons (person_activities)
        ├─ Products (product_activities)
        └─ Warehouses (warehouse_activities)
```

### Activity Types
- **Call** - Phone conversation
- **Meeting** - Scheduled meeting
- **Note** - Internal comment
- **Task** - To-do item

---

## Email Integration

```
Email Thread Support:
  Email A (incoming from customer)
    ├─ Email B (your reply) - parent_id = A
    ├─ Email C (customer reply) - parent_id = A
    └─ Linked to Lead for sales context

Email Fields:
  ├─ subject, source
  ├─ from, sender (JSON)
  ├─ reply_to, cc, bcc (JSON arrays)
  ├─ folders (JSON - Gmail labels, etc.)
  ├─ is_read (tracking)
  ├─ unique_id, message_id (external system IDs)
  ├─ person_id (correspondent)
  └─ lead_id (sales context)
```

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Multi-Pipeline** | Different sales processes for different deal types |
| **Rotten Days** | Tracks how long leads sit without progress |
| **Stage Probability** | Weighted forecasting (lead_value × probability) |
| **Custom Attributes** | Add custom fields to Leads, Quotes, Products |
| **Tagging System** | Color-coded tags for any entity |
| **Activity Logging** | Comprehensive audit trail |
| **Email Threading** | Conversation history with customers |
| **Quote Management** | Full pricing with discounts, taxes |
| **Inventory Tracking** | Stock levels by warehouse/location |
| **Role-Based Access** | Permission system for team management |
| **Data Import/Export** | CSV import for bulk data operations |

---

## Pipeline Configuration

### Pipeline Settings
```
Pipeline
  ├─ name (e.g., "Enterprise Sales", "SMB", "Partner")
  ├─ rotten_days (30 = leads older than 30 days are "rotten")
  ├─ is_default (default pipeline for new leads)
  │
  └──► Stages (ordered by sort_order)
        ├─ code (system identifier: 'new', 'won', 'lost')
        ├─ name (display name)
        ├─ probability (win % for forecasting)
        └─ sort_order (display order)
```

### Default Stages Example
| Code | Name | Probability |
|------|------|-------------|
| `new` | New | 10% |
| `qualification` | Qualification | 20% |
| `proposal` | Proposal | 50% |
| `negotiation` | Negotiation | 75% |
| `won` | Won | 100% |
| `lost` | Lost | 0% |

### Rotten Days Calculation
```
rotten_days = lead.created_at + pipeline.rotten_days - today

If negative → Lead is "rotten" (stalled, needs attention)
```

---

## Quote System

### Quote Structure
```
Quote
  ├─ subject, description
  ├─ billing_address (JSON)
  ├─ shipping_address (JSON)
  ├─ discount_percent, discount_amount
  ├─ tax_amount
  ├─ adjustment_amount
  ├─ sub_total, grand_total
  ├─ expired_at
  ├─ user_id (owner)
  ├─ person_id (customer)
  │
  └──► QuoteItems
        ├─ sku, name
        ├─ quantity, price
        ├─ discount_percent, discount_amount
        ├─ tax_percent, tax_amount
        ├─ total
        └─ product_id (optional link to catalog)
```

### Quote-Lead Relationship
- One Quote can be linked to multiple Leads
- One Lead can have multiple Quotes
- Linked via `lead_quotes` pivot table

---

## Multi-Tenant (SaaS) Architecture

```
Super Admin Panel (headspace.local)
  │
  ├──► Manage Tenants (Companies)
  ├──► System Configuration
  └──► Super Admin Users

Tenant Panel (tenant.headspace.local)
  │
  ├──► Own Users, Roles
  ├──► Own Leads, Persons, Organizations
  ├──► Own Products, Quotes, Activities
  └──► Data isolated by company_id
```

### Tenant Isolation
Every table includes `company_id` foreign key for data isolation:
- `leads.company_id`
- `persons.company_id`
- `products.company_id`
- etc.

---

## Model File Locations

### Contact Package
- `packages/Webkul/Contact/src/Models/Person.php`
- `packages/Webkul/Contact/src/Models/Organization.php`

### Lead Package
- `packages/Webkul/Lead/src/Models/Lead.php`
- `packages/Webkul/Lead/src/Models/Pipeline.php`
- `packages/Webkul/Lead/src/Models/Stage.php`
- `packages/Webkul/Lead/src/Models/Source.php`
- `packages/Webkul/Lead/src/Models/Type.php`

### Quote Package
- `packages/Webkul/Quote/src/Models/Quote.php`
- `packages/Webkul/Quote/src/Models/QuoteItem.php`

### Product Package
- `packages/Webkul/Product/src/Models/Product.php`
- `packages/Webkul/Product/src/Models/ProductInventory.php`

### Warehouse Package
- `packages/Webkul/Warehouse/src/Models/Warehouse.php`
- `packages/Webkul/Warehouse/src/Models/Location.php`

### User Package
- `packages/Webkul/User/src/Models/User.php`
- `packages/Webkul/User/src/Models/Role.php`

### Activity Package
- `packages/Webkul/Activity/src/Models/Activity.php`
- `packages/Webkul/Activity/src/Models/Participant.php`
- `packages/Webkul/Activity/src/Models/File.php`

### Email Package
- `packages/Webkul/Email/src/Models/Email.php`

### Tag Package
- `packages/Webkul/Tag/src/Models/Tag.php`

---

## Repository Pattern

Business logic is encapsulated in Repository classes:

| Repository | Key Methods |
|------------|-------------|
| `LeadRepository` | `create()`, `update()`, `getLeadsQuery()` |
| `QuoteRepository` | `create()`, `update()`, `getQuotesCount()` |
| `PersonRepository` | `create()`, `update()` |
| `ProductRepository` | `create()`, `update()` |
| `ActivityRepository` | `create()`, `update()` |

### Example: Lead Creation
```php
$leadRepository->create([
    'title' => 'New Deal',
    'lead_value' => 10000,
    'person' => ['name' => 'John Doe', 'emails' => ['john@example.com']],
    'lead_source_id' => 1,
    'lead_type_id' => 1,
    'lead_pipeline_id' => 1,
    'lead_pipeline_stage_id' => 1,
]);
```

---

## Summary

This CRM is designed for B2B sales teams to:

1. **Track Contacts** - Manage persons and organizations
2. **Manage Leads** - Track sales opportunities through pipelines
3. **Log Activities** - Record calls, meetings, notes
4. **Send Quotes** - Generate formal proposals
5. **Track Inventory** - Manage products in warehouses
6. **Collaborate** - Team-based access with roles/permissions
7. **Communicate** - Email integration with threading

The multi-tenant architecture allows each company to have isolated data while sharing the same application infrastructure.
