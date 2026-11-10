"""Build crawlable metadata from the current translated content, not frozen page counts."""
from html import escape
import json
import os
from pathlib import Path
import re
from urllib.parse import urlparse
from xml.etree import ElementTree as ET
from bs4 import BeautifulSoup
import markdown

ROOT = Path(__file__).resolve().parents[1]
SITEMAP_NS = 'http://www.sitemaps.org/schemas/sitemap/0.9'


def load_config(root=ROOT):
    config = json.loads((root / 'site.json').read_text())
    parsed = urlparse(config['base_url'])
    if parsed.scheme != 'https' or not parsed.hostname or parsed.username or parsed.password or parsed.query or parsed.fragment or not parsed.path.endswith('/'):
        raise ValueError('base_url must be an absolute HTTPS directory URL without credentials/query/fragment')
    return config


def canonical_url(config, filename):
    if not re.fullmatch(r'[A-Za-z0-9_-]+\.html', filename):
        raise ValueError('Expected a flat generated HTML filename')
    return config['base_url'] + ('' if filename == 'index.html' else filename)


def description_from_markdown(title, text, version, home=False):
    if home:
        return f'Rig Rust 中文文档与教程，涵盖 Agent、工具调用、RAG、流式输出、记忆和 MCP。示例适配 Rig {version}，附完整源码、编译检查与离线测试记录。非官方中文适配版。'
    text = re.sub(r'@@(?:CODE_\d+|ANCHOR:[^@]+)@@', '', text)
    soup = BeautifulSoup(markdown.markdown(text, extensions=['tables', 'fenced_code']), 'html.parser')
    for node in soup.select('pre, table, script, style, blockquote, h1, h2, h3, h4, h5, h6'):
        node.decompose()
    paragraphs = [re.sub(r'\s+', ' ', p.get_text(' ', strip=True)) for p in soup.select('p')]
    paragraphs = [p for p in paragraphs if len(p) >= 15 and not p.startswith('@@')]
    prose = ' '.join(paragraphs[:2]) or f'了解 {title} 的使用方式、示例与运行前提。'
    result = f'{title}：{prose}'
    if len(result) > 160:
        result = result[:159].rstrip(' ，,；;：:') + '…'
    return result


def metadata(config, title, filename, description, source_url=None, noindex=False):
    url = canonical_url(config, filename)
    home = filename == 'index.html'
    document_title = 'Rig Rust 中文文档与教程｜Agent、RAG 与 MCP' if home else f'{title}｜Rig Rust 中文文档'
    website_id = config['base_url'] + '#website'
    page_id = url + '#webpage'
    website = {'@type': 'WebSite', '@id': website_id, 'url': config['base_url'],
               'name': config['name'], 'inLanguage': config['language']}
    page = {'@type': 'WebPage', '@id': page_id, 'url': url, 'name': document_title,
            'description': description, 'inLanguage': config['language'], 'isPartOf': {'@id': website_id}}
    graph = [website, page]
    if source_url:
        article = {'@type': 'TechArticle', '@id': url + '#article', 'headline': title,
                   'description': description, 'inLanguage': config['language'], 'url': url,
                   'mainEntityOfPage': {'@id': page_id}, 'isBasedOn': source_url,
                   'isPartOf': {'@id': website_id}}
        page['mainEntity'] = {'@id': article['@id']}
        graph.append(article)
    if not home and not noindex:
        breadcrumb = {'@type': 'BreadcrumbList', '@id': url + '#breadcrumb', 'itemListElement': [
            {'@type': 'ListItem', 'position': 1, 'name': config['name'], 'item': config['base_url']},
            {'@type': 'ListItem', 'position': 2, 'name': title, 'item': url},
        ]}
        page['breadcrumb'] = {'@id': breadcrumb['@id']}
        graph.append(breadcrumb)
    values = [('description', description), ('robots', 'noindex,follow' if noindex else 'index,follow,max-image-preview:large'),
              ('twitter:card', 'summary'), ('twitter:title', document_title), ('twitter:description', description)]
    tags = [f'<title>{escape(document_title)}</title>', f'<link rel="canonical" href="{escape(url, quote=True)}">',
            f'<link rel="sitemap" type="application/xml" href="{escape(config["base_url"] + "sitemap.xml", quote=True)}">']
    tags += [f'<meta name="{name}" content="{escape(value, quote=True)}">' for name, value in values]
    og = {'og:type': 'article' if source_url and not home else 'website', 'og:title': document_title,
          'og:description': description, 'og:url': url, 'og:site_name': config['name'], 'og:locale': 'zh_CN'}
    tags += [f'<meta property="{name}" content="{escape(value, quote=True)}">' for name, value in og.items()]
    for key, name in [('google_site_verification', 'google-site-verification'), ('bing_site_verification', 'msvalidate.01')]:
        if config.get(key): tags.append(f'<meta name="{name}" content="{escape(config[key], quote=True)}">')
    # Escape '<' so document text cannot close the JSON-LD script element.
    encoded = json.dumps({'@context': 'https://schema.org', '@graph': graph}, ensure_ascii=False).replace('<', '\\u003c')
    if not noindex: tags.append(f'<script type="application/ld+json">{encoded}</script>')
    return ''.join(tags)


