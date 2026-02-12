use std::collections::HashMap;
use std::path::{Path, PathBuf};

use naze_compiler::resolve::ResolvedDep;

use crate::manifest::{DependencySpec, DetailedDep, Manifest};

/// Source of a resolved dependency (for lockfile).
#[derive(Debug, Clone)]
pub enum DepSource {
    Path(PathBuf),
    Git { url: String, resolved_rev: String },
    Registry { version: String, checksum: String },
}

/// A fully resolved dependency with its local path and source info.
#[derive(Debug, Clone)]
pub struct FullResolvedDep {
    pub name: String,
    pub local_path: PathBuf,
    pub source: DepSource,
}

/// Lockfile entry for a single package.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LockEntry {
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Complete lockfile structure.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Lockfile {
    pub schema_version: u32,
    #[serde(default)]
    pub package: Vec<LockEntry>,
}

/// Resolve all dependencies from the manifest.
/// Returns compiler-compatible `ResolvedDep` list.
pub fn resolve_deps(
    manifest: &Manifest,
    project_dir: &Path,
) -> Result<Vec<ResolvedDep>, Box<dyn std::error::Error>> {
    if manifest.dependencies.is_empty() {
        return Ok(vec![]);
    }

    let lock = read_lockfile(project_dir);
    let lock_entries: HashMap<String, &LockEntry> = lock
        .as_ref()
        .map(|l| l.package.iter().map(|e| (e.name.clone(), e)).collect())
        .unwrap_or_default();

    let cache_dir = project_dir.join(".nazec").join("deps");
    let mut resolved = Vec::new();
    let mut lock_updates = Vec::new();

    for (name, spec) in &manifest.dependencies {
        let full = resolve_single(
            name,
            spec,
            project_dir,
            &cache_dir,
            lock_entries.get(name.as_str()).copied(),
        )?;
        lock_updates.push(to_lock_entry(&full, spec));
        resolved.push(ResolvedDep {
            name: full.name,
            local_path: full.local_path,
        });
    }

    // Write updated lockfile
    let new_lock = Lockfile {
        schema_version: 1,
        package: lock_updates,
    };
    write_lockfile(project_dir, &new_lock)?;

    Ok(resolved)
}

fn resolve_single(
    name: &str,
    spec: &DependencySpec,
    project_dir: &Path,
    cache_dir: &Path,
    lock_entry: Option<&LockEntry>,
) -> Result<FullResolvedDep, Box<dyn std::error::Error>> {
    match spec {
        DependencySpec::Version(v) => resolve_registry_dep(name, v, cache_dir, lock_entry),
        DependencySpec::Detailed(detail) => {
            resolve_detailed(name, detail, project_dir, cache_dir, lock_entry)
        }
    }
}

