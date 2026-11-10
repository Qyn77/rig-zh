# Rig 中文文档设计约定

## 1. 气质与身份
来源是用户指定的 https://www.rig.rs/docs，已用浏览器读取渲染样式并查看桌面页面。沿用三栏技术文档布局、暖纸白/黑底和铜色交互强调。标记为非官方中文译本，不仿冒官方身份。不增加营销页和装饰图片。

## 2. 颜色
从官网 computed style 得到：浅色背景 #f6f3ec、正文 #393e35、链接 #984725；深色背景 #000、正文 #ddd9d1、标题 #f6f3ec、强调 #d97e4a、次级文字 #a8a8a0、代码背景 #111。
适配 token：浅色 surface #fffdf8，muted #64685f，border #ddd8ce，code #eeebe4，accent-soft #eee1d5；深色 surface #101010，border #292824，accent-soft #23180f。警示面板使用同一铜色系，错误文字 #b42318，成功 #28734a。
代码配色：keyword #8f4325/#e69b72，string #35724b/#9ecb9b，comment #666b61/#a3a69a，number #694e99/#c8abe6。

## 3. 排版
正文选择 PingFang SC / Microsoft YaHei / system-ui，中文不依赖网络字体。代码用 SFMono-Regular / Menlo / Consolas / monospace。
正文 16px，行高 1.85（中文适配）；标题 42/30/22px，行高 1.25/1.45/1.5；导航14px、元信息12px、代码14px/1.7。移动端 H1/H2/H3 32/26/20px，代码13px。原站正文16/28、标题42/50.4，是本稿适配依据。

## 4. 间距与布局
4px 基准：4/8/12/16/20/24/32/40/48/64。顶部64px（原站）；左栏280px（原站300px，中文缩窄），右栏220px（原站240px），内容区最大800px。最大整体宽1480px。正文使用 document 自然滚动，左侧目录与右侧目录因高度可能超过屏幕各自 overflow:auto，sticky top64px。代码与表格单独横向滚动。参考 sticky-aside https://github.com/changeroa/StyleGallery/blob/main/patterns/split-sidebar/sticky-aside.md。
1040px 以下隐藏右栏；768px 以下目录切换为内联展开面板，正文单列。所有网格内容 min-width:0，长链接 overflow-wrap:anywhere。

## 5. 可复用组件
- Header：品牌文字、非官方标记、搜索触发、主题开关、移动目录按钮；按钮均有 hover/active/focus-visible，44px触摸目标。
- Sidebar：按官方目录分组，details/summary 折叠，当前链接 aria-current=page；桌面独立滚动。
- Prose：共用标题/列表/表格/引用/链接/代码规则，不逐页写样式。
- CodeBlock：语言标签、复制按钮、横向滚动 pre；复制成功/失败均通过 aria-live 提示，可键盘选择。
- Notice：译注与版本差异，清楚标明不是原文；升级后展示 0.46.0 代码；保留 source 原文快照，在 Notice 链接到逐块编译与运行记录。
- OnThisPage：保留原始 heading ID，锚点在固定顶部下可见；短文不撑满栏。
- Search：原生 dialog，搜索输入、结果列表、无结果状态；Esc关闭，关闭后焦点回到触发按钮；支持 Ctrl/Command K，中文与API标识符检索。
- Pagination：上一篇/下一篇真实链接，首尾不放无效按钮。
- Primitive showcase：/showcase.html 提供按钮、警示、代码、表格和搜索入口；先检查此页再组合最终内容。

## 6. 交互与动效
无装饰动效；按压transform位移1px，focus使用2px铜色outline。不引入动画依赖。尊重 prefers-reduced-motion。主题保存在localStorage，失败不影响阅读。搜索异步按需加载本地索引，无第三方请求。

## 7. 层次
原图缺失使用带边框的 figure 和中文状态说明，保留源链接。原站的细分隔线与表面色阶为主，代码8px圆角，弹窗12px圆角；搜索弹窗黑色20%遮罩。浅色弹窗投影0 16px 48px #0003，深色用边框。页面无卡片网格和hero。

## 8. 可访问性与验证边界
中文 lang=zh-CN；跳至正文链接、导航landmark、main、层级标题；正文对比4.5:1，键盘可访问所有控件，输入有label，dialog可Esc关闭；移动无整页横向滚动。
已安装工具只允许 cua_repl 浏览器验证，不能用外部 Playwright 操纵浏览器。Lighthouse 专用评分链路和 /visual-qa skill 在当前工具目录不可用；以实际 CUA截图、DOM尺寸和交互检查记录证据，不报告虚构的评分。手机viewport工具可能缩放而非改变CSS视口，必须核实实际DOM宽度。
