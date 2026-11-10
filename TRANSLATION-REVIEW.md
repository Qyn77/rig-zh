# 中文译文审校记录 · 2026-11-10

目标版本：Rig 0.46.0。扫描全部 52 篇正文的术语、章节开头和易误解表述，修改 43 篇的用词、链接文字或技术说明。重点章节对照仓库中的官方英文快照，并在涉及版本行为时核对本地下载的 0.46.0 crate 实现。这不是对 52 篇文章逐句重新翻译，也不是官方审核。

## 主要修订

- **调用记录与重放**：重写说明并增加天气助手例子。明确调用日志替代的范围，以及 Hook、存储和应用代码仍需另行处理；区分逐次请求比较与主动调用 `check_replayable` 的配置校验。
- **模型生成与协议适配**：解释 Completion 的含义；纠正 `Opaque.replay` 的“再次发送历史内容”与 cassette 离线重放之间的混淆；解释 `ReplayTarget` 的历史适配职责。
- **钩子**：依据实现移除已不存在的记忆分派事件；不是照抄仍残留旧术语的源码注释。
- **测试、工具与错误处理**：Mock 只替代模型，真实工具仍会执行；`max_turns` 统计模型调用轮数，一轮可请求多个工具；类型解析通过不代表回答中的事实正确。
- **安装、架构、记忆与工作流**：解释 feature、统一入口 crate、类型擦除等术语，修正“通过 配置的”“工具接线”“脚本化轮次”等病句或生硬表述。
- **部署指南**：将原文性能数字明确标为原作者历史测量，避免被误读为本站在 0.46.0 上的实测结果。
- **全站一致性**：统一录制/回放、转录/转写、Agent/Embedding 等标题及链接文字。保留既有页面地址、锚点、代码标记及官方章节结构。

## 关键源码依据

以下均来自 crates.io 下载的精确版本源码，路径相对于对应 crate：

| 包与文件 | 核对结论 |
| --- | --- |
| `rig-cassette 0.46.0 / src/agent/replay.rs` | `register_all` 使用 `RequestCheck::Payload`；调用日志头检查与处理器注册，不调用 Agent 的配置比对方法。 |
| `rig-cassette 0.46.0 / src/agent/mod.rs` | `AgentReplayExt::check_replayable` 检查运行配置、钩子列表与处理器，需调用方主动执行；Hook 会再次运行。 |
| `rig-cassette 0.46.0 / src/effect_log/replay.rs` | 请求比较有 Payload、Hash、Skip 模式；规范化时忽略特定会话 ID 和提示词缓存键。 |
| `rig-core 0.46.0 / src/effect/mod.rs` | `EffectFamily` 包含 Completion、Tool、Embed、Rerank、Retrieve、Custom，不含 Memory。 |
| `rig-agent 0.46.0 / src/agent/hook.rs` | 默认 `observes` 实现排除 Embed、Rerank、Retrieve、Custom 四种分派。 |
| `rig-core 0.46.0 / src/completion/message/native.rs` | `Opaque.replay` 控制内容是否发回产生它的模型。 |
| `rig-core 0.46.0 / src/wire/secret.rs` | `Secret` 序列化输出 `[redacted]`，而非密钥原值。 |

官方原文入口：[Recording & Replay](https://www.rig.rs/docs/concepts/recording)。原文当时仍展示 0.44.0 的 Bus 示例和记忆 effect 描述；中文页用明确的 0.46.0 适配说明解释差异。

## 验证与边界

本次 Rust 源码、代码块、Cargo 依赖和英文快照保持逐字节一致。现有构建检查仍核对全部 52 篇的标题层级、锚点、319 个代码块和 230 个 Rust 代码块映射，以及 53 个网页的 SEO 元数据与站点地图。发布仍必须通过七个 Cargo 项目的编译与既有离线回归。

未重新运行外部模型、数据库或云部署，也未重新测量原文的性能数字。本次审校降低了已发现的误译和歧义，不构成译文绝对正确的保证。自动同步的翻译、迁移和独立审校会读取 [TRANSLATION.md](TRANSLATION.md)，但模型审校本身仍可能遗漏问题。
