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
- 如需增量更新，请在相应章节中补充或修正，不建议在非结构化区域添加过多细节以免失真。
