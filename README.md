# Rig 中文文档

[在线阅读中文版](https://qyn77.github.io/rig-zh/) · [升级与验证报告](https://qyn77.github.io/rig-zh/versions.html)

中文搜索、目录、代码复制和深浅色切换可离线使用。当前适配版本和验证日期见在线升级报告及 `validation.json`。本地先按下方命令构建，再双击 **打开中文版.html**。

初始 0.46.0 适配覆盖 52 页、319 个代码块，实际更新 **67 个代码块**；当时 **230 个 Rust 代码块全部通过编译检查**。片段附完整编译上下文，所有项目精确锁定对应 Rig 版本并附 Cargo.lock。

核心 5 个测试、航班 1 个测试，以及记忆、重放、审批、MCP、自定义 provider 等离线程序实际运行通过。**真实模型、外部数据库、云部署与本地模型权重未运行**，需要你自己的凭证、服务或模型文件。

详细改动、测试范围和逐页清单：[升级与验证](https://qyn77.github.io/rig-zh/versions.html)。初始快照中原站 17 张图片失效，仍保留缺失说明。

## 编译与运行

使用 `rust-toolchain.toml` 固定的 Rust 版本（初始为 1.97.1）。首次 Cargo 执行会联网下载依赖。

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

## GitHub Pages 与自动更新

所有发布（包括手动推送 main）都先在 GitHub Actions 编译全部 **7 个 Cargo 项目**：`cargo check --locked --all-targets --all-features`，并运行既有离线回归。每个项目的检查证明都绑定同一份源文件哈希；全部通过后才生成、验证并部署网站。PR 只验证，不部署。

每天北京时间约 **09:23** 检查官方 Cargo 注册表中 `rig` 的最新稳定版本，跳过预发布和撤回版本。GitHub 的定时任务可能延迟；长期没有仓库活动时也可能停用，可在 Actions 中恢复。

发现新版后的流程：

1. 抓取官网完整目录，保留已有页面 ID，识别新增/删除章节。
2. 使用 DeepSeek 翻译中文、迁移实际 API 调用、更新 7 份 Cargo.lock；再次发起独立审校请求。
3. 在独立 CI 任务中重新编译全部项目，运行离线回归，核对译文结构、代码展示与编译输入。
4. 检查通过后自动提交到 main，发布同一份已验证的 Pages 产物。任何失败都保留旧站；若 main 在运行期间变化，本次更新停止，等待下次重试。

### 只需配置一次 DeepSeek

打开 [仓库 Actions Secrets](https://github.com/Qyn77/rig-zh/settings/secrets/actions)，添加 **DEEPSEEK_API_KEY**。不要把密钥写入代码、Issue 或聊天。

默认使用官方 `https://api.deepseek.com/chat/completions` 和 `deepseek-v4-pro`。可在仓库 Actions Variables 设置 `DEEPSEEK_MODEL` 切换为官方支持的模型。模型调用会消耗你的 DeepSeek 余额；单次同步上限为 180 次请求、累计 100 万 token（达到阈值后停止继续请求，最后一次返回可超出 token 阈值）。只有发现新稳定版本才调用模型。

[手动检查 / 查看运行记录](https://github.com/Qyn77/rig-zh/actions/workflows/update.yml)：点击 **Run workflow**，选择 main。无需个人 GitHub Token，版本同步提交和发布使用工作流自带 GITHUB_TOKEN。若分支保护不允许该 Token 推送，提交步骤会失败，需按仓库规则处理，流程不会绕过保护。

缺少 DeepSeek Secret 时仍能检测版本；有新版时同步步骤会明确失败。无法自动解决的破坏性 API 变更、删除 feature、结构异常或审校失败也会阻止发布，并在 Actions 日志保留原因。不会把“只改版本号”当成迁移成功。

模型只获得文档和示例的受限编辑工具，不能修改工作流、验证脚本或现有测试基础设施。生成阶段只解析/下载 Cargo 依赖，不编译或执行生成代码；实际编译和离线程序放在不接收 DeepSeek 密钥的独立任务中运行，GitHub 检出不保留凭证。编译失败不会自动绕过或发布，需要检查 CI 日志修复后重试。

**验证边界**：编译和离线回归不能证明翻译绝对准确，也不能证明真实模型、数据库、云部署或本地权重可用。自动译文未经官方或人工审核。未来版本若移除现有 feature 或测试依赖，需要人工迁移；系统会保留旧站。

实现：`.github/workflows/update.yml`（检测与迁移）、`validate.yml`（全量验证）、`pages.yml`（普通提交发布）。迁移候选与编译日志保存在对应 Actions 产物中；首次配置后建议手动运行一次检查。

协议参考：[DeepSeek 官方 API](https://api-docs.deepseek.com/) · [工具调用](https://api-docs.deepseek.com/guides/tool_calls/) · [GitHub 定时触发说明](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#schedule)。

### 配置后验收

添加或更换密钥后，可运行 [Accept DeepSeek credentials and migration](https://github.com/Qyn77/rig-zh/actions/workflows/acceptance.yml)。该手动工作流真实调用 DeepSeek，翻译短文、迁移旧版 ProviderRef/会话记忆 API，并执行工具读写回合；随后在无模型密钥的独立任务中编译并离线运行生成的 Rust 程序。报告、译文和生成代码保留为 Actions 产物，不覆盖线上正文。

这项验收最多发起 8 次模型请求、累计 6 万 token 阈值，会产生少量 API 用量。它验证实际凭证和代表性流程，不能替代未来新版本的全量迁移与 7 项编译门禁。

## SEO 与搜索引擎通知

全部正文页面自动生成独立标题、中文摘要、canonical 规范网址、Open Graph/Twitter 摘要以及 WebSite、WebPage、TechArticle 和面包屑结构化数据。摘要取自当前译文，新增章节自动进入 [站点地图](https://qyn77.github.io/rig-zh/sitemap.xml)。构建检查会拦截重复网址、缺失页面和元数据错误；样式展示页设置 noindex。

`site.json` 管理站点域名和公开验证标记。迁移域名时修改 `base_url`，并在搜索平台重新验证。当前站点位于 `/rig-zh/` 子目录，robots.txt 只在域名根目录生效，因此不生成无效的子目录 robots.txt；页面通过 robots 元标签允许抓取。

仓库 Actions Secret **INDEXNOW_KEY** 用于 IndexNow 所有权证明。构建时生成公开验证文件；普通发布与 Rig 自动升级发布完成后，`notify-search.yml` 核对线上站点地图和验证文件，再向 IndexNow 提交所有规范网址。HTTP 200/202 表示请求接收（202 尚待验证），不代表已经收录或保证排名。失败会在 Actions 显示，可手动重跑 [搜索通知](https://github.com/Qyn77/rig-zh/actions/workflows/notify-search.yml)；已部署的网站不会因此回滚。

Google Search Console 需要单独操作：添加网址前缀 `https://qyn77.github.io/rig-zh/`，选择 HTML 标记，把 content 值填入 `site.json` 的 `google_site_verification`。发布后在 Search Console 点击验证，提交 `sitemap.xml`。Bing Webmaster Tools 的验证值可填入 `bing_site_verification`。这些是公开站点验证码，不是账号密码。IndexNow 不替代 Google Search Console。

规则依据：[Google 站点地图](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)、[robots.txt 位置规则](https://developers.google.com/crawling/docs/robots-txt/create-robots-txt)、[IndexNow 协议](https://www.indexnow.org/documentation)。

## 中文译文维护

[翻译规则](TRANSLATION.md)约定术语、简洁表达及技术说明边界；自动翻译、API 迁移和独立审校都会读取同一份规则。2026-11-10 的修订扫描了全部 52 篇正文，重点复核了调用重放、模型生成、钩子、工具、记忆与测试等章节，并统一目录和交叉链接。具体范围与源码依据见 [审校记录](TRANSLATION-REVIEW.md)。编译通过不等于翻译准确，自动审校也不能代替专家或官方审核。
