"""Pack/restore only documentation and examples, never workflows or executable scripts."""
import argparse
import json
from pathlib import Path
import shutil
import zipfile
from automation import ROOT, DATA_DIRS, DATA_FILES, allowed_data, data_files, safe_path


def pack(destination, root=ROOT):
    with zipfile.ZipFile(destination, 'w', zipfile.ZIP_DEFLATED) as archive:
        for path in data_files(root):
            archive.write(path, path.relative_to(root).as_posix())


def restore(source, root=ROOT):
    with zipfile.ZipFile(source) as archive:
        names = archive.namelist()
        if len(names) != len(set(names)) or sum(i.file_size for i in archive.infolist()) > 100_000_000:
            raise ValueError('Invalid candidate archive')
        for info in archive.infolist():
            if not allowed_data(info.filename) or (info.external_attr >> 16) & 0o170000 == 0o120000:
                raise ValueError('Candidate contains forbidden path')
            safe_path(root, info.filename)
        for required in ('source/manifest.json', 'validation.json', 'rust-toolchain.toml'):
            if required not in names:
                raise ValueError('Incomplete candidate')
        # Complete snapshot of these three owned directories, including deletions.
        for directory in DATA_DIRS:
            shutil.rmtree(root / directory, ignore_errors=True)
        for name in DATA_FILES:
            (root / name).unlink(missing_ok=True)
        for info in archive.infolist():
            path = safe_path(root, info.filename)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(archive.read(info))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('operation', choices=['pack', 'restore'])
    parser.add_argument('archive', type=Path)
    args = parser.parse_args()
    (pack if args.operation == 'pack' else restore)(args.archive)
