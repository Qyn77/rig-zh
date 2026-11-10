import sys,json,subprocess,hashlib,re
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
from bs4 import BeautifulSoup
from markdownify import markdownify
root=Path(__file__).resolve().parents[1]
subprocess.run(['curl','-fsSL','https://www.rig.rs/docs','-o',str(root/'source'/'00.html')],check=True)
home=BeautifulSoup((root/'source'/'00.html').read_text(),'html.parser')
items=[]
for a in home.select('nav.sidebar a'):
 path=a.get('href','').rstrip('/')
 if not path.startswith('/docs') or any(x['path']==path for x in items):continue
 group='入门'
 if '/concepts' in path: group='核心概念'
 elif '/guides' in path: group='实战指南'
 elif '/integrations' in path: group='生态集成'
 elif path in ['/docs/examples','/docs/faq','/docs/troubleshooting','/docs/how_to_contribute']:group='参考'
 items.append({'id':f'{len(items):02d}','path':path,'title':a.get_text(' ',strip=True),'group':group,'url':'https://www.rig.rs'+path})
def fetch(item):
 rawfile=root/'source'/f'{item["id"]}.html'
 subprocess.run(['curl','--retry','2','-fsSL',item['url'],'-o',str(rawfile)],check=True)
 raw=rawfile.read_text(); soup=BeautifulSoup(raw,'html.parser'); article=soup.select_one('.sl-markdown-content')
 assert article,item
 item['title']=soup.select_one('h1').get_text(' ',strip=True)
 codes=[]
 for pre in article.select('pre'):
  code=pre.select_one('code') or pre
  # Expressive Code stores each logical line in div.ec-line.
  lines=code.select('.ec-line')
  text='\n'.join(x.get_text() for x in lines) if lines else code.get_text()
  codes.append({'lang':pre.get('data-language') or (pre.find_parent().get('data-language') if pre.find_parent() else '') or 'text','text':text})
  pre.replace_with(soup.new_string('\n\n@@CODE_%03d@@\n\n'%(len(codes)-1)))
 for el in article.select('script,style,svg,button,.copy,.sr-only,.sl-anchor-link,.header'):
  el.decompose()
 for h in article.select('h2[id],h3[id],h4[id]'):
  anchor=soup.new_tag('p');anchor.string='@@ANCHOR:'+h['id']+'@@';h.insert_before(anchor)
 md=markdownify(str(article),heading_style='ATX',bullets='-').strip()
 md=re.sub(r'\n{3,}','\n\n',md)
 (root/'source'/f'{item["id"]}.md').write_text('# '+item['title']+'\n\n'+md+'\n')
 (root/'source'/f'{item["id"]}.codes.json').write_text(json.dumps(codes,ensure_ascii=False,indent=2))
 item.update(sha256=hashlib.sha256(raw.encode()).hexdigest(),words=len(md.split()),code_blocks=len(codes),fetched_on='2026-11-10')
 return item
with ThreadPoolExecutor(max_workers=6) as pool: items=list(pool.map(fetch,items))
(root/'source'/'manifest.json').write_text(json.dumps(items,ensure_ascii=False,indent=2))
print('Pages:',len(items),'Prose words:',sum(x['words'] for x in items),'Code blocks:',sum(x['code_blocks'] for x in items))
for i in items:print(i['id'],i['group'],i['title'],i['words'])
