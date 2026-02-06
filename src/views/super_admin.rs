//! Askama view structs for super admin pages.
//!
//! Each struct renders a thin HTML shell: <head>, Vite assets, CSRF meta,
//! __INITIAL_DATA__, __PAGE__, and <div id="app">. Vue/Vuetify renders
//! everything visible.

use askama::Template;
use askama_web::WebTemplate;

use super::vite;

// -- Login --

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/auth/login.html")]
pub struct LoginPage {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl LoginPage {
    pub fn new(csrf_token: String) -> Self {
        Self {
            csrf_token,
            page: "login",
            initial_data: "{}".to_string(),
            js_file: vite::js_file(),
            css_file: vite::css_file(),
        }
    }
}

// -- Authenticated pages share this helper --

fn authenticated_page(
    page: &'static str,
    csrf_token: String,
    initial_data: String,
) -> (String, &'static str, String, String, String) {
    (csrf_token, page, initial_data, vite::js_file(), vite::css_file())
}

// -- Tenants --

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/tenants/index.html")]
pub struct TenantIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl TenantIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("tenant-list", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/tenants/create.html")]
pub struct TenantCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl TenantCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("tenant-form", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/tenants/edit.html")]
pub struct TenantEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl TenantEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("tenant-form", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

// -- Settings --

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/settings/index.html")]
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
            authenticated_page("settings", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

// -- Agents --

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/agents/index.html")]
pub struct AgentIndex {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl AgentIndex {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("agent-list", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/agents/create.html")]
pub struct AgentCreate {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl AgentCreate {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("agent-form", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/agents/edit.html")]
pub struct AgentEdit {
    pub csrf_token: String,
    pub page: &'static str,
    pub initial_data: String,
    pub js_file: String,
    pub css_file: String,
}

impl AgentEdit {
    pub fn new(csrf_token: String, initial_data: String) -> Self {
        let (csrf_token, page, initial_data, js_file, css_file) =
            authenticated_page("agent-form", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

// -- Roles --

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/roles/index.html")]
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
            authenticated_page("role-list", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/roles/create.html")]
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
            authenticated_page("role-form", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/roles/edit.html")]
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
            authenticated_page("role-form", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}

// -- Account --

#[derive(Template, WebTemplate)]
#[template(path = "pages/super/account/edit.html")]
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
            authenticated_page("account-form", csrf_token, initial_data);
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}
