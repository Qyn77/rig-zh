"""Check the official Cargo sparse index; ignore prereleases and yanked versions."""
import argparse
import json
import os
from pathlib import Path
import urllib.request
from automation import ROOT, latest_stable, version_tuple

INDEX = 'https://index.crates.io/3/r/rig'


def detect(text, current):
    release = latest_stable([json.loads(line) for line in text.splitlines() if line.strip()])
    return {'update': version_tuple(release['vers']) > version_tuple(current),
            'current': current, 'version': release['vers'], 'release': release, 'source': INDEX}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--index-file', type=Path)
    args = parser.parse_args()
    if args.index_file:
        text = args.index_file.read_text()
    else:
        req = urllib.request.Request(INDEX, headers={'User-Agent': 'Qyn77/rig-zh release checker'})
        with urllib.request.urlopen(req, timeout=60) as response:
            text = response.read().decode()
    current = json.loads((ROOT / 'validation.json').read_text())['rig_version']
    report = detect(text, current)
    (ROOT / 'release-candidate.json').write_text(json.dumps(report, ensure_ascii=False, indent=2))
    print(json.dumps({k: report[k] for k in ('current', 'version', 'update')}))
    if os.environ.get('GITHUB_OUTPUT'):
        with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
            output.write(f"update={str(report['update']).lower()}\nversion={report['version']}\n")
    if os.environ.get('GITHUB_STEP_SUMMARY'):
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a') as output:
            output.write(f"当前中文版：{current}；官方最新稳定版：{report['version']}。\n")


if __name__ == '__main__':
    main()
