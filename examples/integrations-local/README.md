# 本地推理示例

`cargo check --locked --examples` 检查 Candle 与 Fastembed 的 0.46.0 API。

- `p39_006` 运行前需在当前工作目录 `model/` 放置受支持的 `config.json`、`tokenizer.json` 和单文件 `model.safetensors`。
- `p39_008` 首次运行会下载 Fastembed 的 AllMiniLML6V2Q 模型；构建还需要 ONNX Runtime。

本次不会下载模型权重或声称完成模型推理。查阅根目录验证报告了解实际检查状态。
