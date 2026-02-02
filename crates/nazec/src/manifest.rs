use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub app: App,
    #[serde(default)]
    pub build: Build,
}

#[derive(Debug, Clone, Deserialize)]
pub struct App {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Build {
    #[serde(default = "default_entry")]
    pub entry: String,
    #[serde(default = "default_output")]
    pub output: String,
}

impl Default for Build {
    fn default() -> Self {
        Self {
            entry: default_entry(),
            output: default_output(),
        }
    }
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_entry() -> String {
    "app.naze".to_string()
}

fn default_output() -> String {
    "dist/".to_string()
}

pub fn load(path: impl AsRef<Path>) -> Result<Manifest, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let manifest: Manifest = toml::from_str(&content)?;
    Ok(manifest)
}
