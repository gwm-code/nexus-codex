use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrationKind {
    Github,
    Sentry,
    Slack,
    SQLite,
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub name: String,
    pub kind: IntegrationKind,
    pub enabled: bool,
    pub details: BTreeMap<String, String>,
}

impl IntegrationConfig {
    pub fn new(name: &str, kind: IntegrationKind, enabled: bool) -> Self {
        Self {
            name: name.to_string(),
            kind,
            enabled,
            details: BTreeMap::new(),
        }
    }
}

pub fn default_integrations() -> Vec<IntegrationConfig> {
    vec![
        IntegrationConfig::new("GitHub", IntegrationKind::Github, false),
        IntegrationConfig::new("Sentry", IntegrationKind::Sentry, false),
        IntegrationConfig::new("Slack", IntegrationKind::Slack, false),
        IntegrationConfig::new("SQLite", IntegrationKind::SQLite, true),
        IntegrationConfig::new("Postgres", IntegrationKind::Postgres, false),
    ]
}

pub fn set_enabled(integrations: &mut [IntegrationConfig], name: &str, enabled: bool) -> bool {
    if let Some(target) = integrations
        .iter_mut()
        .find(|integration| integration.name.eq_ignore_ascii_case(name))
    {
        target.enabled = enabled;
        return true;
    }
    false
}

pub fn set_detail(
    integrations: &mut [IntegrationConfig],
    name: &str,
    key: &str,
    value: &str,
) -> bool {
    if let Some(target) = integrations
        .iter_mut()
        .find(|integration| integration.name.eq_ignore_ascii_case(name))
    {
        target.details.insert(key.to_string(), value.to_string());
        return true;
    }
    false
}
