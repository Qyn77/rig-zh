"""Compile shipped examples and execute only explicit offline regressions."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from automation import ROOT as R, PROJECTS, clean_env, fingerprint, validate_mapping


def check(project, evidence=False):
    v = validate_mapping(require_hash=False)
    before = fingerprint()
    env = clean_env()
    env.setdefault('CARGO_TARGET_DIR', str(R / 'examples-target'))
    logs = R / 'validation-logs'
    if evidence:
        shutil.rmtree(logs, ignore_errors=True)
    logs.mkdir(exist_ok=True)
    commands = []

    def cargo(*args):
        return ['cargo', '+' + v['rustc'], *args, '--locked', '--manifest-path', str(R / 'examples' / project / 'Cargo.toml')]

    with tempfile.TemporaryDirectory(prefix='rig-docs-offline-') as work:
        def run(label, cmd, stdin=None, override=None):
            print(label, flush=True)
            result = subprocess.run(cmd, cwd=work if 'run' in cmd else R,
                                    env={**env, **(override or {})}, input=stdin,
                                    text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    timeout=7200)
            output = result.stdout.replace(str(Path.home()), '<home>').replace(str(R), '<project>').replace(work, '<temporary>')
            (logs / (label + '.log')).write_text(output)
            commands.append({'label': label, 'exit_code': result.returncode})
            if result.returncode:
                print(output[-12000:], flush=True)
                raise RuntimeError(f'{label} failed; see validation-logs/{label}.log')
        if project == 'integrations-lance' and not (env.get('PROTOC') or shutil.which('protoc')):
            raise RuntimeError('LanceDB 编译需要 protoc；请安装 protobuf-compiler。')
        # Mapped files must be Cargo examples or directly declared modules of them.
        metadata = subprocess.check_output(cargo('metadata', '--format-version', '1', '--no-deps'), env=env, text=True)
        targets = {Path(t['src_path']).resolve() for p in json.loads(metadata)['packages'] for t in p['targets'] if 'example' in t['kind']}
        for row in v['blocks'].values():
            path = (R / row['file']).resolve()
            if row['file'].startswith(f'examples/{project}/') and path not in targets:
                if not any(path.parent == t.parent and re.search(r'\bmod\s+' + re.escape(path.stem) + r'\s*;', t.read_text()) for t in targets):
                    raise ValueError(f"Mapped source is not a Cargo example target/module: {row['file']}")
        run('check-' + project, cargo('check', '--all-targets', '--all-features'))
        if project == 'core':
            run('core-offline', cargo('test', '--test', 'offline'))
        if project == 'migration':
            for name, opts, stdin in [('offline', [], None), ('p16_2', [], None), ('p05_8', [], None),
                                     ('p05_6', [], None), ('p27_6', [], None), ('p27_7', [], None),
                                     ('p22_2', [], None), ('p22_3', [], None),
                                     ('p10_0', ['--offline'], 'y\n'), ('p10_0', ['--offline'], 'n\n')]:
                run('migration-' + name + ('-deny' if stdin == 'n\n' else ''),
                    cargo('run', '--example', name) + (['--'] + opts if opts else []), stdin,
                    {'OPENAI_API_KEY': 'offline-placeholder', 'DEEPSEEK_API_KEY': 'offline-placeholder'})
        if project == 'guides':
            run('custom-provider', cargo('run', '--example', 'custom_provider'))
            run('flight-format', cargo('test', '--features', 'flight', '--example', 'flight'))
        if project == 'integrations-check':
            run('mcp-offline', cargo('run', '--example', 'p47_005'))
            for name in ['p39_001', 'p39_002', 'p39_003', 'p39_004', 'p43_003', 'p45_003']:
                run('offline-' + name, cargo('run', '--example', name))
    if fingerprint() != before:
        raise RuntimeError('Compiler input changed during validation')
    if evidence:
        destination = R / '.ci-results'
        destination.mkdir(exist_ok=True)
        (destination / (project + '.json')).write_text(json.dumps({
            'project': project, 'fingerprint': before, 'commands': commands,
            'checked_at': datetime.now(timezone.utc).isoformat(),
            'rustc': subprocess.check_output(['rustc', '--version'], env=env, text=True).strip(),
        }, indent=2))
    else:
        for row in v['blocks'].values():
            if row['file'].startswith('examples/' + project + '/'):
                row['status'] = 'passed'
                row['sha256'] = hashlib.sha256((R / row['file']).read_bytes()).hexdigest()
        (R / 'validation.json').write_text(json.dumps(v, ensure_ascii=False, indent=2))


def main():
    parser = argparse.ArgumentParser()
    choice = parser.add_mutually_exclusive_group()
    choice.add_argument('--core', action='store_true')
    choice.add_argument('--project', choices=PROJECTS)
    parser.add_argument('--ci', action='store_true', help='Write fresh per-project evidence without changing inputs')
    args = parser.parse_args()
    for name in ([args.project] if args.project else ['core'] if args.core else PROJECTS):
        check(name, evidence=args.ci)
    print('PASS：编译与离线运行通过。未调用真实外部服务。')


if __name__ == '__main__':
    main()
