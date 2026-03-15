# Roadmap

This document outlines planned features and improvements for Headspace CRM. Items are roughly ordered by priority within each section. Contributions are welcome on any of these.

## Near-term

### Dynamic Attribute Forms
Custom attributes exist in the database but aren't rendered on entity create/edit forms yet. This would add a `DynamicFieldRenderer` Vue component that reads the attribute definitions and generates the appropriate form fields (text, number, select, date, checkbox, etc.) on lead, person, and organization forms.

### Server-side Pagination (All Entities)
Currently only the leads list uses server-side pagination (`v-data-table-server`). Extend this pattern to persons, organizations, products, quotes, and activities for better performance with large datasets.

### Email Attachments (Outbound)
Outbound email compose currently supports text/HTML only. Add multipart file attachment support to the compose dialog and SMTP sending.

### Notification System
In-app notifications for events like new lead assignment, activity reminders, and email sync completion. Bell icon in the header with a dropdown list and read/unread state.

## Mid-term

### Workflow / Automation Engine
Rule-based automation triggered by CRM events:
- **Triggers**: lead created, stage changed, lead won/lost, activity created
- **Conditions**: field value matches, time elapsed, assigned user
- **Actions**: send email (from template), assign user, move stage, create activity, update field

### Reporting & Analytics
Dedicated reporting section with:
- Custom date range comparisons
- Exportable charts and tables
- Sales forecasting based on pipeline data
- User/team performance metrics
- Lead conversion funnel analysis

### API Authentication (Token-based)
REST API authentication via API keys or JWT tokens for external integrations. Currently all API access requires a browser session.

### Audit Log
Track who changed what and when. Record creates, updates, and deletes on all major entities with before/after snapshots. Viewable in settings.

## Longer-term

### Docker / Deployment
- Official Dockerfile (multi-stage: build Rust + frontend, slim runtime image)
- docker-compose.yml with PostgreSQL
- Helm chart for Kubernetes
- Health check and readiness endpoints

### Real-time Updates
WebSocket support (via axum's WebSocket extractor) for:
- Live kanban board updates when another user moves a lead
- New email notifications without polling
- Activity feed updates

### File Management
Centralized file/document management per tenant:
- Upload files to leads, persons, organizations
- File preview (images, PDFs)
- Storage backend abstraction (local disk, S3)

### Calendar Integration
Two-way sync with Google Calendar and Outlook Calendar for activities.

### Mobile-responsive UI
While Vuetify provides some responsiveness out of the box, the current UI is optimized for desktop. A dedicated mobile layout pass would improve the experience on phones and tablets.

### Internationalization (i18n)
Full i18n support with translation files. The locale setting already exists in tenant config but currently only affects date/number formatting, not UI strings.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to get started. If you're interested in working on any roadmap item, open an issue first to discuss the approach.
