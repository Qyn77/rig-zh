# LanceDB 示例

先安装 Protobuf 编译器（macOS：`brew install protobuf`），或把 `PROTOC` 设置为你已有的 `protoc` 可执行文件路径。然后运行 `cargo check --locked --examples` 检查 0.46.0 API。`p42.rs` 把第 42 页的数据结构、写入、索引、查询与过滤片段补全为一个可编译程序。小于 256 行时跳过 PQ 训练，按精确搜索运行。

运行该程序需要 `OPENAI_API_KEY`，并会写入本机 `data/lancedb-store`。本项目已启用 `lancedb/aws` 与 `lancedb/dynamodb`；运行 S3/DynamoDB 示例还需配置 AWS 凭证和存储桶。

`p34_001.rs` 提供 Lambda 中使用的 `build_agent` 函数，附带命令行入口：运行前设置 `LANCEDB_URI` 和 `OPENAI_API_KEY`，并准备好 `montreal_open_data` 表。它的编译不能视为 Lambda 已部署。其他连接片段需修改为你实际的路径/存储桶。

依赖兼容性：`aws-smithy-json 0.63.0` 无法与 `aws-smithy-types 1.7.0` 一起编译。本项目及文档配置固定 `aws-smithy-types = "=1.6.3"`，并保留经过检查的 `Cargo.lock`；请使用 `--locked` 重现结果。
