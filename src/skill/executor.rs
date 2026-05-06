use super::{Skill, SkillContainer, SkillError, SkillExecuteRequest, SkillExecuteResult};
use crate::llm::{LlmClient, Message, SseEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Skill executor - executes skills using LLM
#[derive(Clone)]
pub struct SkillExecutor {
    container: SkillContainer,
    llm_client: LlmClient,
}

impl SkillExecutor {
    pub fn new(container: SkillContainer, llm_client: LlmClient) -> Self {
        Self {
            container,
            llm_client,
        }
    }

    /// Execute a skill by ID with input
    pub async fn execute(
        &self,
        request: SkillExecuteRequest,
    ) -> Result<SkillExecuteResult, SkillError> {
        let skill = self
            .container
            .get_skill(&request.skill_id)
            .await
            .ok_or_else(|| SkillError::NotFound(request.skill_id.clone()))?;

        // Build prompt with input and context
        let prompt = self.build_prompt(&skill, &request.input, &request.context)?;

        // Execute via LLM
        let output = self.execute_llm(&prompt).await?;

        Ok(SkillExecuteResult {
            skill_id: request.skill_id,
            success: true,
            output: Some(output),
            error: None,
        })
    }

    /// Execute a skill with streaming response
    pub async fn execute_streaming(
        &self,
        request: SkillExecuteRequest,
        tx: mpsc::Sender<Result<SseEvent, crate::llm::LlmError>>,
    ) -> Result<(), SkillError> {
        let skill = self
            .container
            .get_skill(&request.skill_id)
            .await
            .ok_or_else(|| SkillError::NotFound(request.skill_id.clone()))?;

        let prompt = self.build_prompt(&skill, &request.input, &request.context)?;

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt,
        }];

        self.llm_client.chat_stream(messages, tx).await
            .map_err(|e| SkillError::ExecutionError(e.to_string()))?;

        Ok(())
    }

    /// Build full prompt by filling template variables
    fn build_prompt(
        &self,
        skill: &Skill,
        input: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, SkillError> {
        let mut prompt = skill.prompt.clone();

        // Replace {input} placeholder
        prompt = prompt.replace("{input}", input);

        // Replace context variables
        for (key, value) in context {
            let placeholder = format!("{{{}}}", key);
            prompt = prompt.replace(&placeholder, value);
        }

        // Check for unfilled placeholders
        let unfilled_re = regex::Regex::new(r"\{(\w+)\}").unwrap();
        let unfilled: Vec<String> = unfilled_re
            .captures_iter(&prompt)
            .map(|c| c.get(1).unwrap().as_str().to_string())
            .collect();

        if !unfilled.is_empty() {
            return Err(SkillError::ExecutionError(format!(
                "Unfilled placeholders: {}",
                unfilled.join(", ")
            )));
        }

        Ok(prompt)
    }

    /// Execute prompt via LLM and return response
    async fn execute_llm(&self, prompt: &str) -> Result<String, SkillError> {
        //创建通道用于接收流式响应，tx用于发送数据，rx用于接收数据
        //将tx给chat_stream方法，chat_stream方法会在接收到流式响应时通过tx发送数据给rx
        //rx在这里等待接收数据，当接收到数据时，将其追加到output字符串中，直到接收到Done事件表示流式响应结束
        let (tx, mut rx) = mpsc::channel(32);

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let llm_client = self.llm_client.clone();
        let handle = tokio::spawn(async move {
            llm_client.chat_stream(messages, tx).await
        });

        let mut output = String::new();
        while let Some(result) = rx.recv().await {
            match result {
                Ok(SseEvent::Content(content)) => {
                    output.push_str(&content);
                }
                Ok(SseEvent::Done) => {
                    break;
                }
                Err(e) => {
                    return Err(SkillError::ExecutionError(e.to_string()));
                }
            }
        }

        if let Err(e) = handle.await {
            return Err(SkillError::ExecutionError(format!("Task join error: {}", e)));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_prompt() {
        // This test would need proper setup
    }
}