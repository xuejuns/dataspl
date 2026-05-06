use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use crate::llm::LlmClient;
use crate::skill::{SkillContainer, SkillExecutor, SkillLoader};

const TOKEN_TTL: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct AppState {
    tokens: Arc<RwLock<HashMap<String, SystemTime>>>,
    pub llm_client: LlmClient,
    pub skill_container: SkillContainer,
    pub skill_executor: SkillExecutor,
}

use std::sync::Arc;

impl AppState {
    pub fn new(llm_client: LlmClient) -> Self {
        let skill_container = SkillContainer::new();
        let skill_executor = SkillExecutor::new(skill_container.clone(), llm_client.clone());
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            llm_client,
            skill_container,
            skill_executor,
        }
    }

    pub async fn add_token(&self, token: String) {
        let expiry = SystemTime::now() + TOKEN_TTL;
        self.tokens.write().await.insert(token, expiry);
    }

    pub async fn validate_and_refresh_token(&self, token: &str) -> bool {
        let now = SystemTime::now();
        let mut tokens = self.tokens.write().await;
        tokens.retain(|_, expiry| *expiry > now);
        if let Some(expiry) = tokens.get_mut(token) {
            *expiry = now + TOKEN_TTL;
            true
        } else {
            false
        }
    }

    /// Load skills from the skills/ directory
    pub async fn load_skills_from_dir(&self, dir_path: &str) -> Result<usize, crate::skill::SkillError> {
        let skills = SkillLoader::load_from_directory(dir_path).await?;
        let count = skills.len();
        for skill in skills {
            self.skill_container.load_skill(skill).await?;
        }
        tracing::info!("[AppState] Loaded {} skills from {}", count, dir_path);
        Ok(count)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(LlmClient::new(crate::llm::DeepSeekConfig::new("default".to_string())))
    }
}