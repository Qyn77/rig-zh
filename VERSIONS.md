# Rig 0.46.0 升级与验证

**代码已实际升级，不再只保留旧示例并加译注。** 2026-11-10 完成适配；固定 `rig = "=0.46.0"`，Rust 1.97.1。

52 页目录与 319 个代码块仍完整保留。其中 **230 个 Rust 代码块全部通过编译检查**：完整程序直接检查，零散片段补齐 imports、类型和初始化上下文后检查。网页显示的代码逐字对应检查文件；没有用另写的简化程序代替原片段。共更新 **67 个代码块、33 个页面的代码**，包括 API、配置、依赖与命令。

## 具体更新

- 模型注册：`Catalog::connect` / `connect_with`，`vendor/model` 引用；模型信息从 `resolve(..)?.spec` 读取。
- HTTP 后端：配置的 `connect(http)` 替代移除的客户端 `with_http`。
- 记忆：`MemoryStore`、`Store` / `AsyncStore`；显式窗口和摘要流程；保存失败通过 `PromptError::Memory` 返回并保留已完成响应。
- 持久化运行：`ModelTurn::from_response`、待处理调用的 `answer`、`run.answer` / `answer_all`、`Aside`。
- 录制回放：`Handlers` / `over_handlers`；修正记忆 I/O 不属于录制 effect 的说明。
- MCP：rmcp **3.5.0**，`McpConnection`、`ClientConfig` / `ServerConfig` 和连接关闭流程。
- 自定义 provider：`Wire::Event`、类型化事件重组、`Arc<Message>`、`cache_key`；保留未知服务商字段。
- 图像常量、RAG 文档反序列化、缺失依赖、航班模块声明、Lambda runtime、LanceDB 索引和 AWS 配置等同步修正。

## 实际验证范围

- 编译：230 / 230 Rust 代码块，7 个锁定依赖的 Cargo 项目；每个块下方可打开完整上下文。
- 离线单元测试：核心 5 / 5、航班格式化 1 / 1 通过。
- 实际离线运行：记忆、窗口与摘要、错误保留、录制回放、审批允许/拒绝、目录和模型构造、自定义 provider 解码、MCP 客户端到服务端工具调用均通过。MCP 的 `2 + 5 = 7` 是实际进程内传输调用。
- 未进行真实模型、Discord、RapidAPI、数据库服务、AWS 或遥测平台端到端调用；没有使用真实 API Key，也没有下载或运行本地模型权重。编译通过不能证明账户权限、服务可用性、网络及部署配置正确。

## 下载与运行

[下载全部示例项目 ZIP](examples-046.zip) · [逐块验证记录 JSON](validation.json) · [实际代码差异](code-changes.diff)

桌面文件夹已有同一份 `examples/`，不必再次下载。首次运行 Cargo 会下载锁定依赖。完整重查中的 LanceDB 需要 C/C++ 工具链与 `protoc`（可安装 protobuf，或通过 `PROTOC` 指定已有可执行文件）；AWS 相关兼容性版本以附带 Cargo.toml / Cargo.lock 为准。

```sh
cd ~/Desktop/rig-docs-zh
# 先执行轻量核心编译与离线测试，不需要模型密钥
python3 scripts/check_examples.py --core
# 全部编译与离线回归（含较大的数据库/本地推理依赖）
python3 scripts/check_examples.py
```

真实模型示例需要自行设置相应环境变量。例如：

```sh
cargo run --locked --manifest-path examples/core/Cargo.toml --example p02_003
```

核心片段的完整文件包含辅助上下文；有些仅演示构造或类型定义，不会输出答案。参考页面正文选择所需的完整程序。`examples/guides/README.md` 列出 Discord、航班、Lambda、遥测的 feature 和启动方式。

## 逐页改动

| 章节 | 更新的代码块（从 1 起计数） |
| --- | --- |
| [安装](01.html) | 1, 2, 4, 5 |
| [快速入门](02.html) | 2 |
| [架构](03.html) | 2, 3 |
| [服务商与客户端](05.html) | 5, 7, 9 |
| [AgentRunner](08.html) | 3 |
| [持久化运行](10.html) | 1 |
| [嵌入 Embeddings](14.html) | 8 |
| [记忆 Memory](16.html) | 1, 3, 4, 5, 6, 7, 8, 9 |
| [媒体功能：图片、音频与转写](19.html) | 1 |
| [可观测性](20.html) | 5 |
| [测试](21.html) | 1 |
| [录制与回放](22.html) | 1, 2, 3, 4 |
| [构建 RAG 系统](25.html) | 2, 4 |
| [文本提取与分类](26.html) | 2 |
| [模型路由](27.html) | 6, 7, 8 |
| [构建 Discord 机器人](29.html) | 3 |
| [构建航班搜索助手](30.html) | 2, 4 |
| [编写自己的服务商](32.html) | 6, 7, 8, 9, 12 |
| [将 Rig 部署到 AWS Lambda](33.html) | 1 |
| [将 Rig 与 LanceDB 部署到 AWS Lambda](34.html) | 1, 7 |
| [生态集成](35.html) | 1 |
| [模型服务商](36.html) | 2 |
| [OpenAI](37.html) | 10 |
| [本地模型](39.html) | 6, 8 |
| [向量存储](40.html) | 6 |
| [LanceDB](42.html) | 1, 6 |
| [MongoDB](43.html) | 1 |
| [Neo4j](44.html) | 1 |
| [Qdrant](45.html) | 1 |
| [SurrealDB](46.html) | 1, 4 |
| [模型上下文协议](47.html) | 1, 2, 3, 4, 5, 6 |
| [示例集](48.html) | 1 |
| [故障排查](50.html) | 9, 11 |

## 原文、授权与验证边界

来源是 [Rig 官方文档](https://www.rig.rs/docs)，本地为非官方中文适配版，未经官方审核。`source/` 原始英文快照与 SHA-256 保持不变；新版代码位于 `content/codes/`，完整检查程序位于 `examples/`。原始 MIT 许可保留。

17 张原站图片仍缺失，页面明确显示原链接。部署文章的性能数字属于历史测量，未重新测量；计算错误已修正。外部文档与仓库不属于本地完整翻译范围。

依据：[0.46.0 发布源码与迁移指南](https://docs.rs/crate/rig/0.46.0/source/MIGRATING.md)、本机 Cargo 下载的对应版本源码，以及真实编译/运行输出。完整记录见本地 `validation-logs/` 和 `QA.md`。
