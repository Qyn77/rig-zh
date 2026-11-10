# Rig 0.46.0 进阶指南示例

以下命令从本目录运行，使用已提交的 Cargo.lock。所有示例依赖固定为 `rig = "=0.46.0"`。

```sh
cargo check --locked --all-targets --all-features
cargo run --locked --example custom_provider
cargo test --locked --features flight --example flight
```

- `custom_provider` 的 main 使用文档中的 Replay transport，离线断言流式响应、usage 和原始响应合并；不会访问 `api.acme.test`。同文件中的真实调用函数也通过类型检查。
- `otel_more` 是可编译的 API 片段集合；其中函数不由空 main 自动调用。
- `otel` 要求 OpenAI 密钥与 OpenTelemetry Collector；未真实连接验证。
- `discord` 要求 OpenAI 密钥、Discord token 和 `documents/` Markdown 文件；未登录 Discord。
- `flight` 要求 OpenAI 和 RapidAPI 密钥；离线测试只验证返回 JSON 的格式化逻辑，不验证第三方接口是否仍可用。
- `lambda` 要求 AWS Lambda 运行环境；未部署。

真实服务示例运行方式：

```sh
cargo run --locked --features otel --example otel
cargo run --locked --features discord --example discord
cargo run --locked --features flight --example flight
cargo run --locked --features lambda --example lambda
```

完整站点根目录的 `validation.json` 记录每个网页代码块与编译文件的对应关系。
