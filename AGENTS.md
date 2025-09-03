# 仓库综述（Agents 参考）

本文件为项目智能体（Agents）使用的知识卡，用于快速理解本仓库的结构、能力与扩展方式。

---

## 一、项目概览
- 名称：Anthropic Rust SDK（第三方、类型安全、与 TypeScript SDK 功能对齐）
- 目标：提供对 Anthropic API 的完整访问能力，涵盖消息、流式传输、工具调用、视觉、文件、批处理、模型等；具备健壮的错误处理、重试与日志体系，以及易用的 Builder 风格 API
- 运行环境：Tokio 异步运行时；Reqwest HTTP 客户端；Serde 序列化

---

## 二、核心模块结构（src/）

- client.rs 与 config.rs
  - Anthropic 客户端入口：new/from_env/with_config，聚合各资源入口（messages、files、batches、models）
  - ClientConfig：超时、最大重试、日志级别、Base URL、自定义鉴权方式等

- http/
  - client.rs：底层 HTTP 封装（build_url、通用 get/post/put/delete、提取 request_id、Beta Header/Thinking 等可选头）
  - auth.rs：多种鉴权方式（Bearer/Token），统一注入请求头
  - retry.rs：RetryExecutor/RetryPolicy（指数退避、抖动、最大时长、重试条件）
  - streaming.rs：HTTP 流式请求构建与消费（StreamConfig、post_stream、事件缓冲/超时/错误重试）

- streaming/
  - mod.rs：MessageStream 抽象（on_text/on_stream_event/on_message/... 链式回调、final_message/done）、状态管理（ended/errored/aborted）
  - events.rs：流式事件定义与派发（本仓已对类型复杂度做过优化，使用类型别名/枚举）

- resources/
  - messages.rs：消息创建与流式接口（MessageCreateBuilderWithClient 协作）
  - files.rs：文件上传、下载、列表、删除、批量上传、进度、等待处理完成、按用途筛选等
  - batches.rs：批处理创建/查询/列出/取消/结果下载；等待完成、进度监控、便捷 create_and_wait
  - models.rs：模型列表/推荐/对比等高阶能力（配合 types/models_api.rs）

- files/
  - mod.rs：File/FileBuilder/FileConstraints 封装（from_bytes/path/base64/std_file、哈希校验、类型与大小约束、MIME 判断）

- tokens/
  - mod.rs：计费与用量统计（ModelPricing、TokenCounter、CostBreakdown、UsageStats、UsageSummary、估算费用）

- tools/
  - registry.rs：ToolRegistry（注册/并行执行/查询/卸载），Tool 定义与运行入口
  - executor.rs：ToolExecutor/ToolExecutionConfig（重试、并行、最大并发、退避策略）
  - conversation.rs：ToolConversation（多轮对话 + 自动执行工具链）、ConversationConfig/Builder
  - mod.rs：SimpleTool 等便捷封装

- types/
  - messages.rs、streaming.rs：消息/流式相关数据结构（Message、ContentBlock、MessageDelta 等）
  - batches.rs、files_api.rs、models.rs、models_api.rs、tools.rs：各 API 的参数与响应类型，含推荐与对比、成本估算、能力矩阵等
  - shared.rs：通用类型（Usage、ThinkingConfig、CacheControl 等）
  - errors.rs：统一错误类型 AnthropicError

- utils/
  - logging.rs：日志初始化与请求/响应日志钩子

- lib.rs：导出版本与 User-Agent 常量，以及对外可用 API 聚合

---

## 三、示例程序（examples/）按主题索引

- 基础与消息：
  - basic_client.rs、messages_api.rs、end_to_end_demo.rs、test_basic.rs、test_demo.rs
- 流式响应：
  - streaming_example.rs、test_streaming.rs、test_streaming_next.rs、test_custom_streaming.rs
- 工具调用：
  - comprehensive_tool_use.rs、tool_use_comprehensive.rs、test_tool_multiturn.rs、test_tool_multiturn_streaming.rs、simple_streaming_tool_test.rs
- 扩展思维/Thinking：
  - test_thinking.rs、test_extended_thinking.rs、test_cache_thinking.rs、test_prompt_caching.rs
- 文件 API：
  - comprehensive_file_upload.rs、file_upload_comprehensive.rs、phase5_2_files_api.rs
- 批处理 API：
  - phase5_1_batches.rs
- 模型 API：
  - phase5_3_models_api.rs
- 自定义网关与鉴权：
  - custom_gateway_demo.rs、custom_gateway_production.rs、custom_auth_gateway.rs、test_custom_gateway.rs、debug_auth_headers.rs
- 基础设施与生产范式：
  - phase4_4_infrastructure.rs、production_patterns.rs
- Shell 辅助脚本：
  - curl_test.sh、curl_stream.sh、curl_test_thinking.sh

---

## 四、构建与运行

- 构建/测试
  - cargo build
  - cargo test（含单元与集成测试）；也可 cargo test -- --nocapture 观察输出

- 运行示例
  - cargo run --example basic_client
  - cargo run --example messages_api
  - cargo run --example streaming_example

- 环境变量
  - ANTHROPIC_API_KEY（必须）
  - ANTHROPIC_BASE_URL、ANTHROPIC_TIMEOUT、ANTHROPIC_MAX_RETRIES（可选）

---

## 五、关键设计要点

- 类型系统：大量 Builder 模式与枚举，serde 自动序列化，运行时校验
- 错误与重试：AnthropicError 分类清晰；RetryPolicy/RetryExecutor 提供指数退避、抖动、最大重试/时长、重试条件
- 流式与背压：Streaming 客户端封装，事件订阅、缓冲与超时处理
- 工具系统：
  - 定义 Tool（名称、描述、输入 schema）；实现 ToolFunction 或使用 SimpleTool 包装异步函数
  - ToolRegistry 注册/执行/并行；ToolExecutor 支持重试与并发配置；ToolConversation 支持「模型-工具-模型」闭环
- 计费与用量：TokenCounter 可记录与估算费用，按模型定价表汇总
- 文件处理：FileBuilder/Constraints、哈希校验、MIME 类型与大小限制、Base64/路径/字节多源输入
- 日志：请求/响应结构化日志与请求 ID 抽取

---

## 新增：流式工具就绪回调

- 新增事件与回调：在 streaming 模块中增加 EventType::ToolReady 与 EventHandler::ToolReady，并提供链式注册 API：
  - MessageStream.on_tool_ready(|tool_use| { /* 工具就绪时触发 */ })
  - MessageStream.on_tool_ready_execute(registry: SharedToolRegistry, on_result: Arc<Fn(ToolResult)>)：解析到完整的 ToolUse（id/name/input）后即刻 tokio::spawn 异步执行工具，将结果通过回调返回。
- 触发时机：当收到 ContentBlockStop 且最终块为 ToolUse 时，立即构造 ToolUse 并回调；无需等待整条消息结束。
- 兼容性：不影响既有 on_stream_event/on_text/on_final_message 等回调；仍可在流结束后统一执行工具作为回退逻辑。
- 示例：examples/test_tool_multiturn_streaming.rs 已接入 on_tool_ready_execute，实时收集 ToolResult 并在第二轮消息中回传给模型。

## 六、安全与合规

- 仅通过 HTTPS；API Key 不记录到日志
- 输入校验与大小/类型限制；速率与错误感知

---

## 七、贡献与发布

- 贡献流程：Fork ➜ Feature 分支 ➜ 提交改动与测试 ➜ PR
- 文档：docs.rs（支持 cargo doc --open 生成本地文档）
- 发布脚本与配置：publish-sdk.sh、publish-docs.sh、cliff.toml、deny.toml

---

## 八、近期维护与质量（截至当前）

- 已进行的清理（摘要）：
  - 使用 strip_prefix、更新 base64 标准 API、实现 ModelPricing 的 Default 等以消除 Clippy 警告
  - streaming/events 类型复杂度优化（类型别名与枚举），多处示例未用变量下划线处理

- 建议的后续方向：
  - tools、types 模块中的个别 Clippy 建议（new_ret_no_self、布尔模式简化、clamp 等）
  - 示例中可读性与静态检查优化（允许注解或小幅重构）

---

## 九、如何扩展（快速参考）

- 新增工具
  - 定义 Tool（.parameter/.required/.build）
  - 实现 ToolFunction 或使用 SimpleTool 包装 async 函数
  - 通过 ToolRegistry.register 注册，使用 ToolExecutor 执行（支持并行/重试）

- 新增资源 API
  - 在 resources/ 下新增 {resource}.rs，结合 http/client.rs 完成请求封装
  - 在 types/ 下定义请求/响应结构体与枚举
  - 在 Anthropic 客户端中暴露入口方法

---

## 十、目录速览（根目录）

- Cargo.toml、Cargo.lock、README.md、LICENSE
- 源码：src/（见“核心模块结构”）
- 示例：examples/
- 工具脚本与配置：publish-*.sh、deny.toml、cliff.toml、_typos.toml、rust-analyzer.toml

---

## 说明

- 本文件根据仓库当前内容生成，用于 Agents 快速对齐上下文。

---

## 改造计划（2025-09-03 · SDK 聚合流与高阶封装）

背景与问题
- 上游协议变更：流式响应取消 snapshot 字段，工具输入需要基于 partial_json 进行手动拼接。
- 现状：拼接与解析逻辑落在 Engine 层（tool_input_buffers、ContentBlockStop 再 parse），易散乱、容易受协议细节影响。
- 体验差异：TypeScript 官方 SDK 做了更多“高层聚合/便捷封装”，Rust 端当前为“薄适配 + 原始事件”，上层引擎侧代码啰嗦且脆弱。

总体目标
- 将“partial_json 聚合 → ToolUse 输入就绪”的职责上移到 SDK，Engine 仅消费“已就绪的工具输入事件/结果”。
- 提供更高层的会话/流式接口，使调用姿势接近 TS 官方 SDK 的简洁体验。
- 兼容性策略：当前处于高速迭代阶段，本计划不保证向后兼容。会尽量提供迁移路径，但在必要的架构优化下优先保证简洁与性能。

范围与不做事项
- 做：
  - 新增“聚合流”接口与事件类型（AggregatedEvent），统一在 SDK 内完成 partial_json 缓冲与解析。
  - 新增高阶运行接口（run_once/stream_with_callbacks），简化常见调用场景。
  - 提供工具声明/视图辅助函数（from_registry_view 或 declare_tools_from(view)），减少注册样板。
  - 统一错误模型与 tracing 点位，增强可观测性。
- 不做：
  - 不改变上层 Engine 的并发策略与视图/安全模型（仅减少其文本与工具输入组装代码）。
  - 不在此阶段引入新的后端 Provider（可在后续“Provider 抽象”里程碑推进）。

里程碑与交付
- M1：聚合流基础（高优先级）
  - messages().stream_aggregated(...)：
    - 输出 AggregatedEvent 序列（TextDelta、ToolUseDelta(partial)、ToolUseReady{id,name,input: Value}、UsageDelta、MessageStop）。
    - 在 SDK 内用哈希表按 content_block index 缓存 partial_json，ContentBlockStop 时一次性 parse 为 Value。
    - 提供 declare_tools_from(view) 或 from_registry_view(view) 便捷函数，从 ToolRegistryView 直接生成 Tool 列表。
  - 上层适配：为 Engine 侧提供一个薄适配（可选），以消费 ToolUseReady 事件；清理引擎内 tool_input_buffers 代码路径。
  - 测试：协议无 snapshot 下工具输入正确性；多 ToolUse 顺序与内容一致性；只读并发策略不受影响。

- M2：高阶接口与回调（中优先级）
  - 新增：
    - run_once(prompt, tools, opts) -> RunTurn { assistant_blocks, tool_uses: Vec<ToolUse>, usage }。
    - stream_with_callbacks(builder, Callbacks { on_text, on_tool_use_ready, on_usage })：工具可用即触发回调。
  - 可选：在聚合流中加入“早触发”钩子，使上层更容易实现就绪即调度（保持并发策略不变）。
  - 文档：示例对齐 TS 调用体验（少参数 + 约定优于配置）。

- M3：错误模型与观测（中优先级）
  - 统一 SDK 错误类型（Network/Timeout/Http/Parse/Aborted），减少上层分散处理底层细节。
  - 在聚合流与高阶接口内打 span 与关键指标（请求耗时、字节大小、状态码分布、工具就绪数量）。

