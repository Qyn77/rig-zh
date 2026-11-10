"""Fetch the official documentation with stable local page IDs."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import urllib.request
from bs4 import BeautifulSoup
from markdownify import markdownify
from automation import ROOT


def fetch(path):
    if not re.fullmatch(r'/docs(?:/[A-Za-z0-9_-]+)*', path):
        raise ValueError('Unexpected official documentation path')
    req = urllib.request.Request('https://www.rig.rs' + path,
                                 headers={'User-Agent': 'Qyn77/rig-zh documentation updater'})
    with urllib.request.urlopen(req, timeout=90) as response:
        return response.read().decode()


def extract(raw):
    soup = BeautifulSoup(raw, 'html.parser')
    article = soup.select_one('.sl-markdown-content')
    if article is None or soup.select_one('h1') is None:
        raise ValueError('Official site structure changed; refusing incomplete snapshot')
    title = soup.select_one('h1').get_text(' ', strip=True)
    codes = []
    for pre in article.select('pre'):
        code = pre.select_one('code') or pre
        lines = code.select('.ec-line')
        text = '\n'.join(x.get_text() for x in lines) if lines else code.get_text()
        codes.append({'lang': pre.get('data-language') or pre.parent.get('data-language') or 'text', 'text': text})
        pre.replace_with(soup.new_string(f'\n\n@@CODE_{len(codes)-1:03d}@@\n\n'))
    for el in article.select('script,style,svg,button,.copy,.sr-only,.sl-anchor-link,.header'):
        el.decompose()
    for heading in article.select('h2[id],h3[id],h4[id]'):
        anchor = soup.new_tag('p')
        anchor.string = '@@ANCHOR:' + heading['id'] + '@@'
        heading.insert_before(anchor)
    md = markdownify(str(article), heading_style='ATX', bullets='-').strip()
    # markdownify escapes underscores in prose, including our machine markers.
    md = re.sub(r'@@[^@]+@@', lambda match: match[0].replace('\\_', '_'), md)
    return title, '# ' + title + '\n\n' + re.sub(r'\n{3,}', '\n\n', md) + '\n', codes


def snapshot(root=ROOT):
    previous = json.loads((root / 'source/manifest.json').read_text())
    known = {p['path']: p['id'] for p in previous}
    next_id = max(int(p['id']) for p in previous) + 1
    home = fetch('/docs')
    paths = list(dict.fromkeys(a.get('href', '').rstrip('/') for a in
                 BeautifulSoup(home, 'html.parser').select('nav.sidebar a')
                 if a.get('href', '').startswith('/docs')))
    if '/docs' not in paths or len(paths) < len(previous) * .75:
        raise ValueError('Official sidebar unexpectedly empty or shrank by over 25%; manual review required')
    pages = []
    for path in paths:
        if path not in known:
            known[path] = f'{next_id:02d}'
            next_id += 1
        group = '入门'
        if '/concepts' in path: group = '核心概念'
        elif '/guides' in path: group = '实战指南'
        elif '/integrations' in path: group = '生态集成'
        elif path in ['/docs/examples', '/docs/faq', '/docs/troubleshooting', '/docs/how_to_contribute']: group = '参考'
        pages.append({'id': known[path], 'path': path, 'url': 'https://www.rig.rs' + path, 'group': group})
    def capture(page):
        raw = home if page['path'] == '/docs' else fetch(page['path'])
        title, md, codes = extract(raw)
        for suffix, text in [('html', raw), ('md', md), ('codes.json', json.dumps(codes, ensure_ascii=False, indent=2))]:
            (root / 'source' / (page['id'] + '.' + suffix)).write_text(text)
        page.update(title=title, sha256=hashlib.sha256(raw.encode()).hexdigest(),
                    words=len(md.split()), code_blocks=len(codes),
                    fetched_on=datetime.now(timezone.utc).date().isoformat())
        return page
    with ThreadPoolExecutor(max_workers=4) as pool:
        pages = list(pool.map(capture, pages))
    (root / 'source/manifest.json').write_text(json.dumps(pages, ensure_ascii=False, indent=2))
    active = {p['id'] for p in pages}
    for old in previous:
        if old['id'] not in active:
            for suffix in ('html', 'md', 'codes.json'):
                (root / 'source' / (old['id'] + '.' + suffix)).unlink(missing_ok=True)
            (root / 'content' / (old['id'] + '.md')).unlink(missing_ok=True)
            (root / 'content/codes' / (old['id'] + '.codes.json')).unlink(missing_ok=True)
    return pages


def validate_translation(source, translated):
    for pattern in (r'@@CODE_\d+@@', r'@@ANCHOR:[^@]+@@', r'^#+ '):
        if re.findall(pattern, source, re.M) != re.findall(pattern, translated, re.M):
            raise ValueError('Translation lost/reordered a heading, anchor, or code placeholder')
    if not re.search(r'[\u4e00-\u9fff]', translated):
        raise ValueError('Translation contains no Chinese text')
