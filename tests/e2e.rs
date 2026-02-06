use std::fs;
use std::path::PathBuf;

use nexus::cache::CacheState;
use nexus::context::build_handshake;
use nexus::vector::{embed, LocalVectorStore, VectorDocument, VectorStore};

fn temp_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "nexus-e2e-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn builds_handshake_and_payload() {
    let root = temp_root("handshake");
    fs::write(root.join("alpha.txt"), "hello").unwrap();
    let mut cache = CacheState::new(root.clone());
    cache.warm().unwrap();
    let handshake = build_handshake(&cache);
    assert!(handshake.file_count >= 1);
}

#[test]
fn vector_store_round_trip() {
    let mut store = LocalVectorStore::default();
    let doc = VectorDocument {
        id: "doc-1".to_string(),
        content: "Hello".to_string(),
        embedding: embed("Hello"),
        metadata: Default::default(),
    };
    store.upsert(vec![doc]).unwrap();
    let matches = store.query("Hello", 1).unwrap();
    assert_eq!(matches.len(), 1);
}
