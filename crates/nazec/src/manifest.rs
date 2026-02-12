use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub app: App,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    #[serde(default)]
    pub seo: Seo,
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    #[serde(default)]
    pub env: HashMap<String, EnvSpec>,
}

/// Environment variable specification.
/// Simple form: `API_URL = "http://localhost:3000"` (string default)
/// Detailed form: `SECRET = { from = "SECRET_KEY", required = true }` (runtime env var)
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnvSpec {
    Default(String),
    Detailed(EnvDetailedSpec),
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvDetailedSpec {
    pub from: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Version(String),
    Detailed(DetailedDep),
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetailedDep {
    pub path: Option<String>,
    pub git: Option<String>,
    pub tag: Option<String>,
    pub branch: Option<String>,
    pub rev: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Seo {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub keywords: Option<String>,
    #[serde(default)]
    pub canonical: Option<String>,
    #[serde(default)]
    pub twitter: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
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

/// Load a `.env` file (simple KEY=VALUE format, one per line).
/// Lines starting with `#` and empty lines are ignored.
pub fn load_dotenv(path: impl AsRef<Path>) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path.as_ref()) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                // Strip surrounding quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    &value[1..value.len() - 1]
                } else {
                    value
                };
                vars.insert(key.to_string(), value.to_string());
            }
        }
    }
    vars
}

/// Resolve env vars from manifest [env] section using priority:
/// 1. Process environment (`std::env::var`)
/// 2. `.env` file overrides
/// 3. Manifest defaults
///
/// Returns `(resolved_vars, missing_required)` where missing_required
/// lists env var names that are required but not found.
pub fn resolve_env_vars(
    manifest: &Manifest,
    dotenv: &HashMap<String, String>,
) -> (HashMap<String, String>, Vec<String>) {
    let mut resolved = HashMap::new();
    let mut missing = Vec::new();

    for (name, spec) in &manifest.env {
        let (env_key, default, required) = match spec {
            EnvSpec::Default(val) => (name.as_str(), Some(val.as_str()), false),
            EnvSpec::Detailed(d) => (d.from.as_str(), None, d.required),
        };

        // Priority: process env > .env file > manifest default
        let value = std::env::var(env_key)
            .ok()
            .or_else(|| dotenv.get(env_key).cloned())
            .or_else(|| default.map(|s| s.to_string()));

        match value {
            Some(v) => {
                resolved.insert(name.clone(), v);
            }
            None => {
                if required {
                    missing.push(name.clone());
                } else {
                    resolved.insert(name.clone(), String::new());
                }
            }
        }
    }

    (resolved, missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_section_simple_default() {
        let toml_str = r#"
[app]
name = "test"
version = "0.1.0"

[env]
API_URL = "http://localhost:3000"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.env.len(), 1);
        match &manifest.env["API_URL"] {
            EnvSpec::Default(val) => assert_eq!(val, "http://localhost:3000"),
            _ => panic!("expected Default variant"),
        }
    }

    #[test]
    fn parse_env_section_detailed() {
        let toml_str = r#"
[app]
name = "test"

[env]
SECRET = { from = "SECRET_KEY", required = true }
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        match &manifest.env["SECRET"] {
            EnvSpec::Detailed(d) => {
                assert_eq!(d.from, "SECRET_KEY");
                assert!(d.required);
            }
            _ => panic!("expected Detailed variant"),
        }
    }

    #[test]
    fn parse_env_section_mixed() {
        let toml_str = r#"
[app]
name = "test"

[env]
API_URL = "http://localhost:3000"
SECRET = { from = "SECRET_KEY", required = true }
OPTIONAL = { from = "OPT_VAR" }
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.env.len(), 3);
    }

    #[test]
    fn resolve_env_uses_default() {
        let toml_str = r#"
[app]
name = "test"

[env]
API_URL = "http://localhost:3000"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        let dotenv = HashMap::new();
        let (resolved, missing) = resolve_env_vars(&manifest, &dotenv);
        assert!(missing.is_empty());
        assert_eq!(resolved["API_URL"], "http://localhost:3000");
    }

    #[test]
    fn resolve_env_dotenv_overrides_default() {
        let toml_str = r#"
[app]
name = "test"

[env]
API_URL = "http://localhost:3000"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        let mut dotenv = HashMap::new();
        dotenv.insert("API_URL".to_string(), "http://prod.example.com".to_string());
        let (resolved, missing) = resolve_env_vars(&manifest, &dotenv);
        assert!(missing.is_empty());
        assert_eq!(resolved["API_URL"], "http://prod.example.com");
    }

    #[test]
    fn resolve_env_missing_required() {
        let toml_str = r#"
[app]
name = "test"

[env]
SECRET = { from = "NAZE_TEST_MISSING_VAR_XYZ", required = true }
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        let dotenv = HashMap::new();
        let (_, missing) = resolve_env_vars(&manifest, &dotenv);
        assert_eq!(missing, vec!["SECRET"]);
    }

    #[test]
    fn resolve_env_detailed_from_dotenv() {
        let toml_str = r#"
[app]
name = "test"

[env]
SECRET = { from = "MY_SECRET_KEY" }
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        let mut dotenv = HashMap::new();
        dotenv.insert("MY_SECRET_KEY".to_string(), "s3cret".to_string());
        let (resolved, missing) = resolve_env_vars(&manifest, &dotenv);
        assert!(missing.is_empty());
        assert_eq!(resolved["SECRET"], "s3cret");
    }

    #[test]
    fn load_dotenv_parses_file() {
        let dir = std::env::temp_dir().join("naze_test_dotenv");
        let _ = std::fs::create_dir_all(&dir);
        let env_file = dir.join(".env");
        std::fs::write(&env_file, "# comment\nKEY1=value1\nKEY2=\"quoted\"\nKEY3='single'\n\nKEY4 = spaced\n").unwrap();
        let vars = load_dotenv(&env_file);
        assert_eq!(vars["KEY1"], "value1");
        assert_eq!(vars["KEY2"], "quoted");
        assert_eq!(vars["KEY3"], "single");
        assert_eq!(vars["KEY4"], "spaced");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_env_section_is_empty() {
        let toml_str = r#"
[app]
name = "test"
"#;
        let manifest: Manifest = toml::from_str(toml_str).unwrap();
        assert!(manifest.env.is_empty());
    }
}
