//! Askama view structs for tenant admin pages.

use askama::Template;
use askama_web::WebTemplate;

use super::vite;

fn authenticated_page(
    page: &'static str,
    csrf_token: String,
    initial_data: String,
) -> (String, &'static str, String, String, String) {
    (
        csrf_token,
        page,
        initial_data,
        vite::js_file(),
        vite::css_file(),
    )
}

// -- Login --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/auth/login.html")]
pub struct LoginPage {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl LoginPage {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        Self {
            csrf_token,
            page: "admin-login",
            initial_data,
            js_file: vite::js_file(),
            css_file: vite::css_file(),
        }
    }
}

// -- Dashboard --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/dashboard/index.html")]
pub struct Dashboard {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl Dashboard {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-dashboard", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Settings --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/index.html")]
pub struct SettingsIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl SettingsIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-settings", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Users --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/users/index.html")]
pub struct UserIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl UserIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-user-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/users/create.html")]
pub struct UserCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl UserCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-user-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/users/edit.html")]
pub struct UserEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl UserEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-user-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Roles --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/roles/index.html")]
pub struct RoleIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl RoleIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-role-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/roles/create.html")]
pub struct RoleCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl RoleCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-role-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/roles/edit.html")]
pub struct RoleEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl RoleEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-role-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Groups --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/groups/index.html")]
pub struct GroupIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl GroupIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-group-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/groups/create.html")]
pub struct GroupCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl GroupCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-group-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/groups/edit.html")]
pub struct GroupEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl GroupEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-group-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Attributes --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/attributes/index.html")]
pub struct AttributeIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl AttributeIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-attribute-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/attributes/create.html")]
pub struct AttributeCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl AttributeCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-attribute-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/attributes/edit.html")]
pub struct AttributeEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl AttributeEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-attribute-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Pipelines --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/pipelines/index.html")]
pub struct PipelineIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl PipelineIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-pipeline-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/pipelines/create.html")]
pub struct PipelineCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl PipelineCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-pipeline-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/pipelines/edit.html")]
pub struct PipelineEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl PipelineEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-pipeline-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Sources --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/sources/index.html")]
pub struct SourceIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl SourceIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-source-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Tags --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/tags/index.html")]
pub struct TagIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl TagIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-tag-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Persons --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/persons/index.html")]
pub struct PersonIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl PersonIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-person-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/persons/create.html")]
pub struct PersonCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl PersonCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-person-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/persons/edit.html")]
pub struct PersonEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl PersonEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-person-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/persons/show.html")]
pub struct PersonShow {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl PersonShow {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-person-detail", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Organizations --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/organizations/index.html")]
pub struct OrganizationIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl OrganizationIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-organization-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/organizations/create.html")]
pub struct OrganizationCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl OrganizationCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-organization-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/organizations/edit.html")]
pub struct OrganizationEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl OrganizationEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-organization-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/contacts/organizations/show.html")]
pub struct OrganizationShow {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl OrganizationShow {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-organization-detail", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Leads --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/leads/index.html")]
pub struct LeadIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl LeadIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-lead-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/leads/create.html")]
pub struct LeadCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl LeadCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-lead-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/leads/edit.html")]
pub struct LeadEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl LeadEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-lead-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/leads/kanban.html")]
pub struct LeadKanbanView {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl LeadKanbanView {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-lead-kanban", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/leads/show.html")]
pub struct LeadShow {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl LeadShow {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-lead-detail", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Activities --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/activities/index.html")]
pub struct ActivityIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl ActivityIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-activity-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/activities/create.html")]
pub struct ActivityCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl ActivityCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-activity-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/activities/edit.html")]
pub struct ActivityEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl ActivityEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-activity-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Products --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/products/index.html")]
pub struct ProductIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl ProductIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-product-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/products/create.html")]
pub struct ProductCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl ProductCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-product-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/products/edit.html")]
pub struct ProductEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl ProductEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-product-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Quotes --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/quotes/index.html")]
pub struct QuoteIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl QuoteIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-quote-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/quotes/create.html")]
pub struct QuoteCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl QuoteCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-quote-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/quotes/edit.html")]
pub struct QuoteEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl QuoteEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-quote-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Account --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/account.html")]
pub struct AccountEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl AccountEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-account", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Configuration --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/configuration/index.html")]
pub struct ConfigurationIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl ConfigurationIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-configuration", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Email --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/mail/index.html")]
pub struct EmailIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl EmailIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-email", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Types --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/types/index.html")]
pub struct TypeIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl TypeIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-type-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Email Templates --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/email_templates/index.html")]
pub struct EmailTemplateIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl EmailTemplateIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-email-template-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/email_templates/create.html")]
pub struct EmailTemplateCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl EmailTemplateCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-email-template-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/email_templates/edit.html")]
pub struct EmailTemplateEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl EmailTemplateEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-email-template-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Warehouses --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/warehouses/index.html")]
pub struct WarehouseIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl WarehouseIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-warehouse-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/warehouses/create.html")]
pub struct WarehouseCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl WarehouseCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-warehouse-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/warehouses/edit.html")]
pub struct WarehouseEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl WarehouseEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-warehouse-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

// -- Web Forms --

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/web_forms/index.html")]
pub struct WebFormIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl WebFormIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-web-form-list", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/web_forms/create.html")]
pub struct WebFormCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl WebFormCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-web-form-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/admin/settings/web_forms/edit.html")]
pub struct WebFormEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl WebFormEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("admin-web-form-form", csrf_token, initial_data);
        Self {
            csrf_token,
            page,
            initial_data,
            js_file,
            css_file,
        }
    }
}
