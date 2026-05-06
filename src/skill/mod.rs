mod loader;
mod executor;

pub use executor::SkillExecutor;
pub use loader::SkillLoader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Skill definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique identifier for the skill
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what the skill does
    pub description: String,
    /// Version of the skill
    #[serde(default = "default_version")]
    pub version: String,
    /// Prompt sent to LLM when skill is invoked
    pub prompt: String,
    /// Examples of usage (optional, parsed from markdown)
    #[serde(default)]
    pub examples: Vec<SkillExample>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Skill usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    pub description: String,
    pub input: String,
    pub expected_output: Option<String>,
}

/// Skill execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecuteRequest {
    pub skill_id: String,
    pub input: String,
    #[serde(default)]
    pub context: HashMap<String, String>,
}

/// Skill execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecuteResult {
    pub skill_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Skill container - manages all loaded skills
#[derive(Clone)]
pub struct SkillContainer {
    skills: Arc<RwLock<HashMap<String, Skill>>>,
}

impl Default for SkillContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillContainer {
    pub fn new() -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a single skill
    pub async fn load_skill(&self, skill: Skill) -> Result<(), SkillError> {
        let skill_id = skill.id.clone();
        let mut skills = self.skills.write().await;
        skills.insert(skill_id.clone(), skill);
        tracing::info!("[SkillContainer] Loaded skill: {}", skill_id);
        Ok(())
    }

    /// Remove a skill by ID
    pub async fn remove_skill(&self, skill_id: &str) -> Option<Skill> {
        let mut skills = self.skills.write().await;
        skills.remove(skill_id)
    }

    /// Get a skill by ID
    pub async fn get_skill(&self, skill_id: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.get(skill_id).cloned()
    }

    /// List all skill IDs
    pub async fn list_skills(&self) -> Vec<String> {
        let skills = self.skills.read().await;
        skills.keys().cloned().collect()
    }

    /// List all skill metadata
    pub async fn list_skill_metadata(&self) -> Vec<SkillMetadata> {
        let skills = self.skills.read().await;
        skills
            .values()
            .map(|s| SkillMetadata {
                id: s.id.clone(),
                name: s.name.clone(),
                description: s.description.clone(),
                version: s.version.clone(),
            })
            .collect()
    }

    /// Check if a skill exists
    pub async fn has_skill(&self, skill_id: &str) -> bool {
        let skills = self.skills.read().await;
        skills.contains_key(skill_id)
    }

    /// Get total skill count
    pub async fn skill_count(&self) -> usize {
        let skills = self.skills.read().await;
        skills.len()
    }

    /// Clear all skills
    pub async fn clear(&self) {
        let mut skills = self.skills.write().await;
        skills.clear();
        tracing::info!("[SkillContainer] Cleared all skills");
    }
}

/// Skill metadata (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
}

/// Skill errors
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Skill not found: {0}")]
    NotFound(String),

    #[error("Load error: {0}")]
    LoadError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}

impl serde::Serialize for SkillError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_skill_container() {
        let container = SkillContainer::new();

        let skill = Skill {
            id: "test-skill".to_string(),
            name: "Test Skill".to_string(),
            description: "A test skill".to_string(),
            version: "1.0.0".to_string(),
            prompt: "Hello, {name}!".to_string(),
            examples: vec![],
        };

        container.load_skill(skill).await.unwrap();
        assert!(container.has_skill("test-skill").await);

        let retrieved = container.get_skill("test-skill").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Skill");
    }
}