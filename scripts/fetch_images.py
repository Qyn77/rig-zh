from pathlib import Path
from bs4 import BeautifulSoup
import subprocess,json,hashlib
from urllib.parse import urljoin,urlparse
from concurrent.futures import ThreadPoolExecutor
root=Path(__file__).resolve().parents[1]; urls=set()
for f in (root/'source').glob('*.html'):
 s=BeautifulSoup(f.read_text(),'html.parser')
 for img in s.select('.sl-markdown-content img'):
  if img.get('src'):urls.add(urljoin('https://www.rig.rs',img['src']))
def get(url):
 name='assets/'+hashlib.sha256(url.encode()).hexdigest()[:12]+Path(urlparse(url).path).suffix
 dest=root/'dist'/name
 if dest.exists() and dest.stat().st_size>100: return url,name
 result=subprocess.run(['curl','-fsSL',url,'-o',str(dest)],capture_output=True)
 return url,name if result.returncode==0 else None
with ThreadPoolExecutor(max_workers=5) as ex: results=dict(ex.map(get,sorted(urls)))
(root/'source'/'images.json').write_text(json.dumps(results,indent=2))
print('Images available:',sum(v is not None for v in results.values()),'unavailable:',sum(v is None for v in results.values()))
