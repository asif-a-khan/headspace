# Headspace CRM Multi-Tenant SaaS - Complete Architecture Document

This document provides an exhaustive bottom-up analysis of the Headspace CRM Multi-Tenant SaaS application. It covers every layer from database schema through business logic to the frontend rendering system. This document serves as the definitive reference for recreating the application.

---

## Table of Contents

1. [Application Overview](#1-application-overview)
2. [Package Architecture](#2-package-architecture)
3. [Database Schema](#3-database-schema)
4. [Multi-Tenant SaaS System](#4-multi-tenant-saas-system)
5. [Routing & Middleware](#5-routing--middleware)
6. [Authentication & Authorization](#6-authentication--authorization)
7. [Service Providers & Bootstrapping](#7-service-providers--bootstrapping)
8. [Business Logic Layer](#8-business-logic-layer)
9. [Custom Attribute System (EAV)](#9-custom-attribute-system-eav)
10. [Event System](#10-event-system)
11. [DataGrid System](#11-datagrid-system)
12. [Data Import/Export](#12-data-importexport)
13. [Email System](#13-email-system)
14. [Automation & Marketing](#14-automation--marketing)
15. [Frontend Architecture](#15-frontend-architecture)
16. [API Layer](#16-api-layer)
17. [Request Lifecycle](#17-request-lifecycle)

---

## 1. Application Overview

### What It Is

A B2B Customer Relationship Management (CRM) system with multi-tenant SaaS architecture. Each tenant (company) gets an isolated workspace accessed via subdomain, while sharing a single database and application deployment.

### Tech Stack (Original)

| Component | Technology |
|-----------|-----------|
| Language | PHP 8.1+ |
| Framework | Laravel 10.x |
| Database | MySQL/MariaDB |
| Frontend | Vue 3 + Laravel Blade |
| CSS | Tailwind CSS 3.3 |
| Build Tool | Vite 5.4 |
| Queue | Database driver (or Redis) |
| Package System | Laravel Concord (modular packages) |

### Core Domain Entities

```
Organizations ──► Persons ──► Leads ──► Quotes
                     │           │
                     ▼           ▼
                Activities   Products
                     │
                     ▼
                  Emails
```

### Access Points

- **Primary Domain** (`headspace.local`) - Super admin panel for managing tenants
- **Tenant Subdomains** (`{tenant}.headspace.local`) - Per-company CRM workspace
- **Custom Domains** (CNAME) - Tenants can use their own domain names

---

## 2. Package Architecture

The application is organized as a collection of Laravel packages using the Concord modular architecture. Each package encapsulates a domain (models, repositories, controllers, views, migrations).

### Package Map

```
packages/Webkul/
├── Activity/          # Activity tracking (calls, meetings, notes, tasks)
├── Admin/             # Admin panel controllers, views, DataGrids, requests
├── Attribute/         # Custom attribute system (EAV pattern)
├── Automation/        # Workflows and webhooks
├── Contact/           # Persons and Organizations
├── Core/              # Core utilities, configuration system, country data
├── DataGrid/          # Dynamic data table component with filter/sort/paginate
├── DataTransfer/      # CSV import/export system
├── Email/             # Email management with threading
├── EmailTemplate/     # Email template CRUD
├── Installer/         # Application installer wizard
├── Lead/              # Leads, pipelines, stages, sources, types
├── Marketing/         # Marketing campaigns and events
├── Product/           # Product catalog and inventory
├── Quote/             # Quotes and quote items
├── SaasHeadspace/        # Multi-tenant SaaS layer (tenant isolation, super admin)
├── Tag/               # Tagging system
├── User/              # Users, roles, groups
├── Warehouse/         # Warehouse and location management
└── WebForm/           # Public web forms for lead capture
```

### Package Internal Structure (Standard)

Each package follows this layout:

```
packages/Webkul/{PackageName}/
├── src/
│   ├── Config/              # Package configuration files
│   ├── Contracts/           # Interface definitions (used for DI)
│   ├── Database/
│   │   ├── Migrations/      # Database migrations
│   │   └── Seeders/         # Data seeders
│   ├── Http/
│   │   ├── Controllers/     # HTTP controllers
│   │   ├── Middleware/       # Package-specific middleware
│   │   └── Requests/        # Form request validation
│   ├── Models/              # Eloquent models + proxy classes
│   ├── Providers/           # Service providers
│   ├── Repositories/        # Business logic (repository pattern)
│   ├── Resources/
│   │   ├── assets/          # CSS, JS, images
│   │   ├── lang/            # Translation files
│   │   └── views/           # Blade templates
│   └── Routes/              # Route definitions
└── composer.json            # Package autoloading
```

### Model Contract/Proxy Pattern

Each model uses a contract (interface) and proxy pattern for dependency injection. This allows the SaaS layer to swap model implementations:

```
1. Contract:     Webkul\Lead\Contracts\Lead         (interface)
2. Base Model:   Webkul\Lead\Models\Lead             (Eloquent model)
3. Proxy:        Webkul\Lead\Models\LeadProxy        (resolves via DI container)
4. SaaS Model:   Webkul\SaasHeadspace\Models\Lead\Lead  (overrides with query scoping)
```

Registration in service provider:
```php
// Base package registers contract → model binding
$this->app->concord->registerModel(
    \Webkul\Lead\Contracts\Lead::class,
    \Webkul\Lead\Models\Lead::class
);

// SaaS package overrides with tenant-scoped model
$this->app->concord->registerModel(
    \Webkul\Lead\Contracts\Lead::class,
    \Webkul\SaasHeadspace\Models\Lead\Lead::class
);
```

---

## 3. Database Schema

### Schema Overview

The database contains **60+ tables** organized across 18 packages plus Laravel framework tables. Every tenant-scoped table includes a `company_id` column for multi-tenant isolation.

### 3.1 Activity Package

#### activities
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| title | string | nullable |
| type | string | nullable |
| comment | text | nullable |
| additional | json | nullable |
| schedule_from | datetime | nullable |
| schedule_to | datetime | nullable |
| is_done | boolean | default(0) |
| location | string | nullable |
| user_id | unsignedInteger | FK → users.id (set null) |
| created_at, updated_at | timestamps | |

#### activity_participants
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| activity_id | unsignedInteger | FK → activities.id (cascade) |
| user_id | unsignedInteger | nullable, FK → users.id (cascade) |
| person_id | unsignedInteger | nullable, FK → persons.id (cascade) |

#### activity_files
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| path | string | required |
| activity_id | unsignedInteger | FK → activities.id (cascade) |

### 3.2 Attribute Package

#### attributes
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| code | string | required |
| name | string | required |
| type | string | required |
| lookup_type | string | nullable (e.g., users, lead_pipelines) |
| entity_type | string | required (leads, persons, organizations, etc.) |
| sort_order | integer | nullable |
| validation | string | nullable (numeric, etc.) |
| is_required | boolean | default(0) |
| is_unique | boolean | default(0) |
| quick_add | boolean | default(0) |
| is_user_defined | boolean | default(1) |

**Attribute types:** text, textarea, price, boolean, select, multiselect, checkbox, email, phone, address, lookup, date, datetime, file, image

#### attribute_options
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | nullable |
| sort_order | integer | nullable |
| attribute_id | unsignedInteger | FK → attributes.id (cascade) |

#### attribute_values
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| entity_type | string | default('leads') |
| text_value | text | nullable |
| boolean_value | boolean | nullable |
| integer_value | integer | nullable |
| float_value | double | nullable |
| datetime_value | datetime | nullable |
| date_value | date | nullable |
| json_value | json | nullable |
| entity_id | unsignedInteger | references the entity row |
| attribute_id | unsignedInteger | FK → attributes.id (cascade) |

### 3.3 Automation Package

#### workflows
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| description | string | nullable |
| entity_type | string | required |
| event | string | required |
| condition_type | string | default('and') |
| conditions | json | nullable |
| actions | json | nullable |

#### webhooks
| Column | Type | Constraints |
|--------|------|-------------|
| id | bigIncrements | PK |
| company_id | unsignedBigInteger | nullable, FK → companies.id (cascade) |
| name | string | required |
| entity_type | string | required |
| description | string | nullable |
| method | string | required (GET, POST, PUT, DELETE) |
| end_point | string | required |
| query_params | json | nullable |
| headers | json | nullable |
| payload_type | string | required |
| raw_payload_type | string | required |
| payload | json | nullable |

### 3.4 Contact Package

#### organizations
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| address | json | nullable |
| user_id | unsignedInteger | nullable, FK → users.id (set null) |

#### persons
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| emails | json | required (array of {value, label}) |
| contact_numbers | json | nullable (array of {value, label}) |
| organization_id | unsignedInteger | nullable, FK → organizations.id (set null) |
| job_title | string | nullable |
| user_id | unsignedInteger | nullable, FK → users.id (set null) |
| unique_id | string | nullable, unique |

#### person_tags (pivot)
- tag_id → tags.id (cascade)
- person_id → persons.id (cascade)

#### person_activities (pivot)
- activity_id → activities.id (cascade)
- person_id → persons.id (cascade)

### 3.5 Core Package

#### core_config
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| code | string | required |
| value | text | required |

#### countries
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| code | string | required |
| name | string | required |

#### country_states
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| country_code | string | required |
| code | string | required |
| name | string | required |
| country_id | unsignedInteger | FK → countries.id (cascade) |

### 3.6 DataGrid Package

#### datagrid_saved_filters
| Column | Type | Constraints |
|--------|------|-------------|
| id | bigIncrements | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| user_id | unsignedInteger | required |
| name | string | required |
| src | string | required (page/source identifier) |
| applied | json | required (filter criteria) |

### 3.7 Data Transfer Package

#### imports
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| state | string | default('pending') |
| process_in_queue | boolean | default(1) |
| type | string | required |
| action | string | required |
| validation_strategy | string | required |
| allowed_errors | integer | default(0) |
| processed_rows_count | integer | default(0) |
| invalid_rows_count | integer | default(0) |
| errors_count | integer | default(0) |
| errors | json | nullable |
| field_separator | string | required |
| file_path | string | required |
| error_file_path | string | nullable |
| summary | json | nullable |
| started_at | datetime | nullable |
| completed_at | datetime | nullable |

#### import_batches
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| state | string | default('pending') |
| data | json | required |
| summary | json | nullable |
| import_id | unsignedInteger | FK → imports.id (cascade) |

### 3.8 Email Package

#### emails
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| subject | string | nullable |
| source | string | required |
| user_type | string | required |
| name | string | nullable |
| reply | text | nullable |
| is_read | boolean | default(0) |
| folders | json | nullable |
| from | json | nullable |
| sender | json | nullable |
| reply_to | json | nullable |
| cc | json | nullable |
| bcc | json | nullable |
| unique_id | string | nullable, unique |
| message_id | string | unique, required |
| reference_ids | json | nullable |
| person_id | unsignedInteger | nullable, FK → persons.id (set null) |
| lead_id | unsignedInteger | nullable, FK → leads.id (set null) |
| parent_id | unsignedInteger | nullable, FK → emails.id (cascade) |

#### email_attachments
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | nullable |
| path | string | required |
| size | integer | nullable |
| content_type | string | nullable |
| content_id | string | nullable |
| email_id | unsignedInteger | FK → emails.id (cascade) |

#### email_tags (pivot)
- tag_id → tags.id (cascade)
- email_id → emails.id (cascade)

### 3.9 Email Template Package

#### email_templates
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| subject | string | required |
| content | text | required |

### 3.10 Lead Package

#### lead_sources
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |

#### lead_types
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |

#### lead_pipelines
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| is_default | boolean | default(0) |
| rotten_days | integer | default(30) |

#### lead_pipeline_stages
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| code | string | nullable |
| name | string | nullable |
| probability | integer | default(0) |
| sort_order | integer | default(0) |
| lead_pipeline_id | unsignedInteger | FK → lead_pipelines.id (cascade) |

#### leads
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| title | string | required |
| description | text | nullable |
| lead_value | decimal(12,4) | nullable |
| status | boolean | nullable |
| lost_reason | text | nullable |
| closed_at | datetime | nullable |
| expected_close_date | date | nullable |
| user_id | unsignedInteger | nullable, FK → users.id (set null) |
| person_id | unsignedInteger | nullable, FK → persons.id (restrict) |
| lead_source_id | unsignedInteger | nullable, FK → lead_sources.id (restrict) |
| lead_type_id | unsignedInteger | nullable, FK → lead_types.id (restrict) |
| lead_pipeline_id | unsignedInteger | nullable, FK → lead_pipelines.id (cascade) |
| lead_pipeline_stage_id | unsignedInteger | nullable, FK → lead_pipeline_stages.id (set null) |

**Note on cascade rules:** `person_id`, `lead_source_id`, and `lead_type_id` use `restrict` to prevent accidental data loss. `user_id` and `lead_pipeline_stage_id` use `set null`.

#### lead_products (junction)
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| quantity | integer | default(0) |
| price | decimal(12,4) | nullable |
| amount | decimal(12,4) | nullable |
| lead_id | unsignedInteger | FK → leads.id (cascade) |
| product_id | unsignedInteger | FK → products.id (cascade) |

#### lead_activities (pivot)
- activity_id → activities.id (cascade)
- lead_id → leads.id (cascade)

#### lead_tags (pivot)
- tag_id → tags.id (cascade)
- lead_id → leads.id (cascade)

#### lead_quotes (pivot)
- quote_id → quotes.id (cascade)
- lead_id → leads.id (cascade)

### 3.11 Marketing Package

#### marketing_events
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | nullable, FK → companies.id (cascade) |
| name | string | required |
| description | string | required |
| date | date | required |

#### marketing_campaigns
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | nullable, FK → companies.id (cascade) |
| name | string | required |
| subject | string | required |
| status | boolean | default(0) |
| type | string | required |
| mail_to | string | required |
| spooling | string | nullable |
| marketing_template_id | unsignedInteger | nullable, FK → email_templates.id (set null) |
| marketing_event_id | unsignedInteger | nullable, FK → marketing_events.id (set null) |

### 3.12 Product Package

#### products
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| sku | string | unique, required |
| name | string | nullable |
| description | string | nullable |
| quantity | integer | default(0) |
| price | decimal(12,4) | nullable |

#### product_inventories
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| in_stock | integer | default(0) |
| allocated | integer | default(0) |
| product_id | unsignedInteger | FK → products.id (cascade) |
| warehouse_id | unsignedInteger | nullable, FK → warehouses.id (cascade) |
| warehouse_location_id | unsignedInteger | nullable, FK → warehouse_locations.id (cascade) |

**Computed value:** `on_hand = in_stock - allocated`

#### product_activities (pivot)
- activity_id → activities.id (cascade)
- product_id → products.id (cascade)

#### product_tags (pivot)
- tag_id → tags.id (cascade)
- product_id → products.id (cascade)

### 3.13 Quote Package

#### quotes
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| subject | string | required |
| description | string | nullable |
| billing_address | json | nullable |
| shipping_address | json | nullable |
| discount_percent | decimal(12,4) | nullable, default(0) |
| discount_amount | decimal(12,4) | nullable |
| tax_amount | decimal(12,4) | nullable |
| adjustment_amount | decimal(12,4) | nullable |
| sub_total | decimal(12,4) | nullable |
| grand_total | decimal(12,4) | nullable |
| expired_at | datetime | nullable |
| person_id | unsignedInteger | FK → persons.id (cascade) |
| user_id | unsignedInteger | FK → users.id (cascade) |

#### quote_items
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| sku | string | nullable |
| name | string | nullable |
| quantity | integer | nullable, default(0) |
| price | decimal(12,4) | default(0) |
| coupon_code | string | nullable |
| discount_percent | decimal(12,4) | nullable, default(0) |
| discount_amount | decimal(12,4) | nullable, default(0) |
| tax_percent | decimal(12,4) | nullable, default(0) |
| tax_amount | decimal(12,4) | nullable, default(0) |
| total | decimal(12,4) | default(0) |
| product_id | unsignedInteger | required |
| quote_id | unsignedInteger | FK → quotes.id (cascade) |

### 3.14 Tag Package

#### tags
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| color | string | nullable |
| user_id | unsignedInteger | FK → users.id (cascade) |

### 3.15 User Package

#### groups
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| description | string | nullable |

#### roles
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| description | string | nullable |
| permission_type | string | required ('all' or 'custom') |
| permissions | json | nullable |

#### users
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| hostname | string | tenant hostname |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| email | string | unique, required |
| password | string | nullable |
| status | boolean | default(0) |
| view_permission | string | nullable, default('global') |
| image | string | nullable |
| role_id | unsignedInteger | FK → roles.id (cascade) |
| remember_token | string | nullable |

**view_permission values:** `'global'` (see all data), `'group'` (see group members' data), `'individual'` (see only own data)

#### user_groups (pivot)
- group_id → groups.id (cascade)
- user_id → users.id (cascade)

### 3.16 Warehouse Package

#### warehouses
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| description | text | nullable |
| contact_name | string | required |
| contact_emails | json | required |
| contact_numbers | json | required |
| contact_address | json | required |

#### warehouse_locations
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | required |
| warehouse_id | unsignedInteger | FK → warehouses.id (cascade) |
| **Unique Index** | (warehouse_id, name) | |

#### warehouse_activities (pivot)
- activity_id → activities.id (cascade)
- warehouse_id → warehouses.id (cascade)

#### warehouse_tags (pivot)
- tag_id → tags.id (cascade)
- warehouse_id → warehouses.id (cascade)

### 3.17 Web Form Package

#### web_forms
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| form_id | string | unique, required |
| title | string | required |
| description | text | nullable |
| submit_button_label | text | required |
| submit_success_action | string | required |
| submit_success_content | string | required |
| create_lead | boolean | default(0) |
| background_color | string | nullable |
| form_background_color | string | nullable |
| form_title_color | string | nullable |
| form_submit_button_color | string | nullable |
| attribute_label_color | string | nullable |

#### web_form_attributes
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| name | string | nullable |
| placeholder | string | nullable |
| is_required | boolean | default(0) |
| is_hidden | boolean | default(0) |
| sort_order | integer | nullable |
| attribute_id | unsignedInteger | FK → attributes.id (cascade) |
| web_form_id | unsignedInteger | FK → web_forms.id (cascade) |

### 3.18 SaaS Package (Multi-Tenant Root)

#### companies
| Column | Type | Constraints |
|--------|------|-------------|
| id | bigIncrements | PK |
| name | string | required |
| email | string | required |
| username | string | required (subdomain) |
| domain | string | required (full domain) |
| cname | string | required (custom domain) |
| description | text | required |
| more_info | json | required ({created, seeded}) |
| is_active | boolean | required |
| user_id | integer | required (first admin user) |

#### super_admins
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| first_name | string | required |
| last_name | string | required |
| image | string | nullable |
| email | string | unique, required |
| password | string | required |
| status | boolean | default(0) |
| role_id | unsignedInteger | required |
| api_token | string(80) | unique, nullable |

#### super_roles
| Column | Type | Constraints |
|--------|------|-------------|
| id | bigIncrements | PK |
| name | string | required |
| description | string | nullable |
| permission_type | string | required ('all' or 'custom') |
| permissions | json | nullable |

#### super_config
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| code | string | required |
| value | text | required |

#### company_addresses
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| address1 | text | nullable |
| address2 | text | nullable |
| city, state, zip_code, country, phone | string | nullable |
| misc | json | nullable |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |

#### company_personal_details
| Column | Type | Constraints |
|--------|------|-------------|
| id | increments | PK |
| first_name, last_name | string | nullable |
| email | string | unique, required |
| phone | string | unique, required |
| skype | string | nullable |
| company_id | unsignedBigInteger | FK → companies.id (cascade) |
| more_info | json | nullable |

### 3.19 Laravel Framework Tables

#### jobs (queue)
- id, queue, payload, attempts, reserved_at, available_at, created_at

#### job_batches
- id (string PK), name, total_jobs, pending_jobs, failed_jobs, failed_job_ids, options, cancelled_at, created_at, finished_at

#### failed_jobs
- id, uuid, connection, queue, payload, exception, failed_at

#### personal_access_tokens (Sanctum)
- id, tokenable_type, tokenable_id, name, token, abilities, last_used_at, expires_at

### Schema Patterns Summary

1. **Multi-Tenant Scoping:** All app tables have `company_id` → `companies.id` with cascade delete
2. **JSON Storage:** Contact info (emails, phones), addresses, workflow conditions, webhook payloads
3. **Pivot Tables:** Many-to-many via dedicated tables (lead_tags, person_activities, etc.)
4. **EAV Pattern:** Dynamic fields via attributes + attribute_values tables
5. **Soft Restrictions:** Leads use `restrict` for person/source/type FKs to prevent accidental deletion
6. **Unique Constraints Removed for SaaS:** Original unique indexes (attribute code, org name, group name) removed to allow same values across tenants

---

## 4. Multi-Tenant SaaS System

### 4.1 Architecture Overview

The system uses **single-database, domain-based multi-tenancy**:

```
┌─────────────────────────────────────────────────────────────────┐
│                     ISOLATION LAYERS                             │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: Domain Resolution                                       │
│   HTTP_HOST → Company lookup → Set tenant context               │
│                                                                  │
│ Layer 2: Middleware                                               │
│   ValidatesDomain → Route access control                         │
│                                                                  │
│ Layer 3: Model Query Scoping                                     │
│   newEloquentBuilder() → WHERE company_id = ?                   │
│                                                                  │
│ Layer 4: Database Query Scoping                                  │
│   Custom DatabaseManager → Auto-adds company_id to raw queries  │
│                                                                  │
│ Layer 5: Observer Injection                                       │
│   36 observers → Auto-set company_id on model creation          │
│                                                                  │
│ Layer 6: Auth Guards                                              │
│   super-admin guard vs user guard (separate auth systems)       │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Company Resolution (`Company.php`)

The `Company::getCurrent()` method resolves the current tenant from the HTTP request. It uses a static variable cache so it only resolves once per request.

**Resolution Algorithm:**

```
1. Extract hostname from $_SERVER['HTTP_HOST'] (strip port)
2. Fallback to $_SERVER['SERVER_NAME'] if HTTP_HOST unavailable
3. Compare with config('app.url') primary domain
   → If match: return string 'super-company'
4. Query companies WHERE domain = hostname
   → If found: check is_active, return Company model
5. Fallback: try with port appended (localhost development)
6. Fallback: try CNAME lookup (custom domain)
   → Query companies WHERE cname = hostname
7. If nothing found: return error Response (domain-not-found)
8. If found but is_active = 0: return error Response (company-blocked)
```

**Return Types:**
- `string 'super-company'` - Primary domain (super admin context)
- `Company model` - Valid tenant found
- `Response` - Error (domain not found, company blocked)
- `Collection` - Edge case (defensive code handles this)

### 4.3 Model Query Scoping

Every tenant-aware model overrides `newEloquentBuilder()` to automatically filter queries by `company_id`:

```php
public function newEloquentBuilder($query)
{
    $company = Company::getCurrent();

    // Super admin or non-tenant context: no filtering
    if (auth()->guard('super-admin')->check() || !isset($company->id)) {
        return new Builder($query);
    }

    // Tenant context: filter by company_id
    return new Builder(
        $query->where('table_name.company_id', $company->id)
    );
}
```

**36 models with query scoping:**
Lead, Person, Organization, User, Product, Quote, QuoteItem, Activity, Email, EmailTemplate, Tag, Pipeline, Stage, Source, Type, Attribute, AttributeValue, Role, Group, Warehouse, Location, WebForm, WebFormAttribute, Campaign, Event, Import, ImportBatch, SavedFilter, Workflow, Webhook, and more.

### 4.4 Database Manager Override

For raw queries (not using Eloquent models), the SaaS package overrides Laravel's `DatabaseManager`:

```php
// Custom DB::table() method
public function table($table)
{
    $company = Company::getCurrent();

    // Super admin: no filtering
    if (auth()->guard('super-admin')->check()) {
        return $this->query()->from($table);
    }

    // System tables (no company_id): no filtering
    if (in_array($table, ['companies', 'country_states', 'countries'])) {
        return $this->query()->from($table);
    }

    // Tenant tables: auto-filter by company_id
    return $this->query()->from($table)
        ->where($table.'.company_id', $company->id);
}
```

Registered as singleton to replace Laravel's default:
```php
$this->app->singleton('db', function ($app) {
    return new DatabaseManager($app, $app['db.factory']);
});
```

### 4.5 Observer Pattern (Auto company_id Injection)

36 observers hook into the `creating` event to auto-inject `company_id`:

```php
public function creating(Model $model)
{
    $company = Company::getCurrent();

    if (!auth()->guard('super-admin')->check() && isset($company->id)) {
        if (!isset($model->company_id)) {
            $model->company_id = $company->id;
        }
    }
}
```

**Complete Observer List:**
- ActivityObserver, FileObserver, ParticipantObserver
- AttributeObserver, AttributeOptionObserver, AttributeValueObserver
- PersonObserver, OrganizationObserver
- LeadObserver, PipelineObserver, StageObserver, SourceObserver, TypeObserver
- ProductObserver (Lead context), ProductObserver (Product context)
- QuoteObserver, QuoteItemObserver
- EmailObserver, AttachmentObserver, EmailTemplateObserver
- UserObserver, RoleObserver, GroupObserver
- TagObserver, CoreConfigObserver, SavedFilterObserver
- WorkflowObserver, WebhookObserver
- CampaignObserver, EventObserver
- WarehouseObserver, LocationObserver
- ImportObserver, ImportBatchObserver
- WebFormObserver, WebFormAttributeObserver

### 4.6 Tenant Provisioning Flow

```
Step 1: Registration (POST /company/create)
  ├─ Validate email uniqueness, username not reserved
  ├─ Generate domain: {username}.{primary_domain}
  ├─ Create Company row (more_info: {created: true, seeded: false})
  ├─ Create Administrator Role (permission_type: 'all')
  ├─ Create User (company_id, role_id, hostname)
  ├─ Link company.user_id → user.id
  └─ Return redirect URL to: http://{subdomain}/company/seed-data

Step 2: Seeding (GET /company/seed-data)
  ├─ Check more_info.seeded (skip if already done)
  ├─ Insert country/state data (first company only)
  ├─ Create default attributes (20+ lead/person attributes)
  ├─ Create default email templates
  ├─ Create default pipeline with stages
  ├─ Create default lead sources and types
  ├─ Update more_info: {company_created: true, seeded: true}
  └─ Auto-login first user → redirect to /admin/dashboard
```

### 4.7 Tenant Deletion

When a company is deleted from the super admin panel:
- `companies` row deleted
- **All related data cascades** due to `ON DELETE CASCADE` foreign keys
- Every table with `company_id` FK loses its rows automatically

---

## 5. Routing & Middleware

### 5.1 Route Organization

```
Routes are loaded from three sources:

1. SaaS Package Routes (packages/Webkul/SaasHeadspace/src/Routes/)
   ├── web.php          → Master loader, includes all sub-route files
   ├── company-routes.php → GET/POST /company/create, GET /company/seed-data
   ├── auth-routes.php   → GET/POST /super/login, POST /super/logout
   └── super-routes.php  → /super/* (tenants, settings, configuration)

2. Admin Package Routes (packages/Webkul/Admin/src/Routes/)
   ├── Admin/web.php     → /admin/* (all CRM functionality)
   └── Front/web.php     → Guest routes (login, password reset, web forms)

3. API Routes
   └── /api/* (RESTful API endpoints)
```

### 5.2 Admin Routes Detail

All admin routes are prefixed with `/admin` and protected by `['web', 'admin_locale', 'user']` middleware:

```
/admin/login                     → SessionController (login form)
/admin/dashboard                 → DashboardController
/admin/leads                     → LeadController (index, create, store, show, edit, update, destroy)
/admin/leads/{id}/tags           → TagController (attach/detach tags)
/admin/leads/{id}/activities     → ActivityController (log activities)
/admin/leads/{id}/emails         → EmailController (send/view emails)
/admin/leads/{id}/quotes         → QuoteController (generate quotes)
/admin/contacts/persons          → PersonController (CRUD)
/admin/contacts/organizations    → OrganizationController (CRUD)
/admin/quotes                    → QuoteController (CRUD)
/admin/products                  → ProductController (CRUD)
/admin/activities                → ActivityController (CRUD)
/admin/emails                    → EmailController (CRUD)
/admin/settings/attributes       → AttributeController
/admin/settings/groups           → GroupController
/admin/settings/pipelines        → PipelineController
/admin/settings/roles            → RoleController
/admin/settings/sources          → SourceController
/admin/settings/tags             → TagController
/admin/settings/types            → TypeController
/admin/settings/users            → UserController
/admin/settings/email-templates  → EmailTemplateController
/admin/settings/webhooks         → WebhookController
/admin/settings/workflows        → WorkflowController
/admin/settings/warehouses       → WarehouseController
/admin/settings/web-forms        → WebFormController
/admin/settings/data-transfer/imports → ImportController
/admin/configuration             → ConfigurationController
```

### 5.3 Super Admin Routes

All super routes are prefixed with `/super` and protected by `['web', 'super-admin', 'saas_locale']`:

```
/super/login                     → SessionController (super admin login)
/super/tenants/companies         → TenantController (CRUD)
/super/settings/agents           → AgentController (super admin users)
/super/settings/roles            → RoleController (super admin roles)
/super/configuration             → ConfigurationController (system settings)
/super/account                   → AccountController (profile)
```

### 5.4 Middleware Stack

**Global Middleware (app/Http/Kernel.php):**
```
\Illuminate\Http\Middleware\HandleCors
\Illuminate\Foundation\Http\Middleware\ValidatePostSize
\Illuminate\Foundation\Http\Middleware\TrimStrings
\Illuminate\Foundation\Http\Middleware\ConvertEmptyStringsToNull
\Webkul\SaasHeadspace\Http\Middleware\ValidatesDomain
```

**'web' Group:**
```
EncryptCookies
AddQueuedCookiesToResponse
StartSession
ShareErrorsFromSession
VerifyCsrfToken
SubstituteBindings
```

**Route Middleware Aliases:**
```
user          → BouncerMiddleware (tenant user auth + permissions)
admin_locale  → Locale (set locale for tenant)
super-admin   → RedirectIfNotSuperAdmin + BouncerMiddleware
saas_locale   → SAASLocale (set locale for super admin)
```

### 5.5 ValidatesDomain Middleware

Runs on every request. Determines tenant context:

```
1. Extract hostname from HTTP_HOST
2. If IP address → redirect to company registration
3. If primary domain:
   a. Allow /super/* and /company/* routes
   b. Block /admin/* routes (must use subdomain)
4. If subdomain:
   a. Block /company/* and /super/* routes
   b. Lookup Company by domain
   c. Verify is_active = true
   d. Allow /admin/* routes
```

---

## 6. Authentication & Authorization

### 6.1 Three-Tier Auth System

```
┌────────────────────────────────────────────┐
│ Tier 1: SUPER ADMIN                        │
│   Guard:    'super-admin'                  │
│   Model:    Agent (super_admins table)     │
│   Routes:   /super/*                       │
│   Roles:    SuperRoles (super_roles table) │
│   Sessions: Separate session state         │
├────────────────────────────────────────────┤
│ Tier 2: TENANT USER                        │
│   Guard:    'user' (default)               │
│   Model:    User (users table + company_id)│
│   Routes:   /admin/*                       │
│   Roles:    Roles (roles table + company_id)│
│   Sessions: Per-tenant session             │
├────────────────────────────────────────────┤
│ Tier 3: API                                │
│   Guard:    'api'                          │
│   Provider: Laravel Sanctum                │
│   Routes:   /api/*                         │
│   Auth:     Bearer token                   │
└────────────────────────────────────────────┘
```

### 6.2 Permission System

Both super admin and tenant user permissions follow the same pattern:

```
Role
├── permission_type: 'all'    → Full access to everything
└── permission_type: 'custom' → Check permissions JSON array

permissions: [
    "leads",
    "leads.create",
    "leads.edit",
    "leads.delete",
    "leads.view",
    "contacts.persons",
    "contacts.persons.create",
    ...
]
```

**Permission Check Flow:**
```php
// In Bouncer middleware
if ($role->permission_type === 'all') {
    return true; // Full access
}
return in_array($requiredPermission, $role->permissions);
```

### 6.3 View Permission (Data Visibility)

Users have a `view_permission` field that controls what data they can see:

| Value | Behavior |
|-------|----------|
| `'global'` | See all data in the tenant |
| `'group'` | See data owned by users in same group(s) |
| `'individual'` | See only own data |

Implementation in repositories:
```php
if ($userIds = bouncer()->getAuthorizedUserIds()) {
    $query->whereIn('user_id', $userIds);
}
```

---

## 7. Service Providers & Bootstrapping

### 7.1 SaasHeadspaceServiceProvider (Boot)

This is the central service provider that sets up the multi-tenant system:

```
boot() method:
1. Load helpers (company(), companyBouncer(), superAcl())
2. Load breadcrumbs
3. Load migrations
4. Load translations (namespace: 'saas')
5. Load views (namespace: 'saas')
6. Load all route files
7. Load publishable assets
8. Register 36 model observers
9. Override 33+ base models with SaaS-scoped versions
10. Override data importers
11. Register middleware aliases (super-admin, saas_locale)
12. Register morphMap for polymorphic relationships
13. Register ModuleServiceProvider
```

### 7.2 Model Override Registration

```php
// Base model → SaaS model mapping
Webkul\Lead\Contracts\Lead           → Webkul\SaasHeadspace\Models\Lead\Lead
Webkul\Contact\Contracts\Person      → Webkul\SaasHeadspace\Models\Contact\Person
Webkul\Contact\Contracts\Organization→ Webkul\SaasHeadspace\Models\Contact\Organization
Webkul\User\Contracts\User           → Webkul\SaasHeadspace\Models\User\User
Webkul\User\Contracts\Role           → Webkul\SaasHeadspace\Models\User\Role
Webkul\User\Contracts\Group          → Webkul\SaasHeadspace\Models\User\Group
Webkul\Product\Contracts\Product     → Webkul\SaasHeadspace\Models\Product\Product
Webkul\Quote\Contracts\Quote         → Webkul\SaasHeadspace\Models\Quote\Quote
Webkul\Quote\Contracts\QuoteItem     → Webkul\SaasHeadspace\Models\Quote\QuoteItem
Webkul\Activity\Contracts\Activity   → Webkul\SaasHeadspace\Models\Activity\Activity
Webkul\Email\Contracts\Email         → Webkul\SaasHeadspace\Models\Email\Email
Webkul\Tag\Contracts\Tag             → Webkul\SaasHeadspace\Models\Tag\Tag
Webkul\Attribute\Contracts\Attribute → Webkul\SaasHeadspace\Models\Attribute\Attribute
// ... and 20+ more
```

### 7.3 Singleton Registrations

```php
$this->app->singleton('company', fn() => app()->make(Company::class));
$this->app->singleton('superbouncer', fn() => app()->make(SuperBouncer::class));
$this->app->singleton('db', fn($app) => new DatabaseManager($app, $app['db.factory']));
```

### 7.4 AdminServiceProvider (Boot)

```
boot() method:
1. Register middleware aliases (user → Bouncer, admin_locale → Locale)
2. Load helpers
3. Register admin routes with ['web', 'admin_locale', 'user'] middleware
4. Register front routes with ['web', 'admin_locale'] middleware
5. Load migrations, translations (namespace: 'admin'), views (namespace: 'admin')
6. Register exception handler
```

---

## 8. Business Logic Layer

### 8.1 Repository Pattern

All business logic is encapsulated in Repository classes extending `Webkul\Core\Eloquent\Repository`. Repositories handle validation, relationship management, and event dispatching.

### 8.2 Lead Repository

**Key Methods:**

**create(array $data):**
```
1. Create or link Person (auto-creates if person data provided)
2. Set default pipeline/stage if not specified
3. Create Lead record
4. Save custom AttributeValues
5. Create associated Products (lead_products junction)
6. Return Lead model
```

**update(array $data, id):**
```
1. Handle Person creation/linking
2. If stage changed to 'won' or 'lost':
   → Auto-set closed_at = now()
3. Update lead record
4. Save custom attributes (partial update)
5. Manage Products (create/update/delete)
```

**getLeadsQuery():**
```
Complex query joining:
  - users (salesperson)
  - persons (customer)
  - lead_types, lead_sources
  - lead_pipeline_stages, lead_pipelines
  - tags
Calculates "rotten_days":
  rotten = created_at + pipeline.rotten_days < today
Filters by user authorization (view_permission)
Groups by leads.id
```

### 8.3 Person Repository

**create(array $data):**
```
1. Generate unique_id from (user_id + org_id + email + phone)
2. Auto-create Organization if organization_name provided
3. Create Person record
4. Save custom AttributeValues
```

**fetchOrCreateOrganizationByName(name):**
```
1. Search organizations WHERE name = provided name
2. If found → return id
3. If not found → create Organization, return id
```

### 8.4 Quote Repository

**create(array $data):**
```
1. Create Quote record
2. Create QuoteItems (line items with sku, quantity, price, discounts, tax)
3. Save custom AttributeValues
```

**update(array $data, id):**
```
1. Update Quote record
2. Manage QuoteItems lifecycle:
   - Create new items
   - Update existing items
   - Delete removed items
3. Update attributes
```

### 8.5 Product Repository

**saveInventories(array $data, productId, warehouseId):**
```
For each inventory entry in data:
  1. If inventory record exists:
     - Update in_stock and allocated quantities
  2. If not exists:
     - Create new ProductInventory record
  3. Delete inventory records not in update data
```

**getInventoriesGroupedByWarehouse(productId):**
```
Returns nested structure:
  warehouse → locations → {in_stock, allocated, on_hand}
```

### 8.6 Activity Repository

**create(array $data):**
```
1. Save attached files (activity_files)
2. Create activity participants:
   - Users (internal team members)
   - Persons (external contacts)
3. Return Activity model
```

**getActivities(dateRange):**
```
Query activities within date range
Filter by participant (current user)
Used for calendar view
```

**isDurationOverlapping():**
```
Check if new activity schedule conflicts with existing ones
Returns boolean
```

### 8.7 Email Repository

**create(array $data):**
```
1. Generate unique message_id
2. Inherit reference_ids for threading (from parent)
3. Set defaults: source, folders, user_type
4. Sanitize email lists (reply_to, cc, bcc - filter empty values)
5. Create Email record
```

### 8.8 Controller Patterns

All controllers follow this pattern:

```php
// Index (list)
public function index() {
    if (request()->ajax()) {
        return datagrid(EntityDataGrid::class)->process();
    }
    return view('admin::entity.index');
}

// Store (create)
public function store(FormRequest $request) {
    Event::dispatch('entity.create.before');
    $entity = $this->repository->create(request()->all());
    Event::dispatch('entity.create.after', $entity);
    return redirect()->route('admin.entity.index');
}

// Update
public function update(FormRequest $request, $id) {
    Event::dispatch('entity.update.before', $id);
    $entity = $this->repository->update(request()->all(), $id);
    Event::dispatch('entity.update.after', $entity);
    return redirect()->route('admin.entity.index');
}

// Destroy
public function destroy($id) {
    Event::dispatch('entity.delete.before', $id);
    $this->repository->delete($id);
    Event::dispatch('entity.delete.after', $id);
    return response()->json(['message' => 'Success']);
}

// Mass Destroy
public function massDestroy(MassDestroyRequest $request) {
    foreach ($request->input('indices') as $id) {
        Event::dispatch('entity.delete.before', $id);
        $this->repository->delete($id);
        Event::dispatch('entity.delete.after', $id);
    }
}
```

---

## 9. Custom Attribute System (EAV)

### 9.1 Architecture

The Entity-Attribute-Value pattern allows unlimited custom fields on any entity:

```
Entity (e.g., Lead)
    │
    └──► attribute_values (rows per field per entity)
            ├── entity_type: 'leads'
            ├── entity_id: 42
            ├── attribute_id: 7 (FK → attributes)
            └── [type_column]: actual value stored

Attribute Definition (attributes table)
    ├── code: 'custom_field_1'
    ├── name: 'Custom Field'
    ├── type: 'text' | 'select' | 'price' | etc.
    ├── entity_type: 'leads'
    └── is_required, is_unique, validation rules
```

### 9.2 Type-to-Column Mapping

| Attribute Type | Storage Column | Notes |
|---------------|----------------|-------|
| text | text_value | |
| textarea | text_value | |
| price | float_value | |
| boolean | boolean_value | Stored as 1/0 |
| select | integer_value | FK to attribute_options.id |
| multiselect | text_value | Comma-separated option IDs |
| checkbox | text_value | Comma-separated option IDs |
| email | json_value | Array of {value, label} |
| phone | json_value | Array of {value, label} |
| address | json_value | JSON object |
| lookup | integer_value | FK to lookup entity |
| date | date_value | |
| datetime | datetime_value | |
| file | text_value | File path string |
| image | text_value | File path string |

### 9.3 Saving Attributes

```
For each attribute in entity's attribute list:
  1. Determine storage column from type mapping
  2. Apply type-specific conversions:
     - boolean → 1/0
     - multiselect/checkbox → implode(',')
     - email/phone → sanitize JSON array
     - image/file → store uploaded file, save path
     - price → allow null
  3. Upsert into attribute_values:
     WHERE entity_type AND entity_id AND attribute_id
  4. If is_unique: validate uniqueness across entities
```

### 9.4 Lookup Attributes

Lookup attributes reference other entities for autocomplete/search:

| Lookup Type | Entity |
|-------------|--------|
| users | User |
| lead_pipelines | Pipeline |
| lead_pipeline_stages | Stage |
| lead_sources | Source |
| lead_types | Type |

---

## 10. Event System

### 10.1 Event Naming Convention

```
{entity}.{action}.before
{entity}.{action}.after
```

### 10.2 Complete Event List

**Lead Events:**
- `lead.create.before` / `lead.create.after`
- `lead.update.before` / `lead.update.after`
- `lead.delete.before` / `lead.delete.after`
- `lead.product.delete.before` / `lead.product.delete.after`

**Contact Events:**
- `contacts.person.create.before` / `contacts.person.create.after`
- `contacts.person.update.before` / `contacts.person.update.after`
- `contacts.person.delete.before` / `contacts.person.delete.after`
- `contact.organization.delete.before` / `contact.organization.delete.after`

**Quote Events:**
- `quote.create.before` / `quote.create.after`
- `quote.update.before` / `quote.update.after`
- `quote.delete.before` / `quote.delete.after`

**Product Events:**
- `product.create.before` / `product.create.after`
- `product.update.before` / `product.update.after`
- `product.delete.before` / `product.delete.after`

**User Events:**
- `settings.user.create.before` / `settings.user.create.after`
- `settings.user.update.before` / `settings.user.update.after`
- `settings.user.delete.before` / `settings.user.delete.after`

**Settings Events:**
- `settings.role.create.before` / `settings.role.create.after`
- `settings.attribute.create.before` / `settings.attribute.create.after`
- `settings.role.update/delete.*`
- `settings.attribute.update/delete.*`

**Super Admin Events:**
- `super.company.delete.before` / `super.company.delete.after`
- `super.company.update.before` / `super.company.update.after`
- `super.agent.delete.before` / `super.agent.delete.after`

**Usage:** Events are dispatched via `Event::dispatch()` and can be listened to for notifications, automations, external integrations, and audit logging.

---

## 11. DataGrid System

### 11.1 Architecture

DataGrids provide server-side filtering, sorting, pagination, and mass actions for entity listing pages.

### 11.2 DataGrid Flow

```
1. Controller checks if request is AJAX
2. If AJAX: return datagrid(EntityDataGrid::class)->process()
3. DataGrid::process() does:
   a. Build query (prepareQueryBuilder)
   b. Apply filters from request
   c. Apply sorting
   d. Paginate results
   e. Return JSON response
4. Frontend Vue component handles rendering
```

### 11.3 LeadDataGrid (Example)

```
Query joins:
  leads
  LEFT JOIN users ON leads.user_id
  LEFT JOIN persons ON leads.person_id
  LEFT JOIN lead_types ON leads.lead_type_id
  LEFT JOIN lead_pipeline_stages ON leads.lead_pipeline_stage_id
  LEFT JOIN lead_sources ON leads.lead_source_id
  LEFT JOIN lead_pipelines ON leads.lead_pipeline_id
  LEFT JOIN lead_tags ON leads.id = lead_tags.lead_id
  LEFT JOIN tags ON lead_tags.tag_id

Computed columns:
  rotten_lead = IF(
    DATEDIFF(NOW(), leads.created_at) > lead_pipelines.rotten_days,
    true, false
  )

Filters: id, user, sales_person, lead_source_name, lead_type_name,
         person_name, type (stage code), stage, tag_name,
         expected_close_date, rotten_lead

Mass Actions: delete, update stage
```

### 11.4 Saved Filters

Users can save filter configurations (stored in `datagrid_saved_filters` table) and reapply them later.

---

## 12. Data Import/Export

### 12.1 Import Flow

```
1. Create Import Record
   - type: 'persons', 'leads', 'products', etc.
   - action: 'append', 'delete'
   - validation_strategy: 'stop-on-errors', 'skip-errors'
   - file_path: uploaded CSV path
   - process_in_queue: boolean

2. Validate Import
   - Parse CSV, validate each row against entity rules
   - Track invalid_rows_count, errors_count
   - Generate error_file_path if errors found

3. Execute Import
   IF process_in_queue = true:
     - Requires QUEUE_CONNECTION=database or redis
     - Splits into ImportBatches
     - Each batch processed as queued job
   ELSE:
     - Process synchronously in current request

4. Processing
   - For each row: create/update entity via repository
   - Track processed_rows_count
   - Update summary JSON
   - Set completed_at when done

5. States: pending → validating → validated → processing → completed/failed
```

### 12.2 Supported Import Types

- Persons (contacts)
- Organizations
- Leads
- Products

---

## 13. Email System

### 13.1 Email Threading

```
Email A (incoming, parent_id = NULL)
├── Email B (reply, parent_id = A.id)
├── Email C (reply, parent_id = A.id)
└── Email D (reply to C, parent_id = A.id)

Threading uses:
- parent_id: FK to parent email (first in thread)
- message_id: unique RFC 2822 message ID
- reference_ids: JSON array of related message IDs
```

### 13.2 Email Fields

- **from, sender**: JSON objects with name/email
- **reply_to, cc, bcc**: JSON arrays of email addresses
- **folders**: JSON array (inbox, sent, draft, trash, etc.)
- **source**: Origin of email (web, imap, etc.)
- **is_read**: Boolean read status
- **person_id**: Links to CRM contact
- **lead_id**: Links to sales opportunity

---

## 14. Automation & Marketing

### 14.1 Workflow System

Workflows define automated actions triggered by entity events:

```
Workflow Definition:
├── entity_type: 'leads', 'persons', etc.
├── event: 'created', 'updated', etc.
├── condition_type: 'and' | 'or'
├── conditions: JSON array of rules
│   [{ attribute: 'lead_value', operator: '>', value: 1000 }]
└── actions: JSON array of actions
    [{ type: 'send_email', template_id: 5 }]
    [{ type: 'update_attribute', attribute: 'status', value: 1 }]
```

### 14.2 Webhook System

Webhooks send HTTP requests when entity events occur:

```
Webhook Definition:
├── entity_type: 'leads'
├── method: 'POST'
├── end_point: 'https://external-api.com/hook'
├── headers: JSON (Authorization, Content-Type, etc.)
├── payload_type: 'json' | 'form'
└── payload: JSON template with entity data placeholders
```

### 14.3 Marketing Campaigns

```
Campaign
├── name, subject
├── type: email type
├── mail_to: target audience
├── status: active/inactive
├── marketing_template_id → email_templates
├── marketing_event_id → marketing_events
└── spooling: scheduling configuration

Events
├── name
├── description
└── date: trigger date
```

---

## 15. Frontend Architecture

### 15.1 Rendering Model

The frontend uses a **hybrid Blade + Vue** approach:

```
Server-side (Blade):
├── Full page layouts (header, sidebar, content area)
├── 156 reusable anonymous components
├── Server-rendered HTML for SEO/initial load
└── Permission checks (@if bouncer()->hasPermission())

Client-side (Vue 3):
├── Interactive components (kanban, forms, modals)
├── Registered inline via <script type="text/x-template">
├── Mounted at window.onload on #app element
└── Global plugins (axios, emitter, validation)
```

### 15.2 Layout System

**Main Layout** (`layouts/index.blade.php`):
```html
<html>
<head>
    <!-- Vite CSS -->
    <!-- Meta, favicon -->
</head>
<body>
    <div id="app">
        <x-admin::layouts.header />      <!-- Top bar -->
        <x-admin::layouts.sidebar />     <!-- Left nav -->
        <main>{{ $slot }}</main>          <!-- Page content -->
    </div>
    <!-- Vite JS -->
    @stack('scripts')  <!-- Vue component registrations -->
</body>
</html>
```

**Anonymous Layout** (`layouts/anonymous.blade.php`):
- Used for login, password reset pages
- No sidebar/header
- Minimal chrome

### 15.3 Blade Component Library (156 components)

```
x-admin::accordion           Collapsible sections
x-admin::activities          Activity timeline + action buttons
x-admin::attachments         File upload/download
x-admin::attributes          Dynamic field rendering (edit/view modes)
x-admin::avatar              User avatars with initials fallback
x-admin::breadcrumbs         Navigation breadcrumbs
x-admin::button              Styled button variants
x-admin::charts              Chart.js integration wrapper
x-admin::datagrid            Data table (sort, filter, paginate, mass actions)
x-admin::dropdown            Dropdown menus
x-admin::flash-group         Toast notifications
x-admin::flat-picker         Date/time picker wrapper
x-admin::form                Form builder
x-admin::form.control-group  Form field wrappers with labels/errors
x-admin::form.controls.*     13 input types:
  ├── text, email, password, number
  ├── textarea, select, multiselect
  ├── checkbox, radio, switch
  ├── date, datetime
  └── file, image
x-admin::lookup              Autocomplete/search fields
x-admin::media               Image/file uploads with preview
x-admin::modal               Dialog modals
x-admin::shimmer.*           Loading skeleton placeholders
x-admin::spinner             Loading indicators
x-admin::table               Semantic table structure
x-admin::tabs                Tabbed interface
x-admin::tags                Tag management
x-admin::tinymce             WYSIWYG rich text editor
x-admin::tree                Hierarchical tree view (permissions)
```

### 15.4 Vue 3 Setup

**Root Instance** (`app.js`):
```javascript
window.app = createApp({
    data() {
        return {
            isMenuActive: false,
            hoveringMenu: null,
        }
    },
    // Sidebar hover/collapse, form validation scroll-to-error
});

// Mounted on window.onload (allows Blade components to register first)
```

**Global Plugins (8):**

| Plugin | Purpose | Global Property |
|--------|---------|-----------------|
| Admin | Currency formatting | `$admin.formatPrice()` |
| Axios | HTTP client with CSRF | `$axios` |
| Emitter | Event bus (mitt) | `$emitter` |
| VeeValidate | Form validation | VForm, VField, VErrorMessage |
| Flatpickr | Date/time picker | Component registration |
| Vue-Cal | Calendar component | Component registration |
| Draggable | Drag-and-drop | vuedraggable component |
| CreateElement | Vue render helpers | `$h`, `$resolveComponent` |

**Global Directives (2):**
- `v-debounce` - Input debouncing (default 500ms)
- `v-safe-html` - DOMPurify HTML sanitization

### 15.5 Key Vue Components (Inline Registration)

**v-leads-kanban** - Kanban board for lead management:
```
- Stage columns with lead counts and value totals
- Drag-and-drop via vuedraggable (moves leads between stages)
- Inline quick creation per stage
- Progress bars showing stage value percentage
- Infinite scroll loading within stages
```

**v-dashboard-filters** - Dashboard date range filtering:
```
- Predefined ranges (today, week, month, quarter, year)
- Custom date range with flatpickr
- Emits filter changes to dashboard widgets
```

**v-dark** - Dark mode toggle:
```
- Reads/writes 'dark_mode' cookie
- Toggles 'dark' class on <html>
- Emits 'change-theme' event for logo switching
```

**v-activities** - Activity list with dual view:
```
- Table view (DataGrid)
- Calendar view (vue-cal)
- View toggle switch
```

### 15.6 CSS Architecture

**Framework:** Tailwind CSS 3.3.2

**Configuration:**
```javascript
// tailwind.config.js
theme: {
    screens: { sm: 525, md: 768, lg: 1024, xl: 1240 },
    extend: {
        colors: { brandColor: 'var(--brand-color)' },
        fontFamily: { inter: ['Inter'], icon: ['icomoon'] }
    }
},
darkMode: 'class',
safelist: [{ pattern: /icon-/ }]
```

**Custom Icon Font:** icomoon (150+ icons defined in CSS)

**Dark Mode:** Class-based toggling (`dark:bg-gray-950`, etc.)

**RTL Support:** Tailwind `ltr:`/`rtl:` prefixes for Arabic/Farsi

### 15.7 Build System

**Vite 5.4** with Laravel Vite Plugin:
- Entry points: `resources/css/app.css`, `resources/js/app.js`
- Hot module replacement in development
- Asset fingerprinting in production
- Output: `/public/build/assets/`

**Key NPM Dependencies:**
```
vue@3.4.21              Chart.js (+ chartjs-chart-funnel)
axios@1.7.4             vuedraggable@4.1.0
tailwindcss@3.3.2       vue-cal@4.9.0
vee-validate@4.9.1      flatpickr@4.6.13
vite@5.4.12             dompurify@3.1.7
mitt@3.0.1              @playwright/test@1.50.1
```

### 15.8 Localization

**20 supported locales:**
ar (Arabic), bn (Bengali), de (German), en (English), es (Spanish), fa (Farsi), fr (French), he (Hebrew), hi_IN (Hindi), it (Italian), ja (Japanese), nl (Dutch), pl (Polish), pt_BR (Portuguese), ru (Russian), sin (Sinhala), tr (Turkish), uk (Ukrainian), vi (Vietnamese), zh_CN (Chinese)

**Implementation:**
- Blade: `@lang('admin::app.leads.index.title')`
- Vue: `@{{ $t('admin::app.leads.index.title') }}`
- VeeValidate: Custom error messages per locale
- Direction: RTL for ar/fa, LTR for others

### 15.9 Key Page Templates

**Dashboard** (`/admin/dashboard`):
- Revenue stats cards
- Overall metrics (line chart)
- Total leads (funnel chart)
- Top selling products, top persons
- Open leads by stage
- Revenue by source/type

**Leads** (`/admin/leads`):
- Dual view: Kanban (default) or table
- Kanban: drag-and-drop stages, inline quick creation
- Table: DataGrid with filters, sorting, mass actions
- Detail view: lead info, activities, emails, quotes

**Contacts - Persons** (`/admin/contacts/persons`):
- DataGrid listing
- Detail view: sticky sidebar, activity actions, tabbed interface
- Custom attributes

**Quotes** (`/admin/quotes`):
- DataGrid listing
- Quote builder with line items
- PDF export template

**Activities** (`/admin/activities`):
- Dual view: table or calendar
- Calendar view with color-coded events
- Activity types: call, meeting, lunch, task, note

### 15.10 View Extension Points

The Blade templates include extensibility hooks:
```blade
{!! view_render_event('admin.leads.index.header.before') !!}
<!-- Content here -->
{!! view_render_event('admin.leads.index.header.after') !!}
```

These allow external packages to inject content at specific points without modifying core templates.

---

## 16. API Layer

### 16.1 REST API

The application exposes a RESTful API at `/api/*` routes, authenticated via Laravel Sanctum bearer tokens.

### 16.2 Internal AJAX Endpoints

Most admin pages use AJAX calls to the same admin routes:
- DataGrid data loading (server-side pagination/filtering)
- Dashboard statistics updates
- Activity calendar fetching
- Quick creation forms
- Mass actions (bulk update/delete)
- Tag management
- File uploads

All AJAX calls include:
- `X-Requested-With: XMLHttpRequest` header
- CSRF token from cookie
- JSON request/response bodies

---

## 17. Request Lifecycle

### 17.1 Tenant User Request Flow

```
Browser: GET https://asif.headspace.local:8000/admin/leads
│
├─ 1. Global Middleware
│   ├─ HandleCors
│   ├─ ValidatePostSize
│   ├─ TrimStrings
│   ├─ ConvertEmptyStringsToNull
│   └─ ValidatesDomain
│       ├─ Extract 'asif.headspace.local' from HTTP_HOST
│       ├─ Compare with config('app.url') → not primary domain
│       ├─ Lookup Company WHERE domain = 'asif.headspace.local'
│       ├─ Verify is_active = 1
│       └─ Set company context singleton
│
├─ 2. Web Middleware Group
│   ├─ EncryptCookies
│   ├─ AddQueuedCookiesToResponse
│   ├─ StartSession
│   ├─ ShareErrorsFromSession
│   ├─ VerifyCsrfToken
│   └─ SubstituteBindings
│
├─ 3. Route Middleware
│   ├─ admin_locale → Set locale from tenant config
│   └─ user (Bouncer)
│       ├─ Check auth()->guard('user')->check() → authenticated?
│       ├─ Check user->status → active?
│       └─ Check user->role->permissions → has 'leads' permission?
│
├─ 4. Controller: LeadController::index()
│   ├─ Check if AJAX → return DataGrid
│   └─ Return view('admin::leads.index')
│
├─ 5. DataGrid Query (if AJAX)
│   ├─ Lead::query() → newEloquentBuilder() auto-adds:
│   │   WHERE leads.company_id = {asif's company id}
│   ├─ Apply user filters, sorting, pagination
│   └─ Return JSON response
│
└─ 6. View Rendering (if not AJAX)
    ├─ Blade template compiled with Tailwind classes
    ├─ Vue components registered via @pushOnce('scripts')
    ├─ Shimmer loading states shown initially
    └─ Vue mounts, AJAX loads DataGrid/Kanban data
```

### 17.2 Super Admin Request Flow

```
Browser: GET https://headspace.local:8000/super/tenants
│
├─ ValidatesDomain → Recognizes primary domain → 'super-company'
├─ Web middleware group
├─ super-admin (Bouncer) → Check super-admin guard
├─ TenantController::index()
│   ├─ Company::getCurrent() = 'super-company'
│   ├─ auth()->guard('super-admin')->check() = true
│   └─ Query ALL companies (no company_id filtering)
└─ Return view with all tenants
```

### 17.3 Data Creation Flow

```
POST /admin/leads (create new lead)
│
├─ LeadForm Request validation (dynamic attribute rules)
│
├─ LeadController::store()
│   ├─ Event::dispatch('lead.create.before')
│   │
│   ├─ LeadRepository::create($data)
│   │   ├─ Create/link Person
│   │   ├─ parent::create($data) → Lead model created
│   │   │   └─ LeadObserver::creating() → auto-inject company_id
│   │   ├─ AttributeValueRepository::save() → store custom fields
│   │   │   └─ AttributeValueObserver::creating() → auto-inject company_id
│   │   └─ Create lead_products → junction records
│   │       └─ LeadProductObserver::creating() → auto-inject company_id
│   │
│   ├─ Event::dispatch('lead.create.after', $lead)
│   │   └─ (Workflow engine checks for matching automation rules)
│   │
│   └─ Redirect to lead index with success flash message
```

---

## Appendix A: Entity Relationship Diagram

```
                    ┌──────────────┐
                    │   companies  │ (tenant root)
                    │   (SaaS)     │
                    └──────┬───────┘
                           │ company_id (on all tables below)
    ┌──────────────────────┼──────────────────────────┐
    │                      │                          │
    ▼                      ▼                          ▼
┌────────┐         ┌──────────────┐            ┌──────────┐
│ users  │◄────────│organizations │            │  roles   │
│        │ user_id │              │            │          │
└───┬────┘         └──────┬───────┘            └──────────┘
    │                     │ organization_id         ▲
    │                     ▼                         │ role_id
    │              ┌──────────────┐                 │
    │  user_id     │   persons   │─────────────────┘
    │              │              │
    │              └──────┬───────┘
    │                     │ person_id
    │                     ▼
    │              ┌──────────────┐      ┌──────────────────┐
    │  user_id     │    leads    │◄─────│ lead_pipelines    │
    ├─────────────►│              │      │   └─► stages     │
    │              │              │──────│ lead_sources      │
    │              │              │      │ lead_types        │
    │              └──────┬───────┘      └──────────────────┘
    │                     │
    │         ┌───────────┼───────────┐
    │         │           │           │
    │         ▼           ▼           ▼
    │  ┌──────────┐ ┌──────────┐ ┌──────────┐
    │  │lead_     │ │lead_     │ │lead_     │
    │  │products  │ │activities│ │quotes    │
    │  └────┬─────┘ └────┬─────┘ └────┬─────┘
    │       │             │            │
    │       ▼             ▼            ▼
    │  ┌──────────┐ ┌──────────┐ ┌──────────┐
    │  │products  │ │activities│ │  quotes  │
    │  │          │ │          │ │          │
    │  └────┬─────┘ └──────────┘ └────┬─────┘
    │       │                         │
    │       ▼                         ▼
    │  ┌──────────────┐         ┌──────────┐
    │  │product_      │         │quote_    │
    │  │inventories   │         │items     │
    │  └────┬─────────┘         └──────────┘
    │       │
    │       ▼
    │  ┌──────────────┐     ┌──────────────┐
    │  │ warehouses   │────►│warehouse_    │
    │  │              │     │locations     │
    │  └──────────────┘     └──────────────┘
    │
    ▼
┌──────────────┐     ┌──────────────┐
│   emails     │     │    tags      │
│ (threaded)   │     │ (polymorphic)│
└──────────────┘     └──────────────┘
```

---

## Appendix B: Configuration System

### Tenant Configuration

Stored in `core_config` table (scoped by company_id):
```
general.general.admin_logo.logo_image
general.general.locale_settings.locale
general.general.locale_settings.timezone
```

Accessed via: `core()->getConfigData('config.code')`

### Super Admin Configuration

Stored in `super_config` table (global):
```
System-wide settings for all tenants
```

Accessed via: `Company::getSuperConfigData('code')`

### Admin Menu Configuration

Defined in `config/menu.php` for each package:
```php
[
    'key'        => 'leads',
    'name'       => 'admin::app.layouts.leads',
    'route'      => 'admin.leads.index',
    'sort'       => 2,
    'icon-class' => 'icon-leads',
]
```

### ACL Configuration

Hierarchical permission structure defined in `config/acl.php`:
```php
[
    'key'   => 'leads',
    'name'  => 'Leads',
    'route' => 'admin.leads.index',
    'sort'  => 1,
    'children' => [
        ['key' => 'leads.create', ...],
        ['key' => 'leads.edit', ...],
        ['key' => 'leads.delete', ...],
        ['key' => 'leads.view', ...],
    ]
]
```

---

## Appendix C: Form Validation

### LeadForm Request

Dynamic validation built from custom attributes:
```
For each attribute in entity type:
  - address: all sub-fields required if attribute is_required
  - email: each value must be valid email, label required
  - phone: each value required, label required
  - text: custom validation (decimal, URL, etc.)
  - price: must be decimal
  - unique: callback validation against existing records

Product validation (if products attached):
  - product_id: required
  - name: required
  - price: required, decimal
  - quantity: required, integer
```

### AttributeForm Request

Generic form used for any entity with custom attributes. Dynamically builds validation rules based on entity's attribute definitions.

---

## Appendix D: Key Design Patterns

### 1. Repository Pattern
All data access via repository classes (not direct model queries in controllers).

### 2. Observer Pattern
36 observers for automatic company_id injection on model creation.

### 3. Contract/Proxy Pattern
Interface-based model binding allows SaaS layer to swap implementations.

### 4. Event-Driven Architecture
Before/after events on every CRUD operation for extensibility.

### 5. EAV (Entity-Attribute-Value)
Dynamic custom fields without schema changes.

### 6. DataGrid Abstraction
Reusable server-side filtered/sorted/paginated data tables.

### 7. Singleton Company Context
Resolved once per request, cached in static variable.

### 8. Guard-Based Auth
Separate authentication systems for super admin vs tenant users.

### 9. Middleware Pipeline
Layered request processing (domain validation → auth → locale → permissions).

### 10. Inline Vue Components
Vue components defined within Blade templates using `<script type="text/x-template">` and `@pushOnce('scripts')`.
