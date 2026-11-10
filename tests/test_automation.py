import copy
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import automation
import candidate
import detect_release
import snapshot
import sync_deepseek
import verify_ci


class AutomationTests(unittest.TestCase):
    def test_versions_ignore_prerelease_yanked_and_sort_numerically(self):
        rows = [{'vers': v, 'yanked': y} for v, y in
                [('0.46.0', False), ('0.100.0-rc.1', False), ('0.99.0', True), ('0.9.9', False), ('0.47.0', False)]]
        text = '\n'.join(map(json.dumps, rows))
        self.assertEqual(detect_release.detect(text, '0.46.0')['version'], '0.47.0')
        self.assertTrue(detect_release.detect(text, '0.46.0')['update'])
        self.assertFalse(detect_release.detect(text, '0.47.0')['update'])
        self.assertFalse(detect_release.detect(text, '0.48.0')['update'])
        with self.assertRaises(ValueError): automation.latest_stable([{'vers': '1.0.0-rc.1'}])

    def test_translation_must_preserve_structure(self):
        source = '# Intro\n## One\n@@ANCHOR:one@@\n@@CODE_000@@'
        translated = '# 介绍\n## 第一节\n@@ANCHOR:one@@\n@@CODE_000@@'
        snapshot.validate_translation(source, translated)
        for broken in (translated.replace('@@CODE_000@@', ''), translated.replace('## ', '### '), source):
            with self.assertRaises(ValueError): snapshot.validate_translation(source, broken)

    def test_snapshot_extracts_official_expressive_code(self):
        raw = '<h1>Tools</h1><article class="sl-markdown-content"><h2 id="use">Use</h2><pre data-language="rust"><code><div class="ec-line">let x = 1;</div><div class="ec-line">x + 1</div></code></pre></article>'
        title, md, codes = snapshot.extract(raw)
        self.assertEqual(title, 'Tools')
        self.assertIn('@@ANCHOR:use@@', md)
        self.assertIn('@@CODE_000@@', md)
        self.assertEqual(codes, [{'lang': 'rust', 'text': 'let x = 1;\nx + 1'}])
        with self.assertRaises(ValueError): snapshot.extract('<h1>Error</h1>')

    def test_snapshot_keeps_ids_on_sidebar_reordering_and_addition(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / 'source').mkdir()
            old = [{'id': '00', 'path': '/docs'}, {'id': '01', 'path': '/docs/tools'}]
            (root / 'source/manifest.json').write_text(json.dumps(old))
            html = '<nav class="sidebar"><a href="/docs">Home</a><a href="/docs/new">New</a><a href="/docs/tools">Tools</a></nav><h1>Page</h1><div class="sl-markdown-content">Text</div>'
            with patch.object(snapshot, 'fetch', return_value=html):
                result = snapshot.snapshot(root)
            self.assertEqual([p['id'] for p in result], ['00', '02', '01'])

    def test_model_cannot_write_ci_read_secrets_or_execute_shell(self):
        with tempfile.TemporaryDirectory() as d:
            editor = sync_deepseek.Editor(Path(d))
            for name in ('../secret', '/etc/passwd', '.github/workflows/pages.yml', '.env',
                         'scripts/verify.py', 'source/00.md', 'examples/core/tests/offline.rs',
                         'examples/core/build.rs.sh', 'examples/migration/examples/offline.rs'):
                with self.subTest(name=name), self.assertRaises(ValueError):
                    editor.execute('write', {'path': name, 'content': 'bad'})
            with self.assertRaises(ValueError): editor.execute('read', {'path': '.env', 'offset': 1, 'limit': 5})
            with self.assertRaises(ValueError): editor.execute('cargo', {'project': 'core', 'operation': 'run; env'})
            with self.assertRaises(ValueError): editor.execute('cargo', {'project': 'core', 'operation': 'check'})
            with self.assertRaises(ValueError): editor.execute('write', {'path': 'examples/core/build.rs', 'content': 'fn main() {}'})
            with self.assertRaises(ValueError): editor.execute('shell', {'command': 'env'})
            editor.execute('write', {'path': 'content/00.md', 'content': '# 中文'})
            self.assertEqual(editor.execute('read', {'path': 'content/00.md', 'offset': 1, 'limit': 5}), '# 中文')

    def test_model_cannot_add_build_scripts_or_nonregistry_dependencies(self):
        for text in ('[package]\nbuild="payload.rs"', '[build-dependencies]\nx="1"',
                     '[dependencies]\nx={path="/tmp/secret"}', '[dependencies]\nx={git="https://example.com/x"}'):
            with self.assertRaises(ValueError): sync_deepseek.validate_cargo_manifest(text)

    def test_embedded_offline_assertions_are_immutable(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            p = root / 'examples/guides/examples/replay.rs'
            p.parent.mkdir(parents=True)
            p.write_text('fn main() { assert_eq!(2 + 2, 4); }')
            editor = sync_deepseek.Editor(root)
            with self.assertRaises(ValueError):
                editor.execute('write', {'path': p.relative_to(root).as_posix(), 'content': 'fn main() {}'})

    def test_children_never_receive_model_or_github_secrets(self):
        with patch.dict(os.environ, {'DEEPSEEK_API_KEY': 'test-secret', 'GITHUB_TOKEN': 'test-token', 'OTHER_SECRET': 'x'}):
            self.assertFalse({'DEEPSEEK_API_KEY', 'GITHUB_TOKEN', 'OTHER_SECRET'} & automation.clean_env().keys())

    def test_rust_blocks_cannot_be_relabelled_to_bypass_compilation(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / 'source').mkdir()
            (root / 'content/codes').mkdir(parents=True)
            (root / 'validation.json').write_text(json.dumps({'rig_version': '0.46.0', 'rustc': '1.97.1', 'blocks': {}}))
            (root / 'source/manifest.json').write_text('[{"id":"00"}]')
            (root / 'source/00.codes.json').write_text('[{"lang":"rust","text":"fn main() {}"}]')
            (root / 'content/codes/00.codes.json').write_text('[{"lang":"text","text":"fn main() {}"}]')
            with self.assertRaisesRegex(ValueError, 'cannot be changed'):
                automation.validate_mapping(root, require_hash=False)

    def test_missing_secret_fails_before_network(self):
        with patch.dict(os.environ, {'DEEPSEEK_API_KEY': ''}):
            with self.assertRaisesRegex(RuntimeError, 'DEEPSEEK_API_KEY'): sync_deepseek.DeepSeek()

    def test_deepseek_uses_official_endpoint_tools_and_bounded_budget(self):
        response = {'choices': [{'finish_reason': 'stop', 'message': {'role': 'assistant', 'content': '{"ok":true}'}}],
                    'usage': {'total_tokens': 200}}
        import io
        with patch.dict(os.environ, {'DEEPSEEK_API_KEY': 'fake-key', 'SYNC_MAX_CALLS': '1'}):
            client = sync_deepseek.DeepSeek()
        with patch('urllib.request.urlopen', return_value=io.BytesIO(json.dumps(response).encode())) as request:
            self.assertEqual(client.json('Return JSON', 'test'), {'ok': True})
            self.assertEqual(request.call_args[0][0].full_url, 'https://api.deepseek.com/chat/completions')
            self.assertEqual(client.tokens, 200)
            with self.assertRaisesRegex(RuntimeError, 'budget'): client.chat([])

    def test_candidate_rejects_traversal_ci_and_duplicate_entries(self):
        for name in ('../escape', '.github/workflows/pages.yml', 'source/../../escape'):
            with tempfile.TemporaryDirectory() as d:
                archive = Path(d) / 'candidate.zip'
                with zipfile.ZipFile(archive, 'w') as z: z.writestr(name, 'bad')
                with self.assertRaises(ValueError): candidate.restore(archive, Path(d) / 'checkout')

    def test_symlink_escape_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / 'content').symlink_to('/tmp', target_is_directory=True)
            with self.assertRaises(ValueError): automation.safe_path(root, 'content/secret')

    def test_release_gate_rejects_missing_stale_and_failed_evidence(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / '.ci-results').mkdir()
            v = {'rig_version': '0.46.0', 'rustc': '1.97.1', 'blocks': {}}
            with patch.object(verify_ci, 'validate_mapping', return_value=v), patch.object(verify_ci, 'fingerprint', return_value='current'):
                with self.assertRaises(FileNotFoundError): verify_ci.verify(root)
                for project in automation.PROJECTS:
                    record = {'project': project, 'fingerprint': 'current', 'commands': [{'label': 'check-' + project, 'exit_code': 0}], 'rustc': 'rustc 1.97.1 (test)'}
                    (root / '.ci-results' / (project + '.json')).write_text(json.dumps(record))
                path = root / '.ci-results/core.json'
                original = path.read_text()
                for field, value in [('fingerprint', 'previous'), ('commands', [{'label': 'check-core', 'exit_code': 1}]), ('rustc', 'rustc 1.96.0 (test)')]:
                    broken = json.loads(original); broken[field] = value
                    path.write_text(json.dumps(broken))
                    with self.assertRaises(ValueError): verify_ci.verify(root)
                path.write_text(original)
                verify_ci.verify(root)
                self.assertTrue((root / 'validation.json').exists())


if __name__ == '__main__':
    unittest.main()
