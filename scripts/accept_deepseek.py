"""Live credential acceptance: translate/migrate with DeepSeek, compile in a separate job."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from automation import ROOT, clean_env
from snapshot import validate_translation
from sync_deepseek import DeepSeek, Editor, SYSTEM, TOOLS

OUT = ROOT / '.acceptance'
ENGLISH = '''# Providers and conversation memory

@@ANCHOR:selection@@

## Selecting a model

A model reference identifies a provider and a model. Constructing an agent does not send a model request.

@@CODE_000@@

@@ANCHOR:memory@@

## Conversation memory

Use a separate conversation identifier for each user. Attaching a memory store enables history management; creating the store alone does not call a model.
'''


def validate_result(result):
    validate_translation(ENGLISH, result['markdown'])
    code = result['rust']
    for name in ('Catalog', 'MemoryStore', 'AgentBuilder'):
        if name not in code:
            raise ValueError('Migration omitted required capability: ' + name)
    if 'ProviderRef' in code or 'InMemoryConversationMemory' in code:
        raise ValueError('Migration retained an obsolete API')
    if not re.search(r'fn\s+main\s*\(', code):
        raise ValueError('Expected a standalone Rust program')


def prepare():
    client = DeepSeek()
    version = json.loads((ROOT / 'validation.json').read_text())['rig_version']
    # Real old official snippets, paired with already compiled API references.
    legacy = [json.loads((ROOT / f'source/{p}.codes.json').read_text())[i]['text']
              for p, i in [('05', 6), ('16', 0)]]
    references = [(ROOT / name).read_text() for name in
                  ['examples/migration/examples/p05_6.rs', 'examples/migration/examples/p16_0.rs']]
    result = client.json(SYSTEM + '''
这是实际 API 验收，不发布文档。返回 JSON 对象 markdown、rust、migration_notes。
markdown 为所给英文段落的完整简洁中文译文，保留全部标题、锚点和代码占位符。
rust 为迁移到目标 Rig 版本的完整可编译程序：使用 Catalog 连接 DeepSeek 的模型描述，创建 MemoryStore 并附到 AgentBuilder。
只构造这些对象，禁止 prompt/网络推理/读取或输出密钥。程序需能用占位凭证离线执行。
使用旧示例识别待迁移能力，使用已编译的参考代码核对真实 API。migration_notes 用中文说明实际替换的 API。''',
        json.dumps({'target_version': version, 'english': ENGLISH, 'old_snippets': legacy,
                    'verified_api_references': references}, ensure_ascii=False))
    validate_result(result)
    OUT.mkdir(exist_ok=True)
    (OUT / 'generated.rs').write_text(result['rust'])
    (OUT / 'translation.md').write_text(result['markdown'])

    # Exercise the production client's actual multi-turn tool-call protocol.
    # Scope this test to two disposable Markdown files, without compilation here.
    calls_seen = []
    with tempfile.TemporaryDirectory(prefix='rig-live-tools-') as directory:
        root = Path(directory)
        (root / 'content').mkdir()
        (root / 'content/acceptance.md').write_text(result['markdown'])
        editor = Editor(root)
        messages = [
            {'role': 'system', 'content': '这是工具调用协议验收。只执行用户明确指定的文件读写，不改写正文。'},
            {'role': 'user', 'content': '先调用 read 读取 content/acceptance.md 的全文（offset=1, limit=400），再调用 write 将相同全文写入 content/acceptance-reviewed.md。必须真实调用两个工具，完成后简短确认。'},
        ]
        for _ in range(5):
            message = client.chat(messages, tools=[t for t in TOOLS if t['function']['name'] in ('read', 'write')])
            messages.append({k: v for k, v in message.items() if k in ('role', 'content', 'tool_calls')})
            calls = message.get('tool_calls') or []
            if not calls: break
            for call in calls:
                name = call['function']['name']
                args = json.loads(call['function']['arguments'])
                expected = {'read': 'content/acceptance.md', 'write': 'content/acceptance-reviewed.md'}
                if name not in expected or args.get('path') != expected[name]:
                    raise ValueError('Acceptance tool call escaped its two-file scope')
                output = editor.execute(name, args)
                calls_seen.append(name)
                messages.append({'role': 'tool', 'tool_call_id': call['id'], 'content': output})
        if not {'read', 'write'} <= set(calls_seen):
            raise ValueError('DeepSeek did not perform the required tool round trip')
        if (root / 'content/acceptance-reviewed.md').read_text().strip() != result['markdown'].strip():
            raise ValueError('Tool round trip changed translation content')
    report = {'status': 'awaiting-independent-compilation', 'model': client.model,
              'rig_version': version, 'requests': client.calls, 'total_tokens': client.tokens,
              'tool_calls': calls_seen, 'migration_notes': result['migration_notes'],
              'rust_sha256': hashlib.sha256((OUT / 'generated.rs').read_bytes()).hexdigest(),
              'scope': 'Live DeepSeek JSON translation, representative obsolete API migration, tool round trip; not a full future-release migration'}
    (OUT / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps(report, ensure_ascii=False, indent=2))


def compile_and_run():
    report = json.loads((OUT / 'report.json').read_text())
    generated = (OUT / 'generated.rs').read_bytes()
    if hashlib.sha256(generated).hexdigest() != report['rust_sha256']:
        raise ValueError('Generated code differs from the live model result')
    version = json.loads((ROOT / 'validation.json').read_text())
    if version['rig_version'] != report['rig_version']:
        raise ValueError('Acceptance was generated against a different version')
    target = ROOT / 'examples/migration/examples/credential_acceptance.rs'
    if target.exists(): raise ValueError('Refusing to overwrite an existing example')
    target.write_bytes(generated)
    env = clean_env()
    env['DEEPSEEK_API_KEY'] = 'offline-placeholder'
    env.setdefault('CARGO_TARGET_DIR', str(ROOT / 'examples-target'))
    command = ['cargo', '+' + version['rustc']]
    options = ['--locked', '--manifest-path', str(ROOT / 'examples/migration/Cargo.toml'),
               '--example', 'credential_acceptance']
    try:
        result = subprocess.run(command + ['build'] + options, env=env, cwd=ROOT,
                                text=True, capture_output=True, timeout=1800)
        (OUT / 'compile.log').write_text(result.stdout + result.stderr)
        if result.returncode:
            print(result.stderr[-16000:])
            raise RuntimeError('The real DeepSeek-generated example did not compile')
        # Execute the built binary directly, so timeout covers runtime rather than compilation.
        binary = Path(env['CARGO_TARGET_DIR']) / 'debug/examples/credential_acceptance'
        with tempfile.TemporaryDirectory(prefix='rig-live-run-') as directory:
            result = subprocess.run([str(binary)], env=env, cwd=directory,
                                    text=True, capture_output=True, timeout=30)
        (OUT / 'runtime.log').write_text(result.stdout + result.stderr)
        if result.returncode: raise RuntimeError('The generated example failed offline execution')
    finally:
        target.unlink(missing_ok=True)
    report['status'] = 'passed'
    report['compile'] = 'cargo build --locked; pinned Rust/Rig'
    report['runtime'] = 'passed with offline placeholder credential; no live provider inference tested'
    (OUT / 'report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as summary:
            summary.write('DeepSeek 实际验收通过：中文翻译、旧 API 迁移、工具读写回合、生成代码编译和离线运行。\n\n')
            summary.write(f"模型：{report['model']}；请求 {report['requests']} 次；token {report['total_tokens']}。\n")
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('phase', choices=['prepare', 'compile'])
    args = parser.parse_args()
    (prepare if args.phase == 'prepare' else compile_and_run)()
