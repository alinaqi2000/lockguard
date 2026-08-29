use crate::error::{Error, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct ComposerLock {
    pub packages: Vec<LockPackage>,
    #[serde(default, rename = "packages-dev")]
    pub packages_dev: Vec<LockPackage>,
}

#[derive(Deserialize, Debug)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
}

pub fn read_lock(path: &Path) -> Result<ComposerLock> {
    let content = std::fs::read_to_string(path).map_err(|e| Error::LockRead {
        path: path.to_path_buf(),
        source: e,
    })?;

    serde_json::from_str::<ComposerLock>(&content).map_err(|e| Error::LockParse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

pub fn normalize(lock: &ComposerLock, path: &Path) -> Result<Vec<LockedPackage>> {
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut result: Vec<LockedPackage> = Vec::new();

    for pkg in lock.packages.iter().chain(lock.packages_dev.iter()) {
        let name = pkg.name.trim().to_lowercase();
        let version = pkg.version.trim().to_string();

        if name.is_empty() || version.is_empty() {
            return Err(Error::LockEmptyIdentity {
                path: path.to_path_buf(),
            });
        }

        if let Some(existing) = seen.get(&name) {
            if existing != &version {
                return Err(Error::LockConflict {
                    path: path.to_path_buf(),
                    package: name,
                    first: existing.clone(),
                    second: version,
                });
            }
        } else {
            seen.insert(name.clone(), version.clone());
            result.push(LockedPackage { name, version });
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, version: &str) -> LockPackage {
        LockPackage {
            name: name.to_string(),
            version: version.to_string(),
        }
    }

    fn lock(packages: Vec<LockPackage>, dev: Vec<LockPackage>) -> ComposerLock {
        ComposerLock {
            packages,
            packages_dev: dev,
        }
    }

    #[test]
    fn normalize_basic() {
        let l = lock(vec![pkg("monolog/monolog", "2.3.0")], vec![]);
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "monolog/monolog");
        assert_eq!(result[0].version, "2.3.0");
    }

    #[test]
    fn normalize_merges_scopes() {
        let l = lock(
            vec![pkg("vendor/prod", "1.0.0")],
            vec![pkg("vendor/dev", "2.0.0")],
        );
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "vendor/dev");
        assert_eq!(result[1].name, "vendor/prod");
    }

    #[test]
    fn normalize_dedup_same_version() {
        let l = lock(
            vec![pkg("vendor/pkg", "1.0.0")],
            vec![pkg("vendor/pkg", "1.0.0")],
        );
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn normalize_conflict_different_versions() {
        let l = lock(
            vec![pkg("vendor/pkg", "1.0.0")],
            vec![pkg("vendor/pkg", "2.0.0")],
        );
        let err = normalize(&l, Path::new("test.lock")).unwrap_err();
        assert!(matches!(err, Error::LockConflict { .. }));
    }

    #[test]
    fn normalize_lowercases_names() {
        let l = lock(vec![pkg("Vendor/Package", "1.0.0")], vec![]);
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert_eq!(result[0].name, "vendor/package");
    }

    #[test]
    fn normalize_dedup_case_insensitive() {
        let l = lock(
            vec![pkg("Vendor/Pkg", "1.0.0")],
            vec![pkg("vendor/pkg", "1.0.0")],
        );
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn normalize_empty_name_rejected() {
        let l = lock(vec![pkg("  ", "1.0.0")], vec![]);
        let err = normalize(&l, Path::new("test.lock")).unwrap_err();
        assert!(matches!(err, Error::LockEmptyIdentity { .. }));
    }

    #[test]
    fn normalize_empty_version_rejected() {
        let l = lock(vec![pkg("vendor/pkg", "")], vec![]);
        let err = normalize(&l, Path::new("test.lock")).unwrap_err();
        assert!(matches!(err, Error::LockEmptyIdentity { .. }));
    }

    #[test]
    fn normalize_sorted_alphabetically() {
        let l = lock(
            vec![pkg("zzz/last", "1.0.0"), pkg("aaa/first", "1.0.0")],
            vec![],
        );
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert_eq!(result[0].name, "aaa/first");
        assert_eq!(result[1].name, "zzz/last");
    }

    #[test]
    fn normalize_empty_lock() {
        let l = lock(vec![], vec![]);
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn normalize_trims_whitespace() {
        let l = lock(vec![pkg("  vendor/pkg  ", "  1.0.0  ")], vec![]);
        let result = normalize(&l, Path::new("test.lock")).unwrap();
        assert_eq!(result[0].name, "vendor/pkg");
        assert_eq!(result[0].version, "1.0.0");
    }

    #[test]
    fn read_lock_valid() {
        let json = r#"{"packages":[{"name":"monolog/monolog","version":"2.3.0"}]}"#;
        let temp = tempfile();
        std::fs::write(&temp, json).unwrap();
        let lock = read_lock(&temp).unwrap();
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].name, "monolog/monolog");
    }

    #[test]
    fn read_lock_with_dev() {
        let json = r#"{
            "packages":[{"name":"vendor/prod","version":"1.0.0"}],
            "packages-dev":[{"name":"vendor/dev","version":"2.0.0"}]
        }"#;
        let temp = tempfile();
        std::fs::write(&temp, json).unwrap();
        let lock = read_lock(&temp).unwrap();
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages_dev.len(), 1);
    }

    #[test]
    fn read_lock_missing_dev_defaults_empty() {
        let json = r#"{"packages":[{"name":"vendor/prod","version":"1.0.0"}]}"#;
        let temp = tempfile();
        std::fs::write(&temp, json).unwrap();
        let lock = read_lock(&temp).unwrap();
        assert!(lock.packages_dev.is_empty());
    }

    #[test]
    fn read_lock_missing_file() {
        let err = read_lock(Path::new("/nonexistent/path.lock")).unwrap_err();
        assert!(matches!(err, Error::LockRead { .. }));
    }

    #[test]
    fn read_lock_invalid_json() {
        let temp = tempfile();
        std::fs::write(&temp, "not json").unwrap();
        let err = read_lock(&temp).unwrap_err();
        assert!(matches!(err, Error::LockParse { .. }));
    }

    #[test]
    fn read_lock_missing_packages() {
        let json = r#"{"packages-dev":[]}"#;
        let temp = tempfile();
        std::fs::write(&temp, json).unwrap();
        let err = read_lock(&temp).unwrap_err();
        assert!(matches!(err, Error::LockParse { .. }));
    }

    #[test]
    fn read_lock_extra_fields_ignored() {
        let json = r#"{
            "packages":[{"name":"vendor/pkg","version":"1.0.0","dist":{"url":"..."},"source":{"url":"..."}}],
            "packages-dev":[],
            "content-hash":"abc123",
            "_readme":["extra"]
        }"#;
        let temp = tempfile();
        std::fs::write(&temp, json).unwrap();
        let lock = read_lock(&temp).unwrap();
        assert_eq!(lock.packages[0].name, "vendor/pkg");
    }

    fn tempfile() -> std::path::PathBuf {
        let dir = std::env::temp_dir();
        let mut path = dir;
        path.push(format!(
            "lockguard_test_{}_{}.lock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        path
    }
}
