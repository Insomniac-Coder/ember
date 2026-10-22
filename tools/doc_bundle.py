"""`[DOC-4]` — the documentation a release publishes, as one archive: the user guide (`docs/book/`,
`[DOC-2]`), the error pages (`docs/errors/`, `[DOC-1]`) and the specification the compiler
implements (`docs/spec-source/development-target.json` names it), with an index that names the
release and each part. The release workflow (`.github/workflows/release.yml`) builds it after the
guide's and the pages' examples have passed under `ember test --doc`, and attaches it to the
release, so the three are always published together and for the same version.

    python tools/doc_bundle.py <version> <out-dir>     writes <out-dir>/ember-docs-<version>.zip
"""
import json
import pathlib
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def parts():
    """(path in the archive, file) for every file the archive holds, the index excepted."""
    target = json.loads((ROOT / 'docs' / 'spec-source' / 'development-target.json').read_text(encoding='utf-8'))
    spec = ROOT / target['path']
    out = [(f'book/{page.name}', page) for page in sorted((ROOT / 'docs' / 'book').glob('*.md'))]
    out += [(f'errors/{page.name}', page) for page in sorted((ROOT / 'docs' / 'errors').glob('*.md'))]
    out.append((f'specification/{spec.name}', spec))
    return out, spec.name


def index(version, files, spec_name):
    book = [name for name, _ in files if name.startswith('book/')]
    errors = [name for name, _ in files if name.startswith('errors/')]
    return '\n'.join([
        f'# Ember {version} documentation',
        '',
        'Published together for this release ([DOC-4]):',
        '',
        f'* the user guide, `book/` ({len(book)} pages, every sample run by `ember test --doc`): start at `book/README.md`;',
        f'* the error pages, `errors/` ({len(errors)} codes, each one\'s examples run too): `ember explain <code>` prints one;',
        f'* the specification, `specification/{spec_name}`.',
        '',
    ])


def bundle(version, out_dir):
    files, spec_name = parts()
    missing = [str(path) for _, path in files if not path.is_file()]
    if missing or not any(name.startswith('book/') for name, _ in files) or not any(name.startswith('errors/') for name, _ in files):
        raise SystemExit(f'doc_bundle: a part is missing: {missing or "the guide or the error pages"}')
    out_dir = pathlib.Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    archive = out_dir / f'ember-docs-{version}.zip'
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as zipped:
        zipped.writestr('index.md', index(version, files, spec_name))
        for name, path in files:
            zipped.write(path, name)
    return archive


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    print(bundle(sys.argv[1], sys.argv[2]))
