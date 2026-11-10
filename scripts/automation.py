"""Shared release invariants. No network, credentials, or model calls here."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PROJECTS = ('core', 'migration', 'guides', 'integrations-check',
            'integrations-local', 'integrations-lance', 'integrations-surreal')
DATA_DIRS = ('content', 'source', 'examples')
DATA_FILES = ('validation.json', 'translation-manifest.json', 'verification.json',
              'VERSIONS.md', 'QA.md', 'README.md', 'code-changes.diff',
              'update-report.json', 'rust-toolchain.toml')


def version_tuple(version):
    if not re.fullmatch(r'\d+\.\d+\.\d+', version):
        raise ValueError('Expected a stable x.y.z version')
    return tuple(map(int, version.split('.')))


def latest_stable(rows):
    rows = [r for r in rows if not r.get('yanked') and
            re.fullmatch(r'\d+\.\d+\.\d+', r.get('vers', ''))]
    if not rows:
        raise ValueError('Registry returned no non-yanked stable Rig release')
    return max(rows, key=lambda r: version_tuple(r['vers']))


def safe_path(root, name):
    relative = Path(name)
    if relative.is_absolute() or '..' in relative.parts or not relative.parts:
        raise ValueError('Unsafe relative path')
    path = root / relative
    if any((root / Path(*relative.parts[:i])).is_symlink()
           for i in range(1, len(relative.parts) + 1)):
        raise ValueError('Symlinks are not supported')
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError('Path escapes checkout')
    return path


def allowed_data(name):
    parts = Path(name).parts
    return (name in DATA_FILES or
            (len(parts) > 1 and parts[0] in DATA_DIRS and
             not any(p.startswith('.') or p in ('target', '__pycache__') for p in parts)))


def data_files(root=ROOT):
    for name in DATA_FILES:
        if (root / name).is_file():
            yield root / name
    for name in DATA_DIRS:
        for path in sorted((root / name).rglob('*')):
            rel = path.relative_to(root).as_posix()
            if path.is_file() and allowed_data(rel):
                safe_path(root, rel)
                yield path


def fingerprint(root=ROOT):
    # Same fingerprint in every isolated compiler job and in the publishing job.
    digest = hashlib.sha256()
    for path in sorted(data_files(root)):
        digest.update(path.relative_to(root).as_posix().encode() + b'\0')
        digest.update(path.read_bytes() + b'\0')
    return digest.hexdigest()


def clean_env():
    # Cargo/build scripts and example programs never inherit model or GitHub tokens.
    names = ('PATH', 'HOME', 'USER', 'TMPDIR', 'TEMP', 'TMP', 'SYSTEMROOT',
             'RUSTUP_HOME', 'CARGO_HOME', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS',
             'PROTOC', 'PKG_CONFIG_PATH', 'CC', 'CXX', 'LIBCLANG_PATH',
             'CARGO_PROFILE_DEV_DEBUG', 'CARGO_INCREMENTAL', 'ORT_LIB_LOCATION',
             'ORT_DYLIB_PATH', 'ORT_SKIP_DOWNLOAD')
    return {k: v for k, v in os.environ.items() if k in names}


def run(command, **kwargs):
    return subprocess.run(command, env=clean_env(), text=True, capture_output=True,
                          timeout=3600, **kwargs)


def validate_mapping(root=ROOT, require_hash=True):
    v = json.loads((root / 'validation.json').read_text())
    version_tuple(v['rig_version'])
    version_tuple(v['rustc'])
    expected = set()
    manifest = json.loads((root / 'source/manifest.json').read_text())
    for page in manifest:
        codes = json.loads((root / 'content/codes' / (page['id'] + '.codes.json')).read_text())
        official = json.loads((root / 'source' / (page['id'] + '.codes.json')).read_text())
        if len(codes) != len(official) or [c['lang'] for c in codes] != [c['lang'] for c in official]:
            raise ValueError(f"{page['id']}: official code count/languages cannot be changed to evade compilation")
        for index, code in enumerate(codes):
            if code['lang'] != 'rust':
                continue
            key = f"{page['id']}:{index}"
            expected.add(key)
            row = v['blocks'][key]
            path = safe_path(root, row['file'])
            parts = Path(row['file']).parts
            if len(parts) < 4 or parts[0] != 'examples' or parts[1] not in PROJECTS or parts[2] != 'examples' or path.suffix != '.rs':
                raise ValueError(f'{key}: compiler input must be an example in a checked project')
            if not code['text'].strip() or code['text'].strip() not in path.read_text():
                raise ValueError(f'{key}: displayed snippet differs from checked Rust source')
            if require_hash and (row['status'] != 'passed' or
                    hashlib.sha256(path.read_bytes()).hexdigest() != row['sha256']):
                raise ValueError(f'{key}: stale compilation evidence')
    if set(v['blocks']) != expected:
        raise ValueError('Every current Rust block needs exactly one mapping; stale mappings are forbidden')
    for project in PROJECTS:
        manifest_path = root / 'examples' / project / 'Cargo.toml'
        cargo = tomllib.loads(manifest_path.read_text())
        dep = cargo['dependencies']['rig']
        if (dep if isinstance(dep, str) else dep['version']) != '=' + v['rig_version']:
            raise ValueError(f'{project}: Rig version pin mismatch')
        lock = tomllib.loads(manifest_path.with_name('Cargo.lock').read_text())
        versions = {p['version'] for p in lock['package'] if p['name'] == 'rig'}
        if versions != {v['rig_version']}:
            raise ValueError(f'{project}: Cargo.lock Rig version mismatch')
    return v
