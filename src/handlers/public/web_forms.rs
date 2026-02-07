use axum::extract::{Extension, Path};
use axum::response::{Html, IntoResponse, Response};

use crate::db::guard::TenantGuard;
use crate::db::Database;
use crate::models::company::Company;
use crate::models::web_form::{WebForm, WebFormAttributeRow};

/// Render the public web form as a standalone HTML page (no Vue, server-rendered).
pub async fn render_form(
    Extension(db): Extension<Database>,
    Extension(company): Extension<Company>,
    Path(form_id): Path<String>,
) -> Response {
    let mut guard = match TenantGuard::acquire(db.reader(), &company.schema_name).await {
        Ok(g) => g,
        Err(_) => return Html("<h1>Form not found</h1>".to_string()).into_response(),
    };

    let form = match guard
        .fetch_optional(sqlx::query_as::<_, WebForm>(
            "SELECT * FROM web_forms WHERE form_id = $1",
        ).bind(&form_id))
        .await
    {
        Ok(Some(f)) => f,
        _ => {
            let _ = guard.release().await;
            return Html("<h1>Form not found</h1>".to_string()).into_response();
        }
    };

    let attrs = guard
        .fetch_all(sqlx::query_as::<_, WebFormAttributeRow>(
            "SELECT wfa.id, wfa.name, wfa.placeholder, wfa.is_required, wfa.sort_order,
                    wfa.attribute_id, wfa.web_form_id,
                    a.name AS attribute_name, a.code AS attribute_code, a.type AS attribute_type
             FROM web_form_attributes wfa
             JOIN attributes a ON a.id = wfa.attribute_id
             WHERE wfa.web_form_id = $1
             ORDER BY wfa.sort_order, wfa.id",
        ).bind(form.id))
        .await
        .unwrap_or_default();

    let _ = guard.release().await;

    // Build form fields HTML
    let mut fields_html = String::new();
    for attr in &attrs {
        let label = attr.name.as_deref()
            .or(attr.attribute_name.as_deref())
            .unwrap_or("Field");
        let code = attr.attribute_code.as_deref().unwrap_or("field");
        let placeholder = attr.placeholder.as_deref().unwrap_or("");
        let required_attr = if attr.is_required { "required" } else { "" };
        let required_star = if attr.is_required { " *" } else { "" };
        let attr_type = attr.attribute_type.as_deref().unwrap_or("text");

        let input_type = match attr_type {
            "email" => "email",
            "phone" => "tel",
            "number" | "price" | "decimal" => "number",
            "date" => "date",
            "datetime" => "datetime-local",
            "boolean" | "checkbox" => "checkbox",
            "textarea" => "textarea",
            _ => "text",
        };

        let label_color = form.attribute_label_color.as_deref().unwrap_or("#546E7A");

        fields_html.push_str(&format!(
            r#"<div style="margin-bottom: 16px;">
                <label style="display: block; margin-bottom: 4px; font-size: 14px; color: {label_color};">{label}{required_star}</label>"#
        ));

        if input_type == "textarea" {
            fields_html.push_str(&format!(
                r#"<textarea name="{code}" placeholder="{placeholder}" {required_attr}
                    style="width: 100%; padding: 8px 12px; border: 1px solid #CFD8DC; border-radius: 4px; font-size: 14px; resize: vertical; min-height: 80px;"></textarea>"#
            ));
        } else if input_type == "checkbox" {
            fields_html.push_str(&format!(
                r#"<input type="checkbox" name="{code}" value="true"
                    style="margin-left: 4px;" />"#
            ));
        } else {
            fields_html.push_str(&format!(
                r#"<input type="{input_type}" name="{code}" placeholder="{placeholder}" {required_attr}
                    style="width: 100%; padding: 8px 12px; border: 1px solid #CFD8DC; border-radius: 4px; font-size: 14px;" />"#
            ));
        }

        fields_html.push_str("</div>");
    }

    let bg_color = form.background_color.as_deref().unwrap_or("#F7F8F9");
    let form_bg_color = form.form_background_color.as_deref().unwrap_or("#FFFFFF");
    let title_color = form.form_title_color.as_deref().unwrap_or("#263238");
    let button_color = form.form_submit_button_color.as_deref().unwrap_or("#0E90D9");
    let description = form.description.as_deref().unwrap_or("");

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background-color: {bg_color}; min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 20px; }}
        .form-container {{ background-color: {form_bg_color}; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); padding: 32px; max-width: 560px; width: 100%; }}
        h1 {{ color: {title_color}; font-size: 24px; margin-bottom: 8px; }}
        .description {{ color: #78909C; font-size: 14px; margin-bottom: 24px; }}
        .submit-btn {{ background-color: {button_color}; color: white; border: none; padding: 12px 24px; border-radius: 4px; font-size: 16px; cursor: pointer; width: 100%; }}
        .submit-btn:hover {{ opacity: 0.9; }}
        .submit-btn:disabled {{ opacity: 0.6; cursor: not-allowed; }}
        .success-msg {{ text-align: center; padding: 40px 20px; }}
        .success-msg h2 {{ color: #2E7D32; margin-bottom: 8px; }}
        .error-msg {{ color: #C62828; font-size: 14px; margin-bottom: 12px; }}
    </style>
</head>
<body>
    <div class="form-container" id="formContainer">
        <h1>{title}</h1>
        {desc_html}
        <div id="errorMsg" class="error-msg" style="display: none;"></div>
        <form id="webForm" onsubmit="return submitForm(event)">
            {fields_html}
            <button type="submit" class="submit-btn" id="submitBtn">{submit_label}</button>
        </form>
    </div>
    <script>
        async function submitForm(e) {{
            e.preventDefault();
            var btn = document.getElementById('submitBtn');
            var errDiv = document.getElementById('errorMsg');
            btn.disabled = true;
            errDiv.style.display = 'none';
            var formData = new FormData(document.getElementById('webForm'));
            var data = {{}};
            formData.forEach(function(v, k) {{ data[k] = v; }});
            try {{
                var resp = await fetch('/web-forms/{form_id}/submit', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(data),
                }});
                var result = await resp.json();
                if (result.success) {{
                    if (result.action === 'redirect' && result.redirect_url) {{
                        window.location.href = result.redirect_url;
                    }} else {{
                        document.getElementById('formContainer').innerHTML = '<div class="success-msg"><h2>&#10003;</h2><p>' + (result.message || 'Thank you!') + '</p></div>';
                    }}
                }} else {{
                    errDiv.textContent = result.error || 'Something went wrong.';
                    errDiv.style.display = 'block';
                    btn.disabled = false;
                }}
            }} catch (err) {{
                errDiv.textContent = 'Network error. Please try again.';
                errDiv.style.display = 'block';
                btn.disabled = false;
            }}
        }}
    </script>
</body>
</html>"#,
        title = html_escape(&form.title),
        desc_html = if description.is_empty() { String::new() } else { format!(r#"<p class="description">{}</p>"#, html_escape(description)) },
        fields_html = fields_html,
        submit_label = html_escape(&form.submit_button_label),
        form_id = html_escape(&form.form_id),
        bg_color = bg_color,
        form_bg_color = form_bg_color,
        title_color = title_color,
        button_color = button_color,
    );

    Html(html).into_response()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#39;")
}
