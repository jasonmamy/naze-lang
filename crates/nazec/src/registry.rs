//! Registry client for publishing, searching, and downloading packages.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum RegistryError {
    Http(String),
    NotFound(String),
    InvalidSemver(String),
    NoMatchingVersion { name: String, constraint: String },
    PublishFailed(String),
    Api(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "registry HTTP error: {e}"),
            Self::NotFound(name) => write!(f, "package '{name}' not found in registry"),
            Self::InvalidSemver(s) => write!(f, "invalid semver constraint: {s}"),
            Self::NoMatchingVersion { name, constraint } => {
                write!(f, "no version of '{name}' matches constraint '{constraint}'")
            }
            Self::PublishFailed(e) => write!(f, "publish failed: {e}"),
            Self::Api(e) => write!(f, "registry API error: {e}"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: String,
    pub latest_version: String,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub checksum: String,
    pub naze_files: i64,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    packages: Vec<SearchPackage>,
}

#[derive(Debug, Deserialize)]
pub struct SearchPackage {
    pub name: String,
    pub description: String,
    pub latest_version: String,
}

#[derive(Debug, Serialize)]
struct PublishMetadata {
    name: String,
    version: String,
    description: String,
}

/// Resolved version from the registry.
pub struct ResolvedVersion {
    pub version: String,
    pub checksum: String,
}

// ─── Client ─────────────────────────────────────────────────────────────────

pub struct RegistryClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl RegistryClient {
    pub fn new(registry_url: Option<&str>) -> Self {
        let base_url = registry_url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("NAZE_REGISTRY_URL").ok())
            .unwrap_or_else(|| "https://registry.naze.dev/api/v1".to_string());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn get_package(&self, name: &str) -> Result<PackageInfo, RegistryError> {
        let url = format!("{}/packages/{}", self.base_url, name);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::Http(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(RegistryError::NotFound(name.to_string()));
        }
        if !resp.status().is_success() {
            return Err(RegistryError::Api(format!(
                "status {}: {}",
                resp.status(),
                resp.text().unwrap_or_default()
            )));
        }

        resp.json::<PackageInfo>()
            .map_err(|e| RegistryError::Api(e.to_string()))
    }

    /// Resolve the best matching version for a semver constraint.
    pub fn resolve_version(
        &self,
        name: &str,
        constraint: &str,
    ) -> Result<ResolvedVersion, RegistryError> {
        let req = semver::VersionReq::parse(constraint)
            .map_err(|e| RegistryError::InvalidSemver(format!("{constraint}: {e}")))?;

        let info = self.get_package(name)?;

        let mut best: Option<(semver::Version, &VersionInfo)> = None;
        for vi in &info.versions {
            if let Ok(sv) = semver::Version::parse(&vi.version) {
                if req.matches(&sv) {
                    if best.as_ref().map_or(true, |(bv, _)| sv > *bv) {
                        best = Some((sv, vi));
                    }
                }
            }
        }

        match best {
            Some((_sv, vi)) => Ok(ResolvedVersion {
                version: vi.version.clone(),
                checksum: vi.checksum.clone(),
            }),
            None => Err(RegistryError::NoMatchingVersion {
                name: name.to_string(),
                constraint: constraint.to_string(),
            }),
        }
    }

    /// Download a package tarball and extract it to `dest`.
    pub fn download_and_extract(
        &self,
        name: &str,
        version: &str,
        dest: &Path,
    ) -> Result<PathBuf, RegistryError> {
        let url = format!("{}/packages/{}/{}/download", self.base_url, name, version);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RegistryError::Api(format!(
                "download failed: status {}",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .map_err(|e| RegistryError::Http(e.to_string()))?;

        std::fs::create_dir_all(dest)
            .map_err(|e| RegistryError::Api(format!("failed to create dir: {e}")))?;

        extract_tarball(&bytes, dest)
            .map_err(|e| RegistryError::Api(format!("failed to extract tarball: {e}")))?;

        Ok(dest.to_path_buf())
    }

    /// Publish a package tarball to the registry.
    pub fn publish(
        &self,
        name: &str,
        version: &str,
        description: &str,
        tarball: &[u8],
    ) -> Result<(), RegistryError> {
        let metadata = serde_json::to_string(&PublishMetadata {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
        })
        .map_err(|e| RegistryError::PublishFailed(e.to_string()))?;

        let form = reqwest::blocking::multipart::Form::new()
            .text("metadata", metadata)
            .part(
                "tarball",
                reqwest::blocking::multipart::Part::bytes(tarball.to_vec())
                    .file_name(format!("{name}-{version}.tar.gz"))
                    .mime_str("application/gzip")
                    .unwrap(),
            );

        let url = format!("{}/packages", self.base_url);
        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .map_err(|e| RegistryError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(RegistryError::PublishFailed(text));
        }

        Ok(())
    }

    pub fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchPackage>, RegistryError> {
        let url = format!("{}/search?q={}&limit={}", self.base_url, query, limit);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| RegistryError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RegistryError::Api(format!("status {}", resp.status())));
        }

        let sr: SearchResponse = resp
            .json()
            .map_err(|e| RegistryError::Api(e.to_string()))?;
        Ok(sr.packages)
    }
}

// ─── Tarball helpers ────────────────────────────────────────────────────────

