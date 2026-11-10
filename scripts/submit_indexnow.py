"""Notify IndexNow after deployment. Receipt is not evidence of search indexing."""
import json
import os
from pathlib import Path
import re
import time
from urllib.parse import urlparse, unquote
import urllib.request
import urllib.error
from xml.etree import ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
ENDPOINT = 'https://api.indexnow.org/indexnow'


def payload_for(base, key, sitemap):
    parsed = urlparse(base)
    if parsed.scheme != 'https' or not parsed.hostname or parsed.username or parsed.query or parsed.fragment or not base.endswith('/'):
        raise ValueError('Invalid site base URL')
    if not re.fullmatch(r'[A-Za-z0-9-]{8,128}', key):
        raise ValueError('Invalid IndexNow key')
    urls = [node.text for node in ET.fromstring(sitemap).findall('.//{http://www.sitemaps.org/schemas/sitemap/0.9}loc')]
    if not urls or len(urls) > 10000 or len(urls) != len(set(urls)):
        raise ValueError('Sitemap URL count is invalid')
    for url in urls:
        if not url or not url.startswith(base) or urlparse(url).fragment or '..' in unquote(urlparse(url).path).split('/') or '\\' in unquote(url):
            raise ValueError('Sitemap URL escapes the verified project directory')
    return {'host': parsed.hostname, 'key': key, 'keyLocation': base + key + '.txt', 'urlList': urls}


def download(url):
    with urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'rig-zh-indexnow/1.0'}), timeout=45) as response:
        return response.read()


def main():
    key = os.environ.get('INDEXNOW_KEY', '')
    if not key:
        print('IndexNow skipped: repository secret INDEXNOW_KEY is not configured')
        return
    base = json.loads((ROOT / 'site.json').read_text())['base_url']
    # Wait for Pages CDN propagation; never post before ownership proof is reachable.
    for attempt in range(6):
        try:
            data = payload_for(base, key, download(base + 'sitemap.xml'))
            if download(data['keyLocation']).decode().strip() != key:
                raise ValueError('Online IndexNow ownership proof differs from configured key')
            break
        except (urllib.error.HTTPError, urllib.error.URLError):
            if attempt == 5: raise RuntimeError('Published sitemap/key is not reachable; IndexNow submission was not sent') from None
            time.sleep(10)
    request = urllib.request.Request(ENDPOINT, data=json.dumps(data).encode(), method='POST',
                                     headers={'Content-Type': 'application/json; charset=utf-8', 'User-Agent': 'rig-zh-indexnow/1.0'})
    try:
        with urllib.request.urlopen(request, timeout=60) as response: status = response.status
    except urllib.error.HTTPError as error:
        raise RuntimeError(f'IndexNow returned HTTP {error.code}; URLs were not confirmed accepted') from None
    if status not in (200, 202): raise RuntimeError(f'Unexpected IndexNow HTTP {status}')
    summary = f'IndexNow HTTP {status}: submitted {len(data["urlList"])} canonical URLs. '
    summary += 'Key validation is pending. ' if status == 202 else ''
    summary += 'This confirms receipt, not indexing or ranking. Google Search Console submission is separate.'
    print(summary)
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as output: output.write(summary + '\n')


if __name__ == '__main__':
    main()
