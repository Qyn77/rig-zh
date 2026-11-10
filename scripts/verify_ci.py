"""Publish only after all seven isolated compiler jobs checked identical input."""
from datetime import datetime, timezone
import hashlib
import json
from automation import ROOT, PROJECTS, fingerprint, validate_mapping


def verify(root=ROOT):
    expected = fingerprint(root)
    v = validate_mapping(root, require_hash=False)
    for project in PROJECTS:
        record = json.loads((root / '.ci-results' / (project + '.json')).read_text())
        if record['project'] != project or record['fingerprint'] != expected:
            raise ValueError(f'{project}: compilation evidence belongs to different source')
        if not record['commands'] or any(c['exit_code'] != 0 for c in record['commands']):
            raise ValueError(f'{project}: checks did not succeed')
        if not any(c['label'] == 'check-' + project for c in record['commands']):
            raise ValueError(f'{project}: missing actual cargo check')
        if not record['rustc'].startswith('rustc ' + v['rustc'] + ' '):
            raise ValueError(f'{project}: incorrect Rust toolchain')
    for row in v['blocks'].values():
        row['status'] = 'passed'
        row['sha256'] = hashlib.sha256((root / row['file']).read_bytes()).hexdigest()
    v['checked_at'] = datetime.now(timezone.utc).date().isoformat()
    (root / 'validation.json').write_text(json.dumps(v, ensure_ascii=False, indent=2))
    if (root / 'update-report.json').exists():
        report = json.loads((root / 'update-report.json').read_text())
        report.update(status='validated', checked_at=v['checked_at'], input_fingerprint=expected)
        (root / 'update-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print('PASS: all seven projects compiled this exact candidate; offline regressions passed')


if __name__ == '__main__':
    verify()
