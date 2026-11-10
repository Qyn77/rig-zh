from pathlib import Path
from html import escape as E
from urllib.parse import urlparse,urljoin,unquote
import re,json,sys,shutil,os
from seo import load_config, metadata, description_from_markdown, write_crawl_files
import markdown
from bs4 import BeautifulSoup
from pygments import highlight
from pygments.lexers import get_lexer_by_name,TextLexer
from pygments.formatters import HtmlFormatter
R=Path(__file__).resolve().parents[1]; D=R/'dist'; S=R/'source'; C=R/'content'
config=load_config()
manifest=json.loads((S/'manifest.json').read_text())
validation=json.loads((R/'validation.json').read_text()) if (R/'validation.json').exists() else {}
VERSION=validation['rig_version']
RUST_VERSION=validation['rustc']
SNAPSHOT=max(p['fetched_on'] for p in manifest)
images=json.loads((S/'images.json').read_text()) if (S/'images.json').exists() else {}
for p in manifest:
 p['file']='index.html' if p['id']=='00' else p['id']+'.html'
 fp=C/(p['id']+'.md')
 p['zh']=fp.read_text().splitlines()[0].lstrip('# ') if fp.exists() else p['title']
paths={p['path']:p['file'] for p in manifest}
searchsvg='<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/></svg>'
themesvg='<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l2 2m10 10 2 2M5 19l2-2M17 7l2-2"/></svg>'
menusvg='<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 6h16M4 12h16M4 18h16"/></svg>'

def sidebar(current):
 out=[]
 for group in ['入门','核心概念','实战指南','生态集成','参考']:
  rows=[p for p in manifest if p['group']==group]
  out.append(f'<details class="side-group" open><summary>{group}</summary>')
  subgroup=None
  for p in rows:
   path=p['path']; sub=None
   if '/guides/advanced/' in path:sub='进阶'
   elif '/guides/deploy/' in path:sub='部署 Rig'
   elif '/integrations/model_providers' in path:sub='模型服务商'
   elif '/integrations/vector_stores' in path:sub='向量存储'
   if sub!=subgroup:
    if subgroup:out.append('</details>')
    if sub:out.append(f'<details open class="subnav"><summary>{sub}</summary>')
    subgroup=sub
   attr=' aria-current="page"' if p['id']==current else ''
   out.append(f'<a href="{p["file"]}"{attr}>{E(p["zh"])}</a>')
  if subgroup:out.append('</details>')
  out.append('</details>')
 out.append(f'<div class="side-foot">{len(manifest)} 篇中文文档 · 官网快照<br>{SNAPSHOT}<a href="versions.html">{VERSION} 升级与验证</a><a href="https://docs.rs/rig/{VERSION}/rig/">{VERSION} API 参考 ↗</a></div>')
 return ''.join(out)

