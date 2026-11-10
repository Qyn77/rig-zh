import json
from pathlib import Path
import sys
import tempfile
import unittest
from bs4 import BeautifulSoup

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from seo import canonical_url, description_from_markdown, metadata, write_crawl_files
from submit_indexnow import payload_for

CONFIG = {'base_url': 'https://example.com/rig-zh/', 'name': 'Rig 中文文档', 'language': 'zh-CN'}


class SeoTests(unittest.TestCase):
    def test_home_canonical_consolidates_index_alias_and_preserves_project_prefix(self):
        self.assertEqual(canonical_url(CONFIG, 'index.html'), CONFIG['base_url'])
        self.assertEqual(canonical_url(CONFIG, '02.html'), CONFIG['base_url'] + '02.html')
        with self.assertRaises(ValueError): canonical_url(CONFIG, '../other.html')

    def test_descriptions_use_prose_not_code_or_internal_placeholders(self):
        text = '# 工具\n\n@@ANCHOR:tool@@\n\n工具让 Agent 调用应用中的 Rust 功能，参数通过类型进行约束。\n\n@@CODE_000@@\n\n```rust\nSECRET_CODE_SAMPLE();\n```'
        result = description_from_markdown('工具', text, '0.46.0')
        self.assertIn('参数通过类型', result)
        self.assertNotIn('@@', result)
        self.assertNotIn('SECRET_CODE_SAMPLE', result)

    def test_metadata_escapes_html_and_jsonld_without_faking_official_identity(self):
        title = '测试 </script><script>alert(1)</script>'
        html = metadata(CONFIG, title, '02.html', '安全的中文描述', 'https://www.rig.rs/docs/quickstart')
        soup = BeautifulSoup('<head>' + html + '</head>', 'html.parser')
        self.assertEqual(len(soup.select('script')), 1)
        graph = json.loads(soup.select_one('script').string)['@graph']
        article = next(n for n in graph if n['@type'] == 'TechArticle')
        self.assertEqual(article['headline'], title)
        self.assertNotIn('publisher', article)
        self.assertNotIn('dateModified', article)
        crumbs = next(n for n in graph if n['@type'] == 'BreadcrumbList')['itemListElement']
        self.assertEqual([c['position'] for c in crumbs], [1, 2])

    def test_preview_is_noindex_and_has_no_rich_result_schema(self):
        soup = BeautifulSoup(metadata(CONFIG, '预览', 'showcase.html', '预览内容', noindex=True), 'html.parser')
        self.assertIn('noindex', soup.select_one('meta[name="robots"]')['content'])
        self.assertFalse(soup.select('script[type="application/ld+json"]'))

    def test_sitemap_updates_for_new_pages_and_excludes_preview(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            write_crawl_files(root, ['index.html', '52.html', 'versions.html'], CONFIG, 'test-key-1234')
            xml = (root / 'sitemap.xml').read_bytes()
            payload = payload_for(CONFIG['base_url'], 'test-key-1234', xml)
            self.assertEqual(len(payload['urlList']), 3)
            self.assertEqual(payload['keyLocation'], CONFIG['base_url'] + 'test-key-1234.txt')
            self.assertEqual((root / 'test-key-1234.txt').read_text(), 'test-key-1234')
            self.assertNotIn(b'lastmod', xml)
            self.assertFalse((root / 'robots.txt').exists())

    def test_robots_is_only_written_at_host_root(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = {**CONFIG, 'base_url': 'https://example.com/'}
            write_crawl_files(root, ['index.html'], config)
            self.assertIn('Sitemap: https://example.com/sitemap.xml', (root / 'robots.txt').read_text())

    def test_indexnow_rejects_other_projects_and_encoded_traversal(self):
        for url in ['https://example.com/other/', 'https://example.com/rig-zh-evil/a.html', 'https://example.com/rig-zh/%2e%2e/private']:
            xml = f'<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><url><loc>{url}</loc></url></urlset>'
            with self.assertRaises(ValueError): payload_for(CONFIG['base_url'], 'test-key-1234', xml)


if __name__ == '__main__':
    unittest.main()
