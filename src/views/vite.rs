//! Vite manifest reader.
//!
//! Reads the Vite build manifest to get hashed CSS/JS filenames
//! for Askama templates.

use once_cell::sync::Lazy;
use std::sync::RwLock;

struct ViteAssets {
    js_file: String,
    css_file: String,
}

static ASSETS: Lazy<RwLock<ViteAssets>> = Lazy::new(|| {
    let assets = load_manifest().unwrap_or_else(|e| {
        tracing::warn!("Failed to load Vite manifest: {e}. Using fallback paths.");
        ViteAssets {
            js_file: "assets/main.js".to_string(),
            css_file: "assets/main.css".to_string(),
        }
    });
    RwLock::new(assets)
});

fn load_manifest() -> anyhow::Result<ViteAssets> {
    let manifest_path = "static/dist/.vite/manifest.json";
    let content = std::fs::read_to_string(manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&content)?;

    let entry = manifest
        .get("src/main.ts")
        .ok_or_else(|| anyhow::anyhow!("Missing src/main.ts entry in manifest"))?;

    let js_file = entry["file"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file field"))?
        .to_string();

    let css_file = entry["css"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing css field"))?
        .to_string();

    Ok(ViteAssets { js_file, css_file })
}

/// Get the hashed JS filename from the Vite manifest.
pub fn js_file() -> String {
    ASSETS.read().unwrap().js_file.clone()
}

/// Get the hashed CSS filename from the Vite manifest.
pub fn css_file() -> String {
    ASSETS.read().unwrap().css_file.clone()
}