def shell(title,body,current='',toc='',meta='',notice='',pager='',filename=None,description=None,source_url=None,noindex=False):
 filename=filename or ('index.html' if current=='00' else current+'.html' if current else 'versions.html')
 description=description or f'Rig {VERSION} 中文文档升级与验证报告，说明 API 示例的实际改动、完整源码、编译检查和离线运行范围，以及真实服务所需的配置。'
 head=metadata(config,title,filename,description,source_url,noindex)
 crumb='<nav class="eyebrow" aria-label="面包屑"><a href="index.html">Rig 中文文档</a>'
 if filename!='index.html':crumb+='<span aria-hidden="true">/</span><span aria-current="page">'+E(title)+'</span>'
 crumb+='</nav>'
 return f'''<!doctype html>
<html lang="zh-CN" data-theme="light"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">{head}<meta name="color-scheme" content="light dark"><link rel="icon" href="assets/favicon.svg" type="image/svg+xml"><script>try{{document.documentElement.dataset.theme=localStorage.getItem('rig-zh-theme')||'light'}}catch{{}}</script><link rel="stylesheet" href="assets/style.css"><script src="assets/app.js" defer></script></head>
<body><a class="skip" href="#main">跳到正文</a><header class="topbar"><div class="top-inner"><a class="brand" href="index.html">RIG<span>中文文档</span></a><span class="unofficial">非官方中文译本</span><div class="header-links"><a href="versions.html">版本说明</a><a href="https://www.rig.rs/docs">英文官网 ↗</a><button class="button search-trigger" id="search-open" aria-label="搜索文档" aria-haspopup="dialog">{searchsvg}<span class="search-label">搜索文档</span><kbd>⌘ K</kbd></button><button class="button icon-button" id="theme" aria-label="切换为深色">{themesvg}</button><button class="button icon-button mobile-toggle" id="menu" aria-label="打开章节目录" aria-expanded="false" aria-controls="sidebar">{menusvg}</button></div></div></header>
<div class="layout"><nav class="sidebar" id="sidebar" aria-label="章节目录">{sidebar(current)}</nav><main id="main" tabindex="-1">{crumb}<h1>{E(title)}</h1>{meta}{notice}<article class="prose">{body}</article>{pager}<footer class="footer">本文译自 Rig 官方英文文档，中文翻译未经官方审核。代码已按 Rig {VERSION} 更新；英文快照保存在 source，验证范围见版本说明。<br>原文版权归 Playgrounds / Rig 作者所有 · <a href="LICENSE.txt">MIT 许可</a> · <a href="versions.html">升级与验证记录</a> · <a href="sitemap.xml">站点地图</a></footer></main><aside class="rail" aria-label="本页目录"><h2 class="rail-title">本页目录</h2><nav>{toc or '<a href="#main">概览</a>'}</nav><div class="rail-footer">阅读提示<br>API 名称保留英文，便于查阅和复制。<a href="versions.html">查看编译与运行记录 →</a></div></aside></div>
<dialog id="search-dialog" aria-label="搜索中文文档"><div class="search-top"><label for="search-input" class="sr-only">搜索中文概念或 API 名称</label><input id="search-input" type="search" placeholder="搜索中文概念或 API 名称…" autocomplete="off"><button id="search-close" aria-label="关闭搜索">Esc</button></div><p id="search-caption" class="search-caption" role="status">搜索中文概念、方法名或英文术语。</p><ul class="search-results" id="search-results"></ul></dialog><div id="status" class="sr-only" role="status" aria-live="polite"></div></body></html>'''

def code_html(code,i):
 lang=code['lang'] or 'text'
 try:lexer=get_lexer_by_name(lang)
 except Exception:lexer=TextLexer()
 # Strip the formatter-added final newline; preserve original code bytes as DOM text.
 formatted=highlight(code['text'],lexer,HtmlFormatter(nowrap=True))
 if not code['text'].endswith('\n') and formatted.endswith('\n'):formatted=formatted[:-1]
 return f'<div class="code-block" data-code="{i}"><div class="code-head"><span>{E(lang)}</span><button type="button" class="copy-button" aria-label="复制代码块 {i+1}">复制代码</button></div><pre tabindex="0"><code>{formatted}</code></pre></div>'

def notes(p,source,codes):
 count=sum(c['lang']=='rust' for c in codes)
 changed=sum(a['text']!=b['text'] for a,b in zip(codes,json.loads((S/(p['id']+'.codes.json')).read_text())))
 extra=f'本页 {count} 个 Rust 代码块均有编译检查记录；片段已补齐所需上下文。' if count else f'依赖与版本引用已同步到 {VERSION}。'
 if changed:extra+=f' {changed} 个代码块已更新。'
 if p['id']=='33':extra+=' 历史数据勘误：6 MB / 202 MB ≈ 2.97%。'
 if p['id']=='34':extra+=' 历史数据勘误：113.1 MB − 96.1 MB = 17.0 MB。'
 return f'<section class="notice" aria-label="版本与验证"><strong>Rig {VERSION} 实际适配版</strong><p>'+extra+'</p><a href="versions.html">查看逐块编译结果、运行前提与示例项目 →</a></section>'