fn extract_tarball(data: &[u8], dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

/// Create a tarball from a package directory (collects .naze files and README.md).
pub fn create_package_tarball(dir: &Path) -> Result<(Vec<u8>, usize), Box<dyn std::error::Error>> {
    let mut builder = tar::Builder::new(Vec::new());
    let mut naze_count = 0;

    for entry in walkdir(dir)? {
        let rel = entry.strip_prefix(dir)?;
        let name = rel.to_string_lossy();
        if name.ends_with(".naze") {
            naze_count += 1;
            builder.append_path_with_name(&entry, rel)?;
        } else if name == "README.md" || name == "naze.toml" {
            builder.append_path_with_name(&entry, rel)?;
        }
    }

    builder.finish()?;
    let tar_data = builder.into_inner()?;

    // gzip compress
    use flate2::write::GzEncoder;
    use std::io::Write;
    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&tar_data)?;
    let gz_data = encoder.finish()?;

    Ok((gz_data, naze_count))
}

fn walkdir(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    walk_recursive(dir, &mut files)?;
    Ok(files)
}

fn walk_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            // Skip hidden dirs and common non-package dirs
            if !name.starts_with('.') && name != "dist" && name != "target" {
                walk_recursive(&path, files)?;
            }
        } else {
            files.push(path);
        }
    }
    Ok(())
}

/// Compute sha256 of bytes, returning hex string.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(data))
}

// ─── High-level CLI operations ──────────────────────────────────────────────

/// Publish the current project to a registry.
pub fn publish_package(registry_url: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = crate::manifest::load("naze.toml")?;
    let project_dir = Path::new(".");

    eprintln!(
        "packaging {} v{}...",
        manifest.app.name, manifest.app.version
    );

    let (tarball, naze_count) = create_package_tarball(project_dir)?;
    if naze_count == 0 {
        return Err("no .naze files found in project".into());
    }

    let checksum = sha256_hex(&tarball);
    eprintln!(
        "  {} .naze files, {:.1}KB tarball (sha256: {}...)",
        naze_count,
        tarball.len() as f64 / 1024.0,
        &checksum[..12]
    );

    let client = RegistryClient::new(registry_url);
    client.publish(
        &manifest.app.name,
        &manifest.app.version,
        "",
        &tarball,
    )?;

    eprintln!(
        "published {} v{} to registry",
        manifest.app.name, manifest.app.version
    );
    Ok(())
}

/// Search the registry and print results.
pub fn search_packages(
    query: &str,
    limit: u32,
    registry_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = RegistryClient::new(registry_url);
    let packages = client.search(query, limit)?;

    if packages.is_empty() {
        eprintln!("no packages found matching '{query}'");
        return Ok(());
    }

    for pkg in &packages {
        eprintln!(
            "  {} v{} — {}",
            pkg.name,
            pkg.latest_version,
            if pkg.description.is_empty() {
                "(no description)"
            } else {
                &pkg.description
            }
        );
    }
    eprintln!("{} package(s) found", packages.len());
    Ok(())
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_tarball_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let pkg_dir = tmp.path().join("my-pkg");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("app.naze"), "app \"test\" {}").unwrap();
        std::fs::write(pkg_dir.join("lib.naze"), "component foo() {}").unwrap();
        std::fs::write(pkg_dir.join("README.md"), "# My Package").unwrap();
        std::fs::write(pkg_dir.join("naze.toml"), "[app]\nname = \"test\"").unwrap();
        // This file should be ignored
        std::fs::write(pkg_dir.join("random.txt"), "ignore me").unwrap();

        let (tarball, naze_count) = create_package_tarball(&pkg_dir).unwrap();
        assert_eq!(naze_count, 2);
        assert!(!tarball.is_empty());

        // Extract and verify
        let extract_dir = tmp.path().join("extracted");
        extract_tarball(&tarball, &extract_dir).unwrap();
        assert!(extract_dir.join("app.naze").exists());
        assert!(extract_dir.join("lib.naze").exists());
        assert!(extract_dir.join("README.md").exists());
        assert!(extract_dir.join("naze.toml").exists());
        assert!(!extract_dir.join("random.txt").exists());
    }

    #[test]
    fn test_semver_resolution_logic() {
        // Test that semver constraint parsing works
        let req = semver::VersionReq::parse("^1.0").unwrap();
        assert!(req.matches(&semver::Version::parse("1.0.0").unwrap()));
        assert!(req.matches(&semver::Version::parse("1.5.3").unwrap()));
        assert!(!req.matches(&semver::Version::parse("2.0.0").unwrap()));

        let req = semver::VersionReq::parse(">=0.2, <0.4").unwrap();
        assert!(!req.matches(&semver::Version::parse("0.1.0").unwrap()));
        assert!(req.matches(&semver::Version::parse("0.3.0").unwrap()));
        assert!(!req.matches(&semver::Version::parse("0.4.0").unwrap()));
    }

    #[test]
    fn test_registry_error_display() {
        let e = RegistryError::NotFound("my-pkg".to_string());
        assert!(e.to_string().contains("my-pkg"));

        let e = RegistryError::NoMatchingVersion {
            name: "foo".to_string(),
            constraint: "^2.0".to_string(),
        };
        assert!(e.to_string().contains("foo"));
        assert!(e.to_string().contains("^2.0"));
    }
}