- M4：兼容与迁移收尾（低优先级）
  - 保留旧路径的 feature gate（如 RUST_SDK_AGGREGATED=1 时启用新聚合流）。
  - 完成主要路径迁移后，清理旧的 messages().stream(...) 低层拼装路径。

API 草案（示意）
- 聚合事件：
  - enum AggregatedEvent {
      TextDelta { text: String, is_final: bool },
      ToolUseDelta { index: usize, partial: String },
      ToolUseReady { index: usize, id: String, name: String, input: serde_json::Value },
      UsageDelta { usage: MessageDeltaUsage },
      MessageStop,
    }
- 新接口：
  - messages().stream_aggregated(builder) -> impl Stream<Item = Result<AggregatedEvent, SdkError>>
  - messages().run_once(prompt, tools, opts) -> Result<RunTurn, SdkError>
  - messages().stream_with_callbacks(builder, Callbacks { on_text, on_tool_use_ready, on_usage })
- 工具声明辅助：
  - declare_tools_from(view: &ToolRegistryView) -> Vec<Tool>

兼容与开关
- 在高速迭代阶段不承诺长期保留旧的 messages().stream(...) 低层接口；短期内可通过 env/feature 开关选择聚合路径以便迁移，后续可能移除旧路径。
- 协议未来变更（如新增事件类型/字段）优先在 SDK 内适配，上层引擎免于跟进细节。

风险与回滚
- 风险：聚合状态机正确性、乱序/并发事件处理、跨版本兼容。
- 缓解：
  - 保留旧接口与上层旧实现路径的开关，双轨运行一段时间。
  - 添加端到端集成测试（多 ToolUse、不同并发策略、长文档、错误路径）。
  - 强化 tracing 与指标，便于回溯与性能分析。
- 回滚：关闭聚合流开关，上层回退到现有低层事件路径。

预期收益
- 上层引擎代码显著收敛到“策略与编排”，协议细节封装在 SDK。
- 适配协议变更成本降低（主要聚焦 SDK），与 TS 官方 SDK 体验更趋一致。
- 更好的可测试性与可观测性，为后续“早触发”与“Provider 抽象”铺路。


---

## TS 高阶接口对齐与 Rust 实现映射（2025-09-03）

概述
- Claude Code 依赖 TypeScript SDK 的高阶流式与聚合能力（MessageStream 等）。本节对齐 TS SDK 的能力边界，并给出在 Rust SDK 中的一致化实现方案与模块映射，便于后续按阶段落地。

一、TS 高阶能力要点（来源：src/lib/MessageStream.ts 等）
- 高阶对象：MessageStream
  - 生命周期控制：controller.abort()；withResponse() 返回 { data, response, request_id }
  - 便利方法：done()、finalMessage()、finalText()
  - 事件模型：on/once/off/emitted（基于内部 listeners）；事件包括：
    - connect、streamEvent(event, snapshot)、text(delta, snapshot)、citation、inputJson(partial, snapshot)、thinking、signature、message、contentBlock、finalMessage、error、abort、end
  - 累积与快照：#accumulateMessage(event) 增量构建当前 Message 快照；
    - tool_use 的 input 采用“原始 JSON 片段累积”策略（JSON_BUF_PROPERTY）并通过 partialParse(jsonBuf) 生成“就地可用的快照”
  - 可读流互转：fromReadableStream()/toReadableStream()
- 流式底座：core/streaming.ts::Stream
  - SSE 解析与 JSON 反序列化；支持 tee() 分叉；toReadableStream()
  - event 范畴：message_start/delta/stop、content_block_start/delta/stop、ping、error
- 消息与类型：resources/messages/messages.ts
  - Message/ContentBlock/MessageStreamEvent 等类型与 delta 结构（text_delta、input_json_delta、thinking_delta、signature_delta）

二、Rust SDK 等价实现方案
- 高阶对象：MessageStreamRunner（新）
  - API：
    - messages().stream(params) -> MessageStreamRunner（等价 TS 的 MessageStream.createMessage）
    - with_response() -> { runner: MessageStreamRunner, response: reqwest::Response, request_id: Option<String> }
    - abort()/is_aborted()/is_ended()
    - done()/final_message()/final_text()（异步便捷函数）
    - 事件订阅：
      - on(Event, handler) 与 once(Event, handler)（以多播 channel/broadcast 实现）
      - 或提供 subscribe() 返回 tokio::mpsc::Receiver<Event> 的统一事件流
  - 事件枚举：MessageStreamEvent（SDK 高层）
    - TextDelta { text, snapshot_text }
    - InputJson { partial_json, snapshot: serde_json::Value }
    - ThinkingDelta { thinking, snapshot }
    - Signature { signature }
    - ContentBlock(ContentBlock)
    - Message(Message)
    - FinalMessage(Message)
    - StreamEvent(raw: provider::MessageStreamEvent, snapshot: Message)
    - Error(AnthropicError)、Abort(APIUserAbortError)、End
  - 快照与累积：
    - 在 runner 内维护 CurrentMessageSnapshot；对 input_json_delta：
      - 工具输入缓冲：HashMap<usize, String>（index -> raw buf）
      - best-effort 部分解析：尝试 serde_json::from_str(&buf)，若失败保留上一 snapshot（可选：引入 third_party/partial_json_parser 的 Rust 版本以改进）
      - content_block_stop 时若存在完整 JSON 尝试一次性 parse 并覆盖 snapshot
- 聚合流（AggregatedEvent）：stream_aggregated(params)
  - 与既有计划一致，提供更扁平的高层事件：
    - TextDelta、ToolUseDelta { index, partial }、ToolUseReady { index, id, name, input }、UsageDelta、MessageStop
  - 内部基于 MessageStreamRunner 做增量聚合，减少 Engine 侧粘合代码
- 取消与响应：
  - 取消：Tokio CancellationToken + reqwest 请求取消
  - with_response：透传 Response 与 request-id 头

三、模块与文件映射（建议路径）
- src/streaming/stream.rs（现有）：承接 core/streaming.ts 级别能力（SSE 解析、tee、to_readable_stream）
- src/lib/message_stream.rs（新增）：MessageStreamRunner 的事件循环、快照管理、on/once 订阅
- src/resources/messages/stream.rs（可选拆分）：messages().stream(params) 与 with_response glue 层
- src/resources/messages/mod.rs：扩展 messages().stream(...) 返回 MessageStreamRunner；新增 messages().stream_aggregated(...)
- src/types/aggregated.rs（新增）：定义 AggregatedEvent、ToolUseReady 等聚合事件

四、实现阶段与验收
- M1：MessageStreamRunner（基础事件 + 快照 + 取消 + with_response）
  - 覆盖事件：connect/streamEvent/text/inputJson/thinking/signature/contentBlock/message/finalMessage/end/abort/error
  - 工具输入：HashMap<usize, String> + best-effort partial 解析；content_block_stop 统一再 parse
  - 提供 subscribe() 统一事件流接口；提供 done()/final_message()/final_text()
  - 测试：单 ToolUse、多 ToolUse 顺序与内容一致；异常/中止分支；request_id 获取
- M2：stream_aggregated(...) 与回调
  - 映射 ToolUseDelta/ToolUseReady，完成与 Engine 的“就绪即调度”对接点
  - 回调版 stream_with_callbacks（on_text/on_tool_use_ready/on_usage）
  - 示例：examples/stream_aggregated.rs、examples/callbacks.rs
- M3：partial JSON 增量解析增强
  - 评估/引入 Rust 端“partial json parser”实现；或维持“保留上次成功快照 + block_stop 强一致”策略
  - 覆盖错误注入测试与性能评测
- M4：文档与迁移
  - docs：对齐 TS 用法示例；差异点与迁移建议
  - 清理 Engine 端冗余粘合代码（当上层迁移完毕）

五、兼容性与策略
- 高速迭代阶段，不保证后向兼容；短期通过 env/feature 开关选择聚合路径；后续可能移除旧路径
- 并发与权限模型由上层引擎决定，本层仅提供事件与聚合便利，不改变策略语义

六、使用示例（Rust）
```rust
let (runner, response, request_id) = client
    .messages()
    .stream(builder)
    .with_response()
    .await?;

let mut rx = runner.subscribe();
while let Some(event) = rx.recv().await {
    match event {
        MessageStreamEvent::TextDelta { text, .. } => print!("{}", text),
        MessageStreamEvent::InputJson { partial_json, .. } => log::debug!("partial: {}", partial_json),
        MessageStreamEvent::FinalMessage(msg) => println!("\n[final] {}", serde_json::to_string(&msg)?),
        MessageStreamEvent::Error(e) => eprintln!("error: {}", e),
        _ => {}
    }
}
```


---

## Claude-Code 实际用法与对齐结论（2025-09-03）

信息来源
- ts_source/Claude-Code/src/services/claude.ts

关键调用与行为
- 使用 TS SDK 的 Beta 通道：anthropic.beta.messages.stream(params, { signal }) 返回 BetaMessageStream
- 消费方式：for await (const part of stream) 仅用于计算 TTFT（在 message_start 到达时记录时间）；随后直接调用 stream.finalMessage() 获取最终 Message
- 参数要点：
  - system 采用 TextBlockParam 列表，并在合适位置附加 { cache_control: { type: 'ephemeral' } } 以启用提示缓存
  - 可选 thinking: { type: 'enabled', budget_tokens }（当 maxThinkingTokens > 0）
  - 按需传入 tools（以 JSON schema 形式），但在该文件中未直接处理客户端 tool_use 执行，仅作为定义传入
  - 使用 betas（当启用缓存并存在可用 betas 时）
  - 通过 AbortSignal 实现取消；记录 request_id 以便日志与观测

与 Rust SDK 的对齐建议（落地到前述“TS 高阶接口对齐与 Rust 实现映射”）
- Beta 通道
  - 提供 client.beta().messages().stream(params) -> MessageStreamRunner
  - 允许传入 betas 与 thinking 参数，并正确序列化到请求（含 CacheControl::ephemeral 支持）
- 流式 Runner 能力
  - MessageStreamRunner 实现 futures::Stream<Item = MessageStreamEvent>，以便上层 while let Some(event) = runner.next().await 对齐 TS 的 for await 语义
  - 同时保留便捷方法：done()/final_message()/final_text()，并暴露 request_id()
  - 暴露 with_response() -> { runner, response, request_id }，与 TS 的 withResponse 语义一致
  - 取消：支持外部 CancellationToken/Abort，并在中止时使 next() 返回错误或尽快结束
- 部分 JSON 聚合
  - 维持在 Runner 内进行 input_json_delta 的原始字符串缓冲与最终 parse（content_block_stop 时一次性 parse）；可后续评估“增量容错解析”实现
- 示例对齐
  - TTFT：在收到 MessageStartEvent 时记录时间；示例展示“仅测量 TTFT + 调用 final_message()”的最简模式
  - system 缓存断点：提供简单 Helper 将 TextBlockParam 附带 { cache_control: ephemeral } 注入到首尾或指定 block

影响评估
- 上层（Engine/TUI/CLI）与 TS 一样可选择“粗粒度消费”：仅计时并最终取 final_message；或切换到更细粒度事件/聚合回调
- 该对齐不会改变既有并发/视图/权限模型，仅是 SDK 封装层的易用性增强


---

## 差距与行动项（基于 Claude-Code 用法对比）

已对齐能力（现状可用）
- 流式消费：Engine 端基于 SSE 事件循环可实时输出 text_delta，并在回合结束返回最终文本；可收集 usage 元数据。
- 工具输入聚合：基于 InputJsonDelta 按 content_block index 缓冲 partial_json，并在 ContentBlockStop 时一次性 parse，得到完整 ToolUse.input。
- 工具执行：支持只读/并发策略（Sequential/Parallel/Auto、safe_mode、视图过滤），能并行执行只读工具。
- 提示缓存：system 与 user 提示支持 CacheControl::ephemeral 注入。

待补强项（建议在 SDK 层或 Engine 对外接口补充）
- 高阶流式对象：提供 MessageStreamRunner，暴露 subscribe()/done()/final_message()/final_text()/with_response()/request_id()/abort() 等便捷方法，使上层调用姿势贴近 TS。
- 聚合流接口：messages().stream_aggregated(...) 在 SDK 内发出 ToolUseReady 事件，减少 Engine 侧状态机与拼装代码。
- Beta 通道与 thinking：支持在请求中直通 betas 与 thinking 参数（当上层开启扩展思维/提示缓存等特性时对齐 TS 用法）。
- 取消能力：对外暴露 CancellationToken/Abort 句柄，允许上层在 CLI/TUI 中断当前流式会话。
- request_id：从响应头提取并在高阶接口中返回，便于诊断与观测（对齐 TS 的 withResponse/request_id）。
- 便捷 API：run_once 与 stream_with_callbacks（on_text/on_tool_use_ready/on_usage）以减少常见场景样板代码。
- 观测与错误模型：统一 SDK 错误分类（Network/Timeout/Http/Parse/Aborted），在聚合流/高阶接口打 span 与关键指标（TTFT、请求耗时、字节大小、工具就绪数量、状态码分布）。

建议实施顺序（迭代推进）
1) M1：实现 MessageStreamRunner 与 stream_aggregated（含 ToolUseReady），补充 examples/stream_aggregated.rs。
2) M2：加入回调风格 stream_with_callbacks 与便捷 run_once，补充 examples/callbacks.rs。
3) M3：统一错误模型与指标观测；继续采用“block_stop 强一致”解析策略，后续再评估增量容错解析器。
4) M4：通过 feature gate 双轨迁移，完成后清理旧低层路径。

- 如需增量更新，请在相应章节中补充或修正，不建议在非结构化区域添加过多细节以免失真。

---

## 改造计划进度更新（2025-09-03）

本次扫描范围与依据：src/streaming/mod.rs、src/resources/messages.rs、src/http/streaming.rs、src/types/*、examples/*。

- 已完成/可用
  - MessageStream 高阶对象：提供 on_text/on_stream_event/on_message/on_final_message/done/final_message/ended/errored/aborted 等便捷能力；实现 futures::Stream<Item = Result<MessageStreamEvent>> 语义。
  - 工具输入增量聚合：MessageStream::from_http_stream 内按 (index, id) 缓冲 partial_json，尝试 serde_json::from_str 聚合；在 ContentBlockStop 搭配最终块覆盖，形成“就绪可用”的 ToolUse.input 快照。
  - 工具就绪回调：on_tool_ready 与 on_tool_ready_execute 已实现，解析到完整 ToolUse 后立即触发，可即时并行执行工具并回传结果。
  - Beta/Thinking：当请求包含 thinking 配置时自动注入 anthropic-beta: interleaved-thinking-2025-05-14 头，满足扩展思维场景。
  - MessagesResource.create_stream/stream：走 HTTP 流（SSE）并产出 MessageStream，示例与测试可用。

- 部分完成
  - 取消/Abort：MessageStream.abort() 目前为占位（未真正取消底层 HTTP 流），注释提示“真实实现应取消 HTTP 请求”。
  - request_id：from_http_stream 能获得并存储 request_id，但尚无对齐 TS 的 with_response() 风格接口统一返回 { stream/runner, response, request_id }。

- 尚未实现（与计划差距）
  - 聚合流接口与事件：messages().stream_aggregated(...) 与 AggregatedEvent 类型。
  - 高阶 Runner：MessageStreamRunner（统一 subscribe()/with_response()/abort 取消到底层）。
  - 便捷接口：run_once、stream_with_callbacks（on_text/on_tool_use_ready/on_usage）。
  - 工具声明辅助：declare_tools_from(view)/from_registry_view(view)。
  - 统一错误模型与指标：Network/Timeout/Http/Parse/Aborted 分类与关键指标埋点（TTFT、耗时、字节大小、工具就绪数量等）。

- 建议的下一步（对齐 M1/M2）
  - M1：在现有 MessageStream 之上提供 messages().stream_aggregated(...)，在 SDK 内完成 ToolUse 输入聚合并发出 ToolUseReady；补充 examples/stream_aggregated.rs；打通 abort 到 HTTP 层。
  - M2：提供 stream_with_callbacks 与 run_once；补充 with_response() 能力，统一暴露 request_id；根据需要引入 subscribe() 或沿用现有 broadcast 机制暴露统一事件流。

- 影响评估
  - 上层 Engine 中当前的 tool_input_buffers 与 "ContentBlockStop 再 parse" 路径可在迁移后下沉至 SDK 聚合层，减少重复状态机代码，调用方只需消费 ToolUseReady/FinalMessage 等高层事件。