def render(p):
 raw=(C/(p['id']+'.md')).read_text(); source=(S/(p['id']+'.md')).read_text(); codes=json.loads((C/'codes'/(p['id']+'.codes.json')).read_text())
 for pattern in [r'@@CODE_\d+@@',r'@@ANCHOR:[^@]+@@']:
  assert re.findall(pattern,raw)==re.findall(pattern,source),(p['id'],'placeholder mismatch')
 assert re.findall(r'^#+ ',raw,re.M)==re.findall(r'^#+ ',source,re.M),(p['id'],'heading count mismatch')
 body=raw.split('\n',1)[1]
 for i,c in enumerate(codes):
  rendered=code_html(c,i)
  check=validation.get('blocks',{}).get(p['id']+':'+str(i))
  if check:
   rendered+='<p class="page-meta"><a href="checks/'+E(check['file'])+'">完整代码与上下文 ↗</a><span>'+('编译通过' if check['status']=='passed' else '待验证')+f' · Rig {VERSION}</span></p>'
  body=body.replace(f'@@CODE_{i:03d}@@',rendered)
 body=re.sub(r'@@ANCHOR:([^@]+)@@',lambda m:'<div class="source-anchor" data-anchor="'+E(m[1],quote=True)+'"></div>',body)
 soup=BeautifulSoup(markdown.markdown(body,extensions=['tables','fenced_code','sane_lists']),'html.parser')
 for el in soup.select('[data-anchor]'):
  h=el.find_next_sibling()
  if h and h.name in ['h2','h3','h4','h5','h6']:
   h['id']=el['data-anchor'];el.decompose()
  else:el['id']=el['data-anchor'];del el['data-anchor']
 for a in soup.select('a[href]'):
  href=a['href']; parsed=urlparse(href)
  if (not parsed.netloc or parsed.netloc in ['www.rig.rs','rig.rs']) and parsed.path.startswith('/docs'):
   local=paths.get(parsed.path.rstrip('/'))
   a['href']=(local+('#'+parsed.fragment if parsed.fragment else '')) if local else urljoin('https://www.rig.rs',href)
  elif href.startswith('/'):
   a['href']='https://www.rig.rs'+href
 for img in soup.select('img[src]'):
  url=urljoin('https://www.rig.rs',img['src'])
  if url in images and images[url] is None:
   figure=soup.new_tag('figure',attrs={'class':'missing-image'});caption=soup.new_tag('figcaption');caption.string=img.get('alt','原文配图');figure.append(caption)
   notice=soup.new_tag('p');notice.string='原站图片链接当前返回 404，未能取得原图。';figure.append(notice)
   link=soup.new_tag('a',href=url);link.string='查看原始图片地址 ↗';figure.append(link);img.replace_with(figure);continue
  img['src']=images.get(url,url);img['loading']='lazy'
  if img['src'].startswith('assets/'):
   # PNG dimensions are in the IHDR header.
   data=(D/img['src']).read_bytes()
   if data[:8]==b'\x89PNG\r\n\x1a\n':
    import struct
    w,h=struct.unpack('>II',data[16:24]);img['width']=w;img['height']=h
  figure=soup.new_tag('figure');img.wrap(figure);caption=soup.new_tag('figcaption');caption.string=img.get('alt','原文配图')+'（保留原始截图）';figure.append(caption)
 for table in soup.select('table'):
  wrap=soup.new_tag('div',attrs={'class':'table-wrap','tabindex':'0','role':'region','aria-label':'数据表格，可横向滚动'});table.wrap(wrap)
 toc=[]
 for h in soup.select('h2[id],h3[id]'):
  title=h.get_text(' ',strip=True);toc.append(f'<a class="depth-{h.name[1]}" href="#{E(h["id"])}">{E(title)}</a>')
  a=soup.new_tag('a',href='#'+h['id'],attrs={'class':'heading-anchor','aria-label':'链接到'+title});a.string='#';h.append(a)
 i=manifest.index(p);pager='<nav class="pagination" aria-label="文档翻页">'
 if i>0:
  prev=manifest[i-1];pager+=f'<a href="{prev["file"]}"><small>← 上一篇</small>{E(prev["zh"])}</a>'
 else:pager+='<span></span>'
 if i<len(manifest)-1:
  nxt=manifest[i+1];pager+=f'<a class="next" href="{nxt["file"]}"><small>下一篇 →</small>{E(nxt["zh"])}</a>'
 pager+='</nav>'
 meta=f'<div class="page-meta"><span>{E(p["group"])}</span><span>官网快照 · {p["fetched_on"]}</span><a href="{p["url"]}">阅读英文原文 ↗</a></div>'
 (D/p['file']).write_text(shell(p['zh'],str(soup),p['id'],''.join(toc),meta,notes(p,source,codes),pager,description=description_from_markdown(p['zh'],raw,VERSION,p['id']=='00'),source_url=p['url']))
 # All code DOM text must match the checked 0.46.0 overlay.
 for actual,original in zip(soup.select('.code-block code'),codes):assert actual.get_text()==original['text'],(p['id'],'code altered')
 assert len(soup.select('.code-block'))==len(codes)
 search={'title':p['zh'],'english':p['title'],'group':p['group'],'href':p['file'],'text':soup.get_text(' ',strip=True)}
 return search

