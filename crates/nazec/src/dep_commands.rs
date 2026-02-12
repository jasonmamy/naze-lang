use std::path::Path;

use crate::deps;
use crate::manifest;

/// Add a dependency to naze.toml and resolve it.
pub fn add(
    package: &str,
    path: Option<&str>,
    git: Option<&str>,
    tag: Option<&str>,
    branch: Option<&str>,
    rev: Option<&str>,
    version: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let toml_path = Path::new("naze.toml");
    let content = std::fs::read_to_string(toml_path)?;
    let mut doc: toml_edit::DocumentMut = content.parse()?;

    // Ensure [dependencies] table exists
    if doc.get("dependencies").is_none() {
        doc["dependencies"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    let deps_table = doc["dependencies"].as_table_mut().ok_or("invalid [dependencies] section")?;

    if path.is_some() || git.is_some() {
        // Detailed dependency
        let mut inline = toml_edit::InlineTable::new();
        if let Some(p) = path {
            inline.insert("path", p.into());
        }
        if let Some(g) = git {
            inline.insert("git", g.into());
        }
        if let Some(t) = tag {
            inline.insert("tag", t.into());
        }
        if let Some(b) = branch {
            inline.insert("branch", b.into());
        }
        if let Some(r) = rev {
            inline.insert("rev", r.into());
        }
        deps_table.insert(package, toml_edit::value(inline));
    } else if let Some(v) = version {
        // Version-only dependency (registry)
        deps_table.insert(package, toml_edit::value(v));
    } else {
        // No source specified — default to wildcard version (registry)
        deps_table.insert(package, toml_edit::value("*"));
    }

    std::fs::write(toml_path, doc.to_string())?;
    eprintln!("added dependency '{}'", package);

    // Resolve to validate and update lockfile
    let manifest = manifest::load(toml_path)?;
    deps::resolve_deps(&manifest, Path::new("."))?;
    eprintln!("resolved successfully");

    Ok(())
}

/// Remove a dependency from naze.toml.
pub fn remove(package: &str) -> Result<(), Box<dyn std::error::Error>> {
    let toml_path = Path::new("naze.toml");
    let content = std::fs::read_to_string(toml_path)?;
    let mut doc: toml_edit::DocumentMut = content.parse()?;

    if let Some(deps_table) = doc.get_mut("dependencies").and_then(|d| d.as_table_mut()) {
        if deps_table.remove(package).is_some() {
            std::fs::write(toml_path, doc.to_string())?;
            eprintln!("removed dependency '{}'", package);

            // Update lockfile
            let manifest = manifest::load(toml_path)?;
            deps::resolve_deps(&manifest, Path::new("."))?;
        } else {
            return Err(format!("dependency '{}' not found in [dependencies]", package).into());
        }
    } else {
        return Err("no [dependencies] section in naze.toml".into());
    }

    Ok(())
}

/// Update dependencies (re-fetch git deps to latest matching version).
pub fn update(package: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let toml_path = Path::new("naze.toml");
    let manifest = manifest::load(toml_path)?;

    if manifest.dependencies.is_empty() {
        eprintln!("no dependencies to update");
        return Ok(());
    }

    if let Some(name) = package {
        if !manifest.dependencies.contains_key(name) {
            return Err(format!("dependency '{}' not found in [dependencies]", name).into());
        }
    }

    // Delete git caches to force re-fetch
    let cache_dir = Path::new(".").join(".nazec").join("deps");
    if cache_dir.exists() {
        if let Some(name) = package {
            // Delete only the specific package cache
            let entries = std::fs::read_dir(&cache_dir)?;
            let sanitized = name.replace('@', "").replace('/', "-");
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with(&sanitized) {
                    std::fs::remove_dir_all(entry.path())?;
                    eprintln!("cleared cache for '{}'", name);
                }
            }
        } else {
            // Delete all git caches
            std::fs::remove_dir_all(&cache_dir)?;
            eprintln!("cleared all dependency caches");
        }
    }

    // Re-resolve (will re-clone git deps)
    // Remove lockfile to force fresh resolution
    let lock_path = Path::new(".").join("naze.lock");
    if lock_path.exists() {
        std::fs::remove_file(&lock_path)?;
    }

    deps::resolve_deps(&manifest, Path::new("."))?;
    eprintln!("dependencies updated");

    Ok(())
}
