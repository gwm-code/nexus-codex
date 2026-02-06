use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub modified: Option<u64>,
    pub size: u64,
    pub hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheState {
    pub root: PathBuf,
    pub files: BTreeMap<String, FileMeta>,
}

impl CacheState {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: BTreeMap::new(),
        }
    }

    pub fn warm(&mut self) -> anyhow::Result<()> {
        self.files.clear();
        for entry in WalkDir::new(&self.root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let path = entry.path();
                let rel = path
                    .strip_prefix(&self.root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                let meta = metadata_for(path)?;
                self.files.insert(rel, meta);
            }
        }
        Ok(())
    }

    pub fn diff(&self, other: &CacheState) -> CacheDiff {
        let mut changed = Vec::new();
        let mut removed = Vec::new();

        for (path, meta) in &self.files {
            match other.files.get(path) {
                Some(other_meta) => {
                    if meta.hash != other_meta.hash || meta.size != other_meta.size {
                        changed.push(path.clone());
                    }
                }
                None => removed.push(path.clone()),
            }
        }

        for path in other.files.keys() {
            if !self.files.contains_key(path) {
                changed.push(path.clone());
            }
        }

        CacheDiff { changed, removed }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheDiff {
    pub changed: Vec<String>,
    pub removed: Vec<String>,
}

fn metadata_for(path: &Path) -> anyhow::Result<FileMeta> {
    let meta = fs::metadata(path)?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|dur| dur.as_secs());
    let size = meta.len();
    let hash = hash_file(path)?;

    Ok(FileMeta {
        modified,
        size,
        hash,
    })
}

fn hash_file(path: &Path) -> anyhow::Result<String> {
    let mut hasher = Hasher::new();
    let mut file = fs::File::open(path)?;
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_hex().to_string())
}