if D.exists():shutil.rmtree(D)
D.mkdir(exist_ok=True)
shutil.copytree(R/'assets',D/'assets',dirs_exist_ok=True)
(D/'.nojekyll').touch()
(D/'assets'/'favicon.svg').write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="7" fill="#984725"/><text x="16" y="23" text-anchor="middle" fill="#fffdf8" font-family="sans-serif" font-size="23" font-weight="bold">R</text></svg>')
if '--showcase' in sys.argv:
 example='<h2 id="reading">组件与阅读状态</h2><p>正文使用中文系统字体。保留 <code>AgentBuilder::new(model)</code> 等 API 名称。<a href="https://www.rig.rs/docs">原文链接</a> 有清晰的交互颜色。</p><div class="notice"><strong>译注 · 版本差异</strong><p>原文版本与当前 API 不一致时，在此明确说明。</p></div>'+code_html({'lang':'rust','text':'let agent = AgentBuilder::new(model).build();\nlet answer = agent.prompt("你好，Rig").await?;'},0)+'<div class="table-wrap"><table><thead><tr><th>接口</th><th>作用</th></tr></thead><tbody><tr><td>prompt</td><td>发送一次提问</td></tr></tbody></table></div>'
 (D/'showcase.html').write_text(shell('组件预览',example,'00','<a href="#reading">组件与阅读状态</a>',filename='showcase.html',noindex=True))
 print('Showcase built');sys.exit()
index=[render(p) for p in manifest]
(D/'assets'/'search-index.js').write_text('window.RIG_SEARCH='+json.dumps(index,ensure_ascii=False,separators=(',',':')).replace('<','\\u003c')+';\n')
if (R/'VERSIONS.md').exists():
 version_html=markdown.markdown((R/'VERSIONS.md').read_text().split('\n',1)[1],extensions=['tables','fenced_code'])
 (D/'versions.html').write_text(shell(f'{VERSION} 升级与验证',version_html))
shutil.copyfile(R/'LICENSE.upstream',D/'LICENSE.txt')
(R/'translation-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2))
print(f'Built {len(manifest)} translated pages; {sum(p["code_blocks"] for p in manifest)} versioned code blocks rendered')

# Ship the same checked source, locks and download links with every rebuild.
import zipfile
shutil.copytree(R/'examples',D/'checks/examples',dirs_exist_ok=True,ignore=shutil.ignore_patterns('target','__pycache__'))
for name in ['validation.json','code-changes.diff']:shutil.copyfile(R/name,D/name)
with zipfile.ZipFile(D/'examples.zip','w',zipfile.ZIP_DEFLATED) as archive:
 for f in (R/'examples').rglob('*'):
  if f.is_file() and 'target' not in f.parts:archive.write(f,str(f.relative_to(R)))
 archive.write(R/'validation.json','validation.json')
 archive.writestr('README.md',f'# Rig {VERSION} 示例\n\n所有项目均有 Cargo.lock。Rust {RUST_VERSION}。\n\n核心检查：`cargo check --locked --all-targets --manifest-path examples/core/Cargo.toml`\n\n离线测试：`cargo test --locked --manifest-path examples/core/Cargo.toml --test offline`\n\n具体运行前提见各项目 README；真实服务需要自己的凭证。\n')

# Preserve the initial public download URL while exposing a version-independent URL.
shutil.copyfile(D/'examples.zip',D/'examples-046.zip')

write_crawl_files(D,[p["file"] for p in manifest]+["versions.html"],config,os.environ.get("INDEXNOW_KEY",""))
