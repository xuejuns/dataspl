use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// DeepSeek API 配置
#[derive(Clone)]
pub struct DeepSeekConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

impl DeepSeekConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.deepseek.com".to_string(),
            model: "deepseek-chat".to_string(),
        }
    }
}

/// LLM 客户端
#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    config: Arc<DeepSeekConfig>,
}

impl LlmClient {
    // 创建 LlmClient 实例
    pub fn new(config: DeepSeekConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config: Arc::new(config),
        }
    }

    /// 流式调用 DeepSeek API
    /// tx表示发送者，用于将流式响应发送给调用者
    pub async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tx: mpsc::Sender<Result<SseEvent, LlmError>>,
    ) -> Result<(), LlmError> {
        let start_time = Instant::now();
        tracing::info!("[LLM] 开始调用 DeepSeek API, 模型: {}", self.config.model);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: true,
        };

        //Client对象send一个post请求,然后等待返回一个response对象
        //这个response请求就是一个流式相应，其返回的SSEEvent的事件流
        //response对象的bytes_stream方法返回一个Stream<Item=Result<Bytes,Error>>，其中Bytes是一个字节数组，表示从服务器接收到的一块数据
        //Stream<Item=Result<Bytes,Error>>,其中Bytes是一个字节数组，表示从服务器接收到的一块数据
        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("[LLM] HTTP 请求失败: {}", e);
                LlmError::NetworkError(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("[LLM] API 返回错误: {} - {}", status, error_text);
            return Err(LlmError::ApiError(status.as_u16(), error_text));
        }

        let elapsed = start_time.elapsed();
        tracing::info!("[LLM] 开始处理数据流0");
        tracing::info!("[LLM] API 响应时间: {:?}", elapsed);

        // 处理流式响应，其中buffer用于存储未完整的行数据，stream用于逐块读取响应体
        let mut buffer: String = String::new();

        //
        let mut stream = response.bytes_stream();

        //reactor模式，逐块处理流式响应，解析出完整的行数据后发送给调用者
        //获取到stream流后，通过next获取到Some(chunck)，如果是Some(chunk)，则继续处理这个chunk，如果error，就结束流
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // 将字节转换为字符串，并追加到缓冲区
                    let bytes_vec: Vec<u8> = bytes.to_vec();
                    if let Ok(text) = String::from_utf8(bytes_vec) {
                        buffer.push_str(&text);
                        
                        // 按行解析，保留未完成的部分
                        loop {
                            // 查找换行符，获取完整的行数据,如果没有换行符，则说明当前行数据未完整，继续等待下一块数据
                            let newline_pos = match buffer.find('\n') {
                                Some(pos) => pos,
                                None => break,
                            };
                            
                            //如果能提取一行数据，就将这一行数据从缓冲区中移除，并解析出data字段的内容，如果data字段的内容是[done]，则说明流式响应结束，发送完成信号给调用者，并返回
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();
                                
                            if line.starts_with("data: ") {
                                let data = line.trim_start_matches("data: ");
                                if data.is_empty() {
                                    continue;
                                }
                                // 如果data字段的内容是[done]，则说明流式响应结束，发送完成信号给调用者，并返回
                                if data == "[DONE]" {
                                    let _ = tx.send(Ok(SseEvent::Done)).await;// 发送完成信号
                                    return Ok(());
                                }

                                // 解析data字段的内容，提取出delta中的content字段，并发送给调用者
                                if let Ok(delta) = serde_json::from_str::<StreamChoice>(&data) {
                                    if let Some(content) = delta.choices.first().and_then(|c| c.delta.content.as_ref()) {
                                        if !content.is_empty() {
                                            let _ = tx.send(Ok(SseEvent::Content(content.clone()))).await;
                                        }
                                    }
                                } else {
                                    tracing::info!("[LLM] 解析失败: {}", data);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    tracing::error!("[LLM] 读取流失败: {}", error_msg);
                    return Err(LlmError::StreamError(error_msg));
                }
            }
        }


        // 处理缓冲区中剩余的数据
        let line = buffer.trim().to_string();
        if !line.is_empty() && line.starts_with("data: ") {
            let data = line.trim_start_matches("data: ");
            if data != "[DONE]" {
                if let Ok(delta) = serde_json::from_str::<StreamChoice>(&data) {
                    if let Some(content) = delta.choices.first().and_then(|c| c.delta.content.as_ref()) {
                        if !content.is_empty() {
                            let _ = tx.send(Ok(SseEvent::Content(content.clone()))).await;
                        }
                    }
                }
            }
        }
        tracing::info!("[LLM] 处理剩余数据: {}", line);

        // 发送完成信号
        let _ = tx.send(Ok(SseEvent::Done)).await;
        tracing::info!("[LLM] 连接关闭");
        Ok(())
    }
}

// ============ 数据类型 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    choices: Vec<Delta>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    delta: DeltaContent,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeltaContent {
    content: Option<String>,
}

/// SSE 事件类型
#[derive(Clone, Debug)]
pub enum SseEvent {
    Content(String),
    Done,
}

/// LLM 错误
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    #[error("API 错误: {0} - {1}")]
    ApiError(u16, String),
    
    #[error("流错误: {0}")]
    StreamError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
}

impl serde::Serialize for LlmError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
