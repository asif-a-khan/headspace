//! Askama view structs for tenant admin pages.

use askama::Template;
use askama_web::WebTemplate;

use super::vite;

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
        Self {
            csrf_token,
            page: "admin-dashboard",
            initial_data,
            js_file: vite::js_file(),
            css_file: vite::css_file(),
        }
    }
}
