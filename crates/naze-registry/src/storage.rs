use std::path::{Path, PathBuf};

pub struct Storage {
    packages_dir: PathBuf,
}

impl Storage {
    pub fn new(data_dir: &str) -> Self {
        let packages_dir = Path::new(data_dir).join("packages");
        std::fs::create_dir_all(&packages_dir).ok();
        Self { packages_dir }
    }

    fn sanitize_name(name: &str) -> String {
        name.replace('/', "__")
    }

    pub fn store_tarball(
        &self,
        name: &str,
        version: &str,
        data: &[u8],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let dir = self.packages_dir.join(Self::sanitize_name(name));
        std::fs::create_dir_all(&dir)?;
        let filename = format!("{version}.tar.gz");
        let path = dir.join(&filename);
        std::fs::write(&path, data)?;
        Ok(path.to_string_lossy().to_string())
    }

    pub fn get_tarball(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let dir = self.packages_dir.join(Self::sanitize_name(name));
        let path = dir.join(format!("{version}.tar.gz"));
        Ok(std::fs::read(path)?)
    }

    #[allow(dead_code)]
    pub fn tarball_exists(&self, name: &str, version: &str) -> bool {
        let dir = self.packages_dir.join(Self::sanitize_name(name));
        dir.join(format!("{version}.tar.gz")).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let tmp = std::env::temp_dir().join("naze-reg-test-storage");
        let _ = std::fs::remove_dir_all(&tmp);
        let store = Storage::new(tmp.to_str().unwrap());

        let data = b"fake tarball content";
        store.store_tarball("my-pkg", "1.0.0", data).unwrap();

        assert!(store.tarball_exists("my-pkg", "1.0.0"));
        assert!(!store.tarball_exists("my-pkg", "2.0.0"));

        let retrieved = store.get_tarball("my-pkg", "1.0.0").unwrap();
        assert_eq!(retrieved, data);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_scoped_package_name() {
        let tmp = std::env::temp_dir().join("naze-reg-test-scoped");
        let _ = std::fs::remove_dir_all(&tmp);
        let store = Storage::new(tmp.to_str().unwrap());

        store
            .store_tarball("@naze/ui-kit", "0.1.0", b"data")
            .unwrap();
        assert!(store.tarball_exists("@naze/ui-kit", "0.1.0"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
