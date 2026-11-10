# SurrealDB 示例

`cargo check --locked --examples` 检查 0.46.0 API。`p46_001.rs` 包含主流程；`p46_002.rs` 包含主流程与独立的阈值搜索函数；`p46_003.rs` 包含自定义存储配置。

内嵌 Mem 数据库不要求独立服务，但嵌入模型仍需要 `OPENAI_API_KEY`。已补齐 namespace/database 选择，避免连接成功后因未选定数据库而在查询时报错。
