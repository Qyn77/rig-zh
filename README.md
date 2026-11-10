# Rig 中文文档 · 0.46.0 实际适配版

[在线阅读中文版](https://qyn77.github.io/rig-zh/) · [升级与验证报告](https://qyn77.github.io/rig-zh/versions.html)

52 页、319 个代码块；中文搜索、目录、代码复制和深浅色切换可离线使用。本地先按下方命令构建，再双击 **打开中文版.html**。

这次已实际更新 **67 个代码块**；**230 个 Rust 代码块全部通过编译检查**，不再只给旧代码加说明。片段附完整编译上下文，所有项目精确锁定 Rig 0.46.0 并附 Cargo.lock。

核心 5 个测试、航班 1 个测试，以及记忆、回放、审批、MCP、自定义 provider 等离线程序实际运行通过。**真实模型、外部数据库、云部署与本地模型权重未运行**，需要你自己的凭证、服务或模型文件。

详细改动、测试范围和逐页清单：[0.46.0 升级与验证](https://qyn77.github.io/rig-zh/versions.html)。原站 17 张图片失效，仍保留缺失说明。

## 编译与运行

需要 Rust 1.97.1。首次 Cargo 执行会联网下载依赖。

```sh
cd ~/Desktop/rig-docs-zh
python3 scripts/check_examples.py --core
# 全部检查额外需要 protoc、C/C++ 工具链，依赖较大
python3 scripts/check_examples.py
```

执行真实模型的完整快速入门（先配置 OPENAI_API_KEY）：

```sh
cargo run --locked --manifest-path examples/core/Cargo.toml --example p02_003
```

不需要把 API Key 写进代码或文档。每个 Rust 代码块下方的「完整代码与上下文」可用于定位对应文件；相关项目 Cargo.toml 给出依赖和 feature。

## 文件

- `dist/`：构建生成的本地网站与示例 ZIP，不提交到 Git。
- `assets/`：网站样式与交互脚本。
- `content/`：中文正文；`content/codes/` 为 0.46.0 代码。
- `source/`：未改写的官方英文快照。
- `examples/`：7 个编译项目、锁文件与离线回归。
- `validation.json`：230 个块到编译源文件的映射和哈希。
- `validation-logs/`：最终成功的编译与运行日志。
- `code-changes.diff`：原文代码和新版代码的实际差异。
- `VERSIONS.md`、`QA.md`：升级内容与验证边界。

## 本地生成网站

```sh
git clone https://github.com/Qyn77/rig-zh.git
cd rig-zh
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r requirements.txt
python3 scripts/build.py
python3 scripts/verify.py
```

保留正文的代码和锚点占位符。若修改代码，需同时修改对应 `examples/` 输入并重新运行检查；验证器会拒绝网页与检查文件不一致的内容。`source/` 保留溯源，不用于生成新版代码。`fetch.py` 会覆盖英文快照，仅在准备重新核对所有译文时使用。

来源：[Rig 官网](https://www.rig.rs/docs)。非官方中文适配版，原作者 MIT 许可见 `LICENSE.upstream`。

## GitHub Pages

推送到 `main` 会触发 `.github/workflows/pages.yml`：安装固定版本的 Python 依赖，生成 `dist/`，通过页面与代码映射检查，再发布到 GitHub Pages。Pull Request 只构建和检查，不部署。

工作流使用 GitHub 自动提供的 `GITHUB_TOKEN`，无需配置模型 API Key 或额外发布密钥。Pages 的发布来源设为 **GitHub Actions**。此文档构建工作流不重新执行大型 Rust 全量编译；Rust 检查与离线运行命令见上文。
