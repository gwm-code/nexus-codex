use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMatch {
    pub id: String,
    pub score: f32,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorStoreSnapshot {
    pub documents: Vec<VectorDocument>,
}

pub trait VectorStore {
    fn upsert(&mut self, docs: Vec<VectorDocument>) -> anyhow::Result<()>;
    fn query(&self, query: &str, top_k: usize) -> anyhow::Result<Vec<VectorMatch>>;
}

#[derive(Debug, Default)]
pub struct LocalVectorStore {
    pub documents: Vec<VectorDocument>,
}

impl LocalVectorStore {
    pub fn from_snapshot(snapshot: VectorStoreSnapshot) -> Self {
        Self {
            documents: snapshot.documents,
        }
    }

    pub fn snapshot(&self) -> VectorStoreSnapshot {
        VectorStoreSnapshot {
            documents: self.documents.clone(),
        }
    }
}

impl VectorStore for LocalVectorStore {
    fn upsert(&mut self, docs: Vec<VectorDocument>) -> anyhow::Result<()> {
        for doc in docs {
            if let Some(existing) = self.documents.iter_mut().find(|d| d.id == doc.id) {
                *existing = doc;
            } else {
                self.documents.push(doc);
            }
        }
        Ok(())
    }

    fn query(&self, query: &str, top_k: usize) -> anyhow::Result<Vec<VectorMatch>> {
        let query_embedding = embed(query);
        let mut matches: Vec<VectorMatch> = self
            .documents
            .iter()
            .map(|doc| VectorMatch {
                id: doc.id.clone(),
                score: cosine_similarity(&query_embedding, &doc.embedding),
                metadata: doc.metadata.clone(),
            })
            .collect();

        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(top_k);
        Ok(matches)
    }
}

#[derive(Debug)]
pub struct ChromaStore {
    pub base_url: String,
    pub collection: String,
    client: reqwest::blocking::Client,
}

impl ChromaStore {
    pub fn new(base_url: String, collection: String) -> Self {
        Self {
            base_url,
            collection,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn collection_url(&self) -> String {
        format!("{}/api/v1/collections/{}", self.base_url, self.collection)
    }
}

impl VectorStore for ChromaStore {
    fn upsert(&mut self, docs: Vec<VectorDocument>) -> anyhow::Result<()> {
        let url = format!("{}/upsert", self.collection_url());
        let payload = serde_json::json!({
            "ids": docs.iter().map(|doc| doc.id.clone()).collect::<Vec<_>>(),
            "documents": docs.iter().map(|doc| doc.content.clone()).collect::<Vec<_>>(),
            "embeddings": docs.iter().map(|doc| doc.embedding.clone()).collect::<Vec<_>>(),
            "metadatas": docs.iter().map(|doc| doc.metadata.clone()).collect::<Vec<_>>(),
        });

        self.client.post(url).json(&payload).send()?.error_for_status()?;
        Ok(())
    }

    fn query(&self, query: &str, top_k: usize) -> anyhow::Result<Vec<VectorMatch>> {
        let url = format!("{}/query", self.collection_url());
        let payload = serde_json::json!({
            "query_embeddings": vec![embed(query)],
            "n_results": top_k,
        });

        let response = self
            .client
            .post(url)
            .json(&payload)
            .send()?
            .error_for_status()?;
        let body: serde_json::Value = response.json()?;
        let ids = body
            .get("ids")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let distances = body
            .get("distances")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let metadatas = body
            .get("metadatas")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut matches = Vec::new();
        for (idx, id_value) in ids.iter().enumerate() {
            let id = id_value.as_str().unwrap_or_default().to_string();
            let distance = distances
                .get(idx)
                .and_then(|v| v.as_f64())
                .unwrap_or_default() as f32;
            let metadata_value = metadatas.get(idx).cloned().unwrap_or_default();
            let metadata = metadata_value
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string())))
                        .collect::<BTreeMap<String, String>>()
                })
                .unwrap_or_default();

            matches.push(VectorMatch {
                id,
                score: 1.0 - distance,
                metadata,
            });
        }

        Ok(matches)
    }
}

pub fn embed(text: &str) -> Vec<f32> {
    let hash = blake3::hash(text.as_bytes());
    hash.as_bytes()
        .chunks(4)
        .take(8)
        .map(|chunk| {
            let mut bytes = [0u8; 4];
            for (idx, value) in chunk.iter().enumerate() {
                bytes[idx] = *value;
            }
            let value = u32::from_le_bytes(bytes);
            (value as f32) / (u32::MAX as f32)
        })
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0;
    let mut mag_a = 0.0;
    let mut mag_b = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        mag_a += x * x;
        mag_b += y * y;
    }
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a.sqrt() * mag_b.sqrt())
}
