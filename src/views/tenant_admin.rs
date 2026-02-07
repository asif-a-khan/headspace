//! Askama view structs for tenant admin pages.

use askama::Template;
use askama_web::WebTemplate;

use super::vite;

fn authenticated_page(
    page: &'static str,
    csrf_token: String,
    initial_data: String,
) -> (String, &'static str, String, String, String) {
    (csrf_token, page, initial_data, vite::js_file(), vite::css_file())
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
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
        Self { csrf_token, page, initial_data, js_file, css_file }
    }
}