fn resolve_registry_dep(
    name: &str,
    constraint: &str,
    cache_dir: &Path,
    lock_entry: Option<&LockEntry>,
) -> Result<FullResolvedDep, Box<dyn std::error::Error>> {
    // Check if the lockfile already has a registry entry that satisfies the constraint
    if let Some(entry) = lock_entry {
        if entry.source == "registry" {
            if let (Some(locked_version), Some(locked_checksum)) = (&entry.version, &entry.checksum)
            {
                // Verify the locked version still satisfies the constraint
                if let (Ok(req), Ok(sv)) = (
                    semver::VersionReq::parse(constraint),
                    semver::Version::parse(locked_version),
                ) {
                    if req.matches(&sv) {
                        let dep_dir = cache_dir.join(format!(
                            "{}-registry-{}",
                            sanitize_name(name),
                            locked_version
                        ));
                        if dep_dir.exists() {
                            return Ok(FullResolvedDep {
                                name: name.to_string(),
                                local_path: dep_dir,
                                source: DepSource::Registry {
                                    version: locked_version.clone(),
                                    checksum: locked_checksum.clone(),
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    // Resolve from registry
    let client = crate::registry::RegistryClient::new(None);
    let resolved = client.resolve_version(name, constraint)?;

    let dep_dir = cache_dir.join(format!(
        "{}-registry-{}",
        sanitize_name(name),
        resolved.version
    ));

    if !dep_dir.exists() {
        std::fs::create_dir_all(cache_dir)?;
        client.download_and_extract(name, &resolved.version, &dep_dir)?;
    }

    Ok(FullResolvedDep {
        name: name.to_string(),
        local_path: dep_dir,
        source: DepSource::Registry {
            version: resolved.version,
            checksum: resolved.checksum,
        },
    })
}

fn resolve_detailed(
    name: &str,
    detail: &DetailedDep,
    project_dir: &Path,
    cache_dir: &Path,
    lock_entry: Option<&LockEntry>,
) -> Result<FullResolvedDep, Box<dyn std::error::Error>> {
    if let Some(path_str) = &detail.path {
        // Path dependency: resolve relative to project dir
        let dep_path = if Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            project_dir.join(path_str)
        };
        let dep_path = dep_path.canonicalize().map_err(|e| {
            format!(
                "dependency '{}': path '{}' not found: {}",
                name, path_str, e
            )
        })?;
        if !dep_path.is_dir() {
            return Err(format!(
                "dependency '{}': path '{}' is not a directory",
                name,
                dep_path.display()
            )
            .into());
        }
        Ok(FullResolvedDep {
            name: name.to_string(),
            local_path: dep_path.clone(),
            source: DepSource::Path(dep_path),
        })
    } else if let Some(git_url) = &detail.git {
        resolve_git_dep(name, git_url, detail, cache_dir, lock_entry)
    } else {
        Err(format!("dependency '{}': must specify either 'path' or 'git'", name).into())
    }
}

fn resolve_git_dep(
    name: &str,
    git_url: &str,
    detail: &DetailedDep,
    cache_dir: &Path,
    lock_entry: Option<&LockEntry>,
) -> Result<FullResolvedDep, Box<dyn std::error::Error>> {
    // Compute a stable cache directory name from the URL
    let url_hash = simple_hash(git_url);
    let dep_cache_dir = cache_dir.join(format!("{}-{:016x}", sanitize_name(name), url_hash));

    // Determine the target ref
    let pinned_rev = detail.rev.clone().or_else(|| {
        lock_entry.and_then(|e| {
            if e.url.as_deref() == Some(git_url) {
                e.rev.clone()
            } else {
                None
            }
        })
    });

    if dep_cache_dir.exists() {
        // Check if we need to update
        if let Some(rev) = &pinned_rev {
            let current = get_git_rev(&dep_cache_dir);
            if current.as_deref() == Some(rev.as_str()) {
                return Ok(FullResolvedDep {
                    name: name.to_string(),
                    local_path: dep_cache_dir,
                    source: DepSource::Git {
                        url: git_url.to_string(),
                        resolved_rev: rev.clone(),
                    },
                });
            }
        }
        // Stale or different ref — remove and re-clone
        std::fs::remove_dir_all(&dep_cache_dir)?;
    }

    std::fs::create_dir_all(cache_dir)?;

    // Clone with appropriate ref
    if let Some(tag) = &detail.tag {
        git_clone_ref(git_url, tag, &dep_cache_dir)?;
    } else if let Some(branch) = &detail.branch {
        git_clone_ref(git_url, branch, &dep_cache_dir)?;
    } else if let Some(rev) = &pinned_rev {
        git_clone_and_checkout(git_url, rev, &dep_cache_dir)?;
    } else {
        // Default branch, shallow clone
        git_clone_ref(git_url, "HEAD", &dep_cache_dir)?;
    }

    let resolved_rev = get_git_rev(&dep_cache_dir).unwrap_or_else(|| "unknown".to_string());

    Ok(FullResolvedDep {
        name: name.to_string(),
        local_path: dep_cache_dir,
        source: DepSource::Git {
            url: git_url.to_string(),
            resolved_rev,
        },
    })
}

fn git_clone_ref(url: &str, git_ref: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let dest_str = dest.to_string_lossy().to_string();
    let mut cmd = std::process::Command::new("git");
    cmd.args(["clone", "--depth", "1"]);
    if git_ref != "HEAD" {
        cmd.args(["--branch", git_ref]);
    }
    cmd.args([url, &dest_str]);

    let output = cmd
        .output()
        .map_err(|e| format!("failed to run git: {} (is git installed?)", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {}", stderr.trim()).into());
    }
    Ok(())
}

fn git_clone_and_checkout(
    url: &str,
    rev: &str,
    dest: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dest_str = dest.to_string_lossy().to_string();

    let output = std::process::Command::new("git")
        .args(["clone", url, &dest_str])
        .output()
        .map_err(|e| format!("failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {}", stderr.trim()).into());
    }

    let output = std::process::Command::new("git")
        .args(["checkout", rev])
        .current_dir(dest)
        .output()
        .map_err(|e| format!("git checkout failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git checkout {} failed: {}", rev, stderr.trim()).into());
    }

    Ok(())
}

fn get_git_rev(repo_dir: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn simple_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn sanitize_name(name: &str) -> String {
    name.replace('@', "").replace('/', "-")
}

fn to_lock_entry(dep: &FullResolvedDep, spec: &DependencySpec) -> LockEntry {
    match &dep.source {
        DepSource::Path(p) => LockEntry {
            name: dep.name.clone(),
            source: "path".to_string(),
            path: Some(p.to_string_lossy().to_string()),
            url: None,
            rev: None,
            tag: None,
            branch: None,
            version: None,
            checksum: None,
        },
        DepSource::Git { url, resolved_rev } => {
            let (tag, branch) = match spec {
                DependencySpec::Detailed(d) => (d.tag.clone(), d.branch.clone()),
                _ => (None, None),
            };
            LockEntry {
                name: dep.name.clone(),
                source: "git".to_string(),
                path: None,
                url: Some(url.clone()),
                rev: Some(resolved_rev.clone()),
                tag,
                branch,
                version: None,
                checksum: None,
            }
        }
        DepSource::Registry { version, checksum } => LockEntry {
            name: dep.name.clone(),
            source: "registry".to_string(),
            path: None,
            url: None,
            rev: None,
            tag: None,
            branch: None,
            version: Some(version.clone()),
            checksum: Some(checksum.clone()),
        },
    }
}

/// Read the lockfile from the project directory, if it exists.
pub fn read_lockfile(project_dir: &Path) -> Option<Lockfile> {
    let path = project_dir.join("naze.lock");
    let content = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}

/// Write the lockfile to the project directory.
pub fn write_lockfile(
    project_dir: &Path,
    lockfile: &Lockfile,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = project_dir.join("naze.lock");
    let content = toml::to_string_pretty(lockfile)?;
    let header = "# This file is auto-generated by nazec. Do not edit manually.\n\n";
    std::fs::write(&path, format!("{}{}", header, content))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{App, Build, Seo};
    use std::fs;

    fn test_manifest(deps: HashMap<String, DependencySpec>) -> Manifest {
        Manifest {
            app: App {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
            },
            build: Build::default(),
            scripts: HashMap::new(),
            seo: Seo::default(),
            dependencies: deps,
            env: HashMap::new(),
        }
    }

    #[test]
    fn resolve_empty_deps() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = test_manifest(HashMap::new());
        let deps = resolve_deps(&manifest, dir.path()).unwrap();
        assert!(deps.is_empty());
    }

    #[test]
    fn resolve_path_dep() {
        let project = tempfile::tempdir().unwrap();
        let lib_dir = tempfile::tempdir().unwrap();

        // Create a .naze file in the lib
        fs::write(
            lib_dir.path().join("button.naze"),
            "component button(label: text) {\n  text \"{label}\"\n}\n",
        )
        .unwrap();

        let mut deps_map = HashMap::new();
        deps_map.insert(
            "@test/lib".to_string(),
            DependencySpec::Detailed(DetailedDep {
                path: Some(lib_dir.path().to_string_lossy().to_string()),
                git: None,
                tag: None,
                branch: None,
                rev: None,
                version: None,
            }),
        );

        let manifest = test_manifest(deps_map);
        let deps = resolve_deps(&manifest, project.path()).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "@test/lib");
        assert!(deps[0].local_path.exists());
    }

    #[test]
    fn resolve_path_dep_relative() {
        let base = tempfile::tempdir().unwrap();
        let project_dir = base.path().join("my-app");
        let lib_dir = base.path().join("my-lib");

        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&lib_dir).unwrap();
        fs::write(
            lib_dir.join("card.naze"),
            "component card(title: text) {\n  text \"{title}\"\n}\n",
        )
        .unwrap();

        let mut deps_map = HashMap::new();
        deps_map.insert(
            "@local/ui".to_string(),
            DependencySpec::Detailed(DetailedDep {
                path: Some("../my-lib".to_string()),
                git: None,
                tag: None,
                branch: None,
                rev: None,
                version: None,
            }),
        );

        let manifest = test_manifest(deps_map);
        let deps = resolve_deps(&manifest, &project_dir).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "@local/ui");
    }

    #[test]
    fn resolve_missing_path_dep() {
        let project = tempfile::tempdir().unwrap();
        let mut deps_map = HashMap::new();
        deps_map.insert(
            "@missing/lib".to_string(),
            DependencySpec::Detailed(DetailedDep {
                path: Some("/nonexistent/path".to_string()),
                git: None,
                tag: None,
                branch: None,
                rev: None,
                version: None,
            }),
        );

        let manifest = test_manifest(deps_map);
        let result = resolve_deps(&manifest, project.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn resolve_version_only_spec_errors_without_registry() {
        let project = tempfile::tempdir().unwrap();
        let mut deps_map = HashMap::new();
        deps_map.insert(
            "@naze/ui".to_string(),
            DependencySpec::Version("^1.0".to_string()),
        );

        let manifest = test_manifest(deps_map);
        let result = resolve_deps(&manifest, project.path());
        // Without a running registry, this will fail with an HTTP/connection error
        assert!(result.is_err());
    }

    #[test]
    fn lockfile_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let lock = Lockfile {
            schema_version: 1,
            package: vec![
                LockEntry {
                    name: "@test/lib".to_string(),
                    source: "path".to_string(),
                    path: Some("../lib".to_string()),
                    url: None,
                    rev: None,
                    tag: None,
                    branch: None,
                    version: None,
                    checksum: None,
                },
                LockEntry {
                    name: "@test/remote".to_string(),
                    source: "git".to_string(),
                    path: None,
                    url: Some("https://github.com/test/remote.git".to_string()),
                    rev: Some("abc123".to_string()),
                    tag: Some("v1.0.0".to_string()),
                    branch: None,
                    version: None,
                    checksum: None,
                },
            ],
        };

        write_lockfile(dir.path(), &lock).unwrap();
        let read_back = read_lockfile(dir.path()).unwrap();
        assert_eq!(read_back.schema_version, 1);
        assert_eq!(read_back.package.len(), 2);
        assert_eq!(read_back.package[0].name, "@test/lib");
        assert_eq!(read_back.package[1].rev.as_deref(), Some("abc123"));
    }
}