def write_crawl_files(destination, filenames, config, indexnow_key=''):
    ET.register_namespace('', SITEMAP_NS)
    root = ET.Element(f'{{{SITEMAP_NS}}}urlset')
    for filename in filenames:
        entry = ET.SubElement(root, f'{{{SITEMAP_NS}}}url')
        ET.SubElement(entry, f'{{{SITEMAP_NS}}}loc').text = canonical_url(config, filename)
    ET.ElementTree(root).write(destination / 'sitemap.xml', encoding='utf-8', xml_declaration=True)
    # Build/fetch dates are not genuine content modification dates: omit lastmod.
    if urlparse(config['base_url']).path == '/':
        (destination / 'robots.txt').write_text('User-agent: *\nAllow: /\nSitemap: ' + config['base_url'] + 'sitemap.xml\n')
    if indexnow_key:
        if not re.fullmatch(r'[A-Za-z0-9-]{8,128}', indexnow_key):
            raise ValueError('Invalid IndexNow verification key')
        (destination / (indexnow_key + '.txt')).write_text(indexnow_key)


def verify_site(destination, manifest, config):
    filenames = [p['file'] for p in manifest] + ['versions.html']
    expected = {canonical_url(config, file) for file in filenames}
    xml = ET.parse(destination / 'sitemap.xml')
    locations = [e.text for e in xml.findall(f'.//{{{SITEMAP_NS}}}loc')]
    assert set(locations) == expected and len(locations) == len(expected), 'Sitemap coverage or duplicate URL error'
    titles, descriptions = set(), set()
    for file in filenames:
        soup = BeautifulSoup((destination / file).read_text(), 'html.parser')
        canonical = canonical_url(config, file)
        assert [e['href'] for e in soup.select('link[rel="canonical"]')] == [canonical], file
        assert soup.select_one('meta[property="og:url"]')['content'] == canonical, file
        assert 'noindex' not in soup.select_one('meta[name="robots"]')['content'], file
        title = soup.title.get_text()
        description = soup.select_one('meta[name="description"]')['content']
        assert title not in titles and description not in descriptions, (file, 'duplicate title/description')
        assert 30 <= len(description) <= 180 and '@@' not in description, (file, 'invalid description')
        titles.add(title); descriptions.add(description)
        data = json.loads(soup.select_one('script[type="application/ld+json"]').string)
        assert data['@context'] == 'https://schema.org', file
        page = next(n for n in data['@graph'] if n['@type'] == 'WebPage')
        assert page['url'] == canonical and page['inLanguage'] == 'zh-CN', file
        if file != 'index.html':
            crumbs = next(n for n in data['@graph'] if n['@type'] == 'BreadcrumbList')['itemListElement']
            assert [x['position'] for x in crumbs] == [1, 2], file
            assert crumbs[0]['item'] == config['base_url'] and crumbs[1]['item'] == canonical, file
        if file != 'versions.html':
            article = next(n for n in data['@graph'] if n['@type'] == 'TechArticle')
            assert article['isBasedOn'].startswith('https://www.rig.rs/docs'), file
    if urlparse(config['base_url']).path != '/':
        assert not (destination / 'robots.txt').exists(), 'Project-subdirectory robots.txt would not apply'
    print(f'PASS: SEO metadata, structured data and sitemap cover {len(filenames)} pages')
