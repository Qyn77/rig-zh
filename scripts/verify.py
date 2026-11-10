from pathlib import Path
from bs4 import BeautifulSoup
from urllib.parse import urlparse,unquote
import json,re,collections,hashlib
from automation import validate_mapping
R=Path(__file__).resolve().parents[1]; D=R/'dist';manifest=json.loads((R/'translation-manifest.json').read_text())
validation=json.loads((R/'validation.json').read_text())
issues=[];counts=collections.Counter();trees={}
for p in D.glob('*.html'):trees[p.name]=BeautifulSoup(p.read_text(),'html.parser')
for row in manifest:
 ident=row['id'];file=row['file'];src=(R/'source'/f'{ident}.md').read_text();zh=(R/'content'/f'{ident}.md').read_text();soup=trees[file]
 for pattern,label in [(r'@@CODE_\d+@@','code placeholders'),(r'@@ANCHOR:[^@]+@@','anchors'),(r'^#+ ','headings')]:
  assert re.findall(pattern,src,re.M)==re.findall(pattern,zh,re.M),(ident,label)
 assert len(soup.select('main h1'))==1
 ids=[e['id'] for e in soup.select('[id]')];assert len(ids)==len(set(ids)),(file,'duplicate ids')
 codes=json.loads((R/'content'/'codes'/f'{ident}.codes.json').read_text());actual=soup.select('.code-block code')
 assert len(actual)==len(codes),(file,'code count')
 for a,b in zip(actual,codes):assert a.get_text()==b['text'],(file,'code bytes')
 for i,code in enumerate(codes):
  if code['lang']!='rust':continue
  checkrow=validation['blocks'][ident+':'+str(i)]
  assert checkrow['status']=='passed',(ident,i,'not compiled')
  context=(R/checkrow['file']).read_text()
  assert code['text'].strip() in context,(ident,i,'checked source differs')
  assert hashlib.sha256((R/checkrow['file']).read_bytes()).hexdigest()==checkrow['sha256'],(ident,i,'checked source changed')
 counts['pages']+=1;counts['code_blocks']+=len(codes);counts['missing_image_notices']+=len(soup.select('.missing-image'))
 assert hashlib.sha256((R/'source'/f'{ident}.html').read_bytes()).hexdigest()==row['sha256']
for file,soup in trees.items():
 if file=='showcase.html':continue
 for a in soup.select('a[href]'):
  u=urlparse(a['href'])
  if u.scheme or u.netloc:continue
  target=u.path or file
  if not (D/target).exists():issues.append([file,'missing file',a['href']]);continue
  if u.fragment and target.endswith('.html') and target in trees:
   if not trees[target].find(id=unquote(u.fragment)):issues.append([file,'missing anchor',a['href']])
 for e in soup.select('script[src],link[href],img[src]'):
  url=e.get('src') or e.get('href');u=urlparse(url)
  if not u.scheme and not u.netloc:assert (D/u.path).exists(),(file,url)
validate_mapping()
assert counts['pages']==len(manifest)
assert counts['code_blocks']==sum(p['code_blocks'] for p in manifest)
counts['compiled_rust_blocks']=len(validation['blocks'])
report={'counts':dict(counts),'local_link_issues':issues}
(R/'verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2))
print(json.dumps(report,ensure_ascii=False,indent=2))
if issues:raise SystemExit(1)

import tomllib
for p in (R/'examples').glob('*/Cargo.toml'):
 dep=tomllib.loads(p.read_text())['dependencies']['rig']
 assert (dep if isinstance(dep,str) else dep['version'])=='='+validation['rig_version'],p
 lock=tomllib.loads(p.with_name('Cargo.lock').read_text())
 assert next(x['version'] for x in lock['package'] if x['name']=='rig')==validation['rig_version'],p
for p in (R/'content/codes').glob('*.json'):
 for code in json.loads(p.read_text()):
  if code['lang']=='toml':
   parsed=tomllib.loads(code['text'])
   dep=parsed.get('dependencies',{}).get('rig')
   if dep is not None:assert (dep if isinstance(dep,str) else dep['version'])=='='+validation['rig_version'],(p,'displayed Cargo pin')
