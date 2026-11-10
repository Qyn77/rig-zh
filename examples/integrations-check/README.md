# 可选集成示例（Rig 0.46.0）

`cargo check --locked --examples` 检查 MongoDB、Neo4j、Qdrant、MCP 和 Ollama/llama.cpp 的示例。每个文件名中的数字对应中文版的页编号与代码块编号；`p47_001.rs` 同时包含第 1、2 块。

无需密钥即可执行 MCP 工具调用测试：

```sh
cargo run --locked --example p47_005
```

它会启动进程内 MCP 服务器，建立真实 MCP 客户端连接，通过 Rig 工具执行器调用加法并断言 `2 + 5 = 7`，随后关闭连接；不调用语言模型。

其他运行条件：

- `p39_000`：本机 Ollama，已安装 `qwen3` 模型；其余本地客户端配置片段仅构造对象。
- `p43_002`：`OPENAI_API_KEY`、`MONGODB_CONNECTION_STRING`，MongoDB Atlas collection/vector index 配置须按章节准备。
- `p44_001`：`OPENAI_API_KEY`，本机 Neo4j，代码中的连接账号改为你的账号。
- `p45_002`：`OPENAI_API_KEY`，本机 Qdrant gRPC 端口 6334；代码会创建 collection 并写入示例文档。
- `p47_001`、`p47_003`：`OPENAI_API_KEY`，支持 streamable HTTP 的 MCP 服务（示例地址 `http://localhost:8080`）。

服务型示例编译验证不代表对应服务已部署或密钥可用。不要把密钥写入代码。
