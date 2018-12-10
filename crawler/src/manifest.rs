use super::error::Result;
use super::task::{Task, Work};
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use url::Url;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Manifest {
    name: String,
    #[serde(with = "url_serde")]
    url: Url,
    main: String,
}

impl Manifest {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn main(&self) -> &str {
        &self.main
    }
}

#[derive(Debug, PartialEq)]
pub struct Package {
    root: PathBuf,
    manifest: Manifest,
}

impl Package {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn resolve_asset<T: AsRef<str>>(&self, path: T) -> PathBuf {
        self.root.join(path.as_ref())
    }

    pub fn load<T: AsRef<Path>>(root: T) -> Option<Package> {
        let path = root.as_ref();

        if !path.is_dir() {
            warn!("root path not a directory");
            return None;
        }

        let manifest_path = path.join("manifest.yaml");

        if !manifest_path.is_file() {
            warn!("manifest path is not a file: {:?}", manifest_path);
            return None;
        }

        let data = match fs::read_to_string(manifest_path) {
            Ok(m) => m,
            Err(_) => return None,
        };

        let manifest: Manifest = match serde_yaml::from_str(&data) {
            Err(_) => return None,
            Ok(m) => m,
        };

        Some(Package {
            root: path.to_path_buf(),
            manifest: manifest,
        })
    }

    pub fn task(&self) -> Result<Task> {
        Task::new(
            self.root().to_str().unwrap(),
            self.manifest.url().clone().as_str(),
            Work::path(self.manifest.main()),
        )
    }
}
