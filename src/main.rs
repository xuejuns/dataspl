mod handler;
mod llm;
mod router;
mod skill;
mod state;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axumdemo=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Axum SSE Chat Demo with DeepSeek LLM");

    // 从环境变量读取 DeepSeek API Key
    let deepseek_api_key = std::env::var("DEEPSEEK_API_KEY")
        .unwrap_or_else(|_| "your-api-key-here".to_string());

    // 创建 LLM 客户端
    let llm_client = llm::LlmClient::new(llm::DeepSeekConfig::new(deepseek_api_key));

    // 创建应用状态
    let app_state = Arc::new(state::AppState::new(llm_client));

    // 从 skills/ 目录加载所有 .md 文件作为 skills
    if let Err(e) = app_state.load_skills_from_dir("skills").await {
        tracing::warn!("Failed to load skills from skills/ directory: {}", e);
    }

    let app = router::create_router(app_state);

    //创建tcp监听器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    //启动服务器
    println!("Server running on http://127.0.0.1:3000");
    println!("Set DEEPSEEK_API_KEY environment variable for LLM functionality");
    axum::serve(listener, app).await.unwrap();
}