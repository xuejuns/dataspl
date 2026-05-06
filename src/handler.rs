use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{sse::Event, Sse, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::llm::{Message, SseEvent};
use crate::skill::{SkillExecuteRequest, SkillMetadata};
use crate::state::AppState;

// AISPL 语法手册（用于生成AISPL）
const AISPL_MANUAL: &str = r#"### **AiSPL语法手册**
**AiSPL** (AI Search Processing Language) 是飞梭数据中台专为日志数据分析设计的处理语言，支持多级管道处理流程。原始数据首先通过**索引过滤条件**进行初步筛选，再通过**多级SPL指令**进行结构化提取、字段操作和数据加工，最终输出处理结果。

#### **一、数据源指定**
```spl
'索引名模式'
```
- 原始日志：`'ailpha-securitylog-*'`
- 原始告警：`'ailpha-securityalarm-*'`
- 原始告警：`'ailpha-securityevent-*'`

#### **二、基础分析**
**数据过滤**
```spl
 filter 字段名 运算符 值
```
- 等值过滤：`filter severity == 5`
- 范围过滤：`filter destPort > 1024`
- 时间过滤：`filter collectorReceiptTime >= "2025-07-18 01:00:00"`
- 字段存在性：`filter srcAddress exist`

**运算符说明**
| 语法 | 描述 | 示例 |
| :--- | :--- | :--- |
| == | 等于 | srcAddress == "192.168.1.101" |
| != | 不等于 | destPort != 1024 |
| exist | 存在 | destAddress exist |
| notexist | 不存在 | destAddress notexist |
| in | 属于 | catOutcome in ["OK","FAIL"] |
| notin | 不属于 | catOutcome notin ["OK","FAIL"] |
| startwith | 开始于 | name startwith "远程" |
| endwith | 结束于 | requestUrl endwith ".php" |
| > < >= <= | 比较 | destPort > 1024 |
| contains | 包含 | message contains "SQL注入" |
| NOT contains | 不包含 | NOT name contains "弱口令" |
| =~ | 正则匹配 | message =~ /\d{2}/ |
| !~ | 正则不匹配 | message !~ /\d{2}/ |

#### **三、聚合分析**
```spl
 summarize 聚合函数(字段) as 别名 by 分组字段
```
- COUNT(), DCOUNT(), SUM(), AVG(), MAX(), MIN(), STDDEV(), COLLECT()
- 时间分组：`by bin collectorReceiptTime interval 1m`

#### **四、字段操作**
```spl
| let 新字段名 = 表达式
```

#### **五、活动列表碰撞**
```spl
hit 活动列表名 字段1, 字段2 ON 列表键 where 列表条件 RETURN 列表字段 as 别名
```

#### **六、维表关联**
```spl
dbload 维表别名 "JDBC连接参数"
|lookup 维表别名 主表字段 join 维表关联键 return 维表字段1 as 别名1
```

#### **七、时间语法**
```spl
| let last3d = now()-3d
| filter collectorReceiptTime>last3d
```

#### **八、字段投影**
```spl
| project 字段1, 字段2
| project -字段1, -字段2
```

#### **九、窗口分析**
```spl
windowsummarize 聚合函数(字段) as 别名 by 分组字段 window 窗口大小 on 时间字段
```

#### **十、排序与限制**
```spl
| sort +字段名
| sort -字段名
| limit N
```

#### **十一、数据输出**
```spl
| output 结果表名
```

#### **十二、内置函数**
- 字符串：concat, substring, trim, lower, upper, strlen, md5
- 时间：currentDate(), currentTime(), now(), dateFormat(), year(), month(), day(), hour()
- JSON：jsonValue(), jsonExists(), isJson()
- 类型转换：tointeger(), toDouble(), toString()
- 数学：sqrt()
- 条件：if(condition, true, false), ifnull(field, default)
- 数组：array_count, array_distinct, array_first, array_last, array_get, array_join

#### **十三、可用字段列表**
以下字段可直接用于filter、summarize、project等操作中：

**地址相关字段：**
- srcAddress, destAddress - 源/目标IP地址
- srcHostName, destHostName - 源/目标主机名
- srcGeoCountry, destGeoCountry - 源/目标国家
- srcGeoCity, destGeoCity - 源/目标城市
- srcGeoRegion, destGeoRegion - 源/目标地区

**端口相关字段：**
- srcPort, destPort - 源/目标端口
- srcTransPort, destTransPort - 传输层源/目标端口

**用户相关字段：**
- srcUserName, destUserName - 源/目标用户名
- srcUserId, destUserId - 源/目标用户ID
- srcUserAccount, destUserAccount - 源/目标账户

**网络相关字段：**
- bytesIn, bytesOut - 入/出字节数
- srcMacAddress, destMacAddress - 源/目标MAC地址
- appProtocol, protocolType - 应用/协议类型
- transProtocol, flowProtocol - 传输/流量协议

**时间相关字段：**
- collectorReceiptTime - 采集接收时间（时间过滤推荐使用）
- startTime, endTime - 开始/结束时间
- eventTime, createTime - 事件/创建时间

**告警相关字段：**
- severity - 严重程度
- confidence - 置信度
- alarmName, alarmType, alarmStatus - 告警名称/类型/状态
- alertType, alarmSource - 告警类型/来源
- threatSeverity, threatName - 威胁严重程度/名称

**行为相关字段：**
- catBehavior - 目录行为
- catOutcome - 行为结果（如OK、FAIL）
- catSignificance, catTechnique - 分类显著性/技术
- attackMethod, attackSource, attackTarget - 攻击方法/来源/目标

**文件相关字段：**
- fileName, filePath, fileHash - 文件名/路径/哈希
- fileSize, fileType, fileMd5 - 文件大小/类型/MD5
- targetFilename, oldFileName - 目标/旧文件名

**进程相关字段：**
- processName, processId - 进程名/ID
- parentProcessName, parentProcessId - 父进程名/ID
- commandLine, image - 命令行/镜像路径

**日志类型字段：**
- logType, logVersion - 日志类型/版本
- eventId, eventType - 事件ID/类型
- message, description - 消息/描述
- rawEvent - 原始事件

**设备相关字段：**
- deviceId, deviceName, deviceHostname - 设备ID/名称/主机名
- deviceProductType, deviceVendor - 设备产品类型/厂商
- deviceAddress, deviceAssetType - 设备地址/资产类型

**其他常用字段：**
- isAPT - 是否为APT攻击
- isJson - 是否为JSON格式
- tags - 标签数组
- requestUrl, requestMethod - 请求URL/方法
- responseCode, responseTime - 响应码/响应时间
- status, errorCode, errorMessage - 状态/错误码/错误消息
- requestBody, responseHeader - 请求体/响应头

#### **十四、注意事项**
1. 正则中如有|字符需转义\|
2. 正则不使用忽略大小写/i和全局搜索/g
3. let语句中变量名不要与上述原有字段名相同
"#;

// 聊天请求
#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

// AISPL生成请求
#[derive(Deserialize)]
pub struct GenerateAisplRequest {
    pub requirement: String,
}

// 聊天响应
#[derive(Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ChatResponse {
    #[serde(rename = "thinking")]
    Thinking(String),
    #[serde(rename = "response")]
    Response(String),
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "error")]
    Error(String),
}

// 登录请求
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// 登录响应
#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub message: String,
}

// token验证查询
#[derive(Deserialize)]
pub struct TokenQuery {
    pub token: String,
}

const VALID_USERNAME: &str = "admin";
const VALID_PASSWORD: &str = "123456";

// 登录处理
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Json<LoginResponse> {
    if request.username == VALID_USERNAME && request.password == VALID_PASSWORD {
        let token = format!(
            "token_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        state.add_token(token.clone()).await;
        tracing::info!("用户登录成功: {}, token: {}", request.username, token);
        Json(LoginResponse {
            success: true,
            token: Some(token),
            message: "登录成功".to_string(),
        })
    } else {
        tracing::warn!("用户登录失败: {}", request.username);
        Json(LoginResponse {
            success: false,
            token: None,
            message: "用户名或密码错误".to_string(),
        })
    }
}

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Query(token_query): Query<TokenQuery>,
    Json(request): Json<ChatRequest>,
) -> Response {
    // 验证并刷新 token 过期时间
    if !state.validate_and_refresh_token(&token_query.token).await {
        return (StatusCode::UNAUTHORIZED, "未授权或登录已过期").into_response();
    }

    tracing::info!("收到聊天请求: {}", request.message);

    // 创建一个 mpsc 通道来发送 SSE 事件
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    // 获取 LLM 客户端的克隆
    let llm_client = state.llm_client.clone();

    // 启动异步任务调用 LLM
    tokio::spawn(async move {
        // 1. 发送 thinking 状态
        let thinking = ChatResponse::Thinking("正在思考...".to_string());
        let _ = tx
            .send(Ok(Event::default().json_data(thinking).unwrap()))
            .await;

        // 2. 构建消息列表
        let messages = vec![Message {
            role: "user".to_string(),
            content: request.message.clone(),
        }];

        // 创建内部通道用于 LLM 响应，其中llm_tx用于发送LLM响应，llm_rx用于接收LLM响应
        let (llm_tx, mut llm_rx) = mpsc::channel::<Result<SseEvent, crate::llm::LlmError>>(32);

        // 启动 LLM 调用任务（在单独任务中运行，避免阻塞接收循环）
        // 发送请求给LLM客户端，LLM客户端会将流式响应通过llm_tx发送回来
        // messages是一个包含用户消息的向量，llm_tx是一个发送者，用于将LLM的流式响应发送给调用者
        let llm_task = tokio::spawn(async move {
            llm_client.chat_stream(messages, llm_tx).await
        });

        // 转发 LLM 响应到 SSE 通道
        while let Some(result) = llm_rx.recv().await {
            match result {
                Ok(SseEvent::Content(content)) => {
                    let response = ChatResponse::Response(content);
                    if tx.send(Ok(Event::default().json_data(response).unwrap())).await.is_err() {
                        break;
                    }
                }
                Ok(SseEvent::Done) => {
                    tracing::info!("[LLM] 发送到完成");
                    let done = ChatResponse::Done;
                    let _ = tx.send(Ok(Event::default().json_data(done).unwrap())).await;
                    tracing::info!("LLM 响应完成");
                    break;
                }
                Err(e) => {
                    tracing::error!("LLM 调用错误: {}", e);
                    let error = ChatResponse::Error(e.to_string());
                    let _ = tx.send(Ok(Event::default().json_data(error).unwrap())).await;
                    break;
                }
            }
        }
        
        // 等待 LLM 任务完成（处理可能的错误）
        if let Err(e) = llm_task.await {
            tracing::error!("LLM 任务 panic: {}", e);
        }
    });

    // 返回 SSE 流
    Sse::new(ReceiverStream::new(rx))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("keep-alive"),
        )
        .into_response()
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
}

pub async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Query(token_query): Query<TokenQuery>,
) -> Json<VerifyResponse> {
    let valid = state.validate_and_refresh_token(&token_query.token).await;
    Json(VerifyResponse { valid })
}

// AISPL生成处理
pub async fn generate_aispl_handler(
    State(state): State<Arc<AppState>>,
    Query(token_query): Query<TokenQuery>,
    Json(request): Json<GenerateAisplRequest>,
) -> Response {
    // 验证并刷新 token 过期时间
    if !state.validate_and_refresh_token(&token_query.token).await {
        return (StatusCode::UNAUTHORIZED, "未授权或登录已过期").into_response();
    }

    tracing::info!("收到AISPL生成请求: {}", request.requirement);

    // 创建 mpsc 通道发送 SSE 事件
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);
    let llm_client = state.llm_client.clone();
    let requirement = request.requirement.clone();

    tokio::spawn(async move {
        // 发送 thinking 状态
        let thinking = ChatResponse::Thinking("正在生成AISPL语句...".to_string());
        let _ = tx
            .send(Ok(Event::default().json_data(thinking).unwrap()))
            .await;

        // 构建系统提示，包含AISPL语法手册
        let system_prompt = format!(
            r#"你是一个专业的AiSPL语法专家。请根据用户的自然语言需求，生成对应的AiSPL查询语句。

{}的语法手册：

{}

## 任务：
1. 理解用户的日志分析需求
2. 根据语法手册生成正确的AISPL语句
3. 只返回AISPL语句，不要添加其他解释或说明
4. 如果需求不完整或模糊，给出一个合理的推断并生成语句

## 要求：
- 生成的AISPL语句必须符合上述语法规范
- 使用管道符|连接多个操作
- 确保字段名和语法正确

用户需求：{}"#,
            "AiSPL", AISPL_MANUAL, requirement
        );

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "你是一个专业的AiSPL语法专家，只返回AISPL语句，不返回其他说明。".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: system_prompt,
            },
        ];

        let (llm_tx, mut llm_rx) = mpsc::channel::<Result<SseEvent, crate::llm::LlmError>>(32);

        let llm_task = tokio::spawn(async move {
            llm_client.chat_stream(messages, llm_tx).await
        });

        while let Some(result) = llm_rx.recv().await {
            match result {
                Ok(SseEvent::Content(content)) => {
                    let response = ChatResponse::Response(content);
                    if tx.send(Ok(Event::default().json_data(response).unwrap())).await.is_err() {
                        break;
                    }
                }
                Ok(SseEvent::Done) => {
                    let done = ChatResponse::Done;
                    let _ = tx.send(Ok(Event::default().json_data(done).unwrap())).await;
                    break;
                }
                Err(e) => {
                    tracing::error!("LLM调用错误: {}", e);
                    let error = ChatResponse::Error(e.to_string());
                    let _ = tx.send(Ok(Event::default().json_data(error).unwrap())).await;
                    break;
                }
            }
        }

        if let Err(e) = llm_task.await {
            tracing::error!("LLM任务panic: {}", e);
        }
    });

    Sse::new(ReceiverStream::new(rx))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("keep-alive"),
        )
        .into_response()
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// ===== Skill Handlers =====

/// List all available skills
pub async fn list_skills_handler(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<SkillMetadata>> {
    let skills = state.skill_container.list_skill_metadata().await;
    Json(skills)
}

/// Execute a skill with streaming response
pub async fn execute_skill_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SkillExecuteRequest>,
) -> Response {
    tracing::info!("[Skill] Execute request for skill: {}", request.skill_id);

    // Create mpsc channel for SSE
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);
    let executor = state.skill_executor.clone();
    let skill_id = request.skill_id.clone();

    tokio::spawn(async move {
        // Send thinking status
        let thinking = ChatResponse::Thinking(format!("Executing skill '{}'...", skill_id));
        let _ = tx
            .send(Ok(Event::default().json_data(thinking).unwrap()))
            .await;

        // Create adapter channel for LLM events
        let (llm_tx, mut llm_rx) = mpsc::channel::<Result<SseEvent, crate::llm::LlmError>>(32);

        // Execute skill with streaming in background
        let exec_request = request.clone();
        let llm_task = tokio::spawn(async move {
            executor.execute_streaming(exec_request, llm_tx).await
        });

        // Forward LLM events to SSE
        while let Some(result) = llm_rx.recv().await {
            match result {
                Ok(SseEvent::Content(content)) => {
                    let response = ChatResponse::Response(content);
                    if tx.send(Ok(Event::default().json_data(response).unwrap())).await.is_err() {
                        break;
                    }
                }
                Ok(SseEvent::Done) => {
                    let done = ChatResponse::Done;
                    let _ = tx.send(Ok(Event::default().json_data(done).unwrap())).await;
                    break;
                }
                Err(e) => {
                    tracing::error!("[Skill] LLM error: {}", e);
                    let error = ChatResponse::Error(e.to_string());
                    let _ = tx.send(Ok(Event::default().json_data(error).unwrap())).await;
                    break;
                }
            }
        }

        if let Err(e) = llm_task.await {
            tracing::error!("[Skill] Execution task error: {}", e);
        }
    });

    Sse::new(ReceiverStream::new(rx))
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("keep-alive"),
        )
        .into_response()
}
