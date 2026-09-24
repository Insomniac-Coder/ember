"""The Unicode case-mapping tables of the runtime (`[TXT-10]`'s `to_upper` and
`to_lower`), generated from Python's own full case mappings.

Writes the section between the two markers in
`runtime/ember_rt/templates/ember_rt.c.in`; `--check` fails when the section is
not what this Python would write (the Unicode version is in its first line).

    python tools/unicode_case.py            # rewrite the section
    python tools/unicode_case.py --check    # verify it
"""
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEMPLATE = ROOT / 'runtime' / 'ember_rt' / 'templates' / 'ember_rt.c.in'
BEGIN = '/* BEGIN GENERATED: tools/unicode_case.py */\n'
END = '/* END GENERATED: tools/unicode_case.py */\n'


def table(name, pick):
    rows = []
    for cp in range(0x110000):
        if 0xD800 <= cp <= 0xDFFF:
            continue
        mapped = pick(chr(cp))
        if mapped != chr(cp):
            codes = [ord(c) for c in mapped] + [0, 0, 0]
            rows.append(f'    {{0x{cp:X}, {{0x{codes[0]:X}, 0x{codes[1]:X}, 0x{codes[2]:X}}}}},')
    return (f'static const ember_case_entry {name}[] = {{\n' + '\n'.join(rows) + '\n};\n'
            f'static const size_t {name}_len = sizeof {name} / sizeof {name}[0];\n')


def section():
    return (BEGIN
            + f'/* Unicode {unicodedata.unidata_version}: full case mappings, as Python {sys.version.split()[0]} gives them. */\n'
            + 'typedef struct ember_case_entry { uint32_t cp; uint32_t to[3]; } ember_case_entry;\n'
            + table('ember_upper_table', str.upper)
            + table('ember_lower_table', str.lower)
            + END)


def main():
    text = TEMPLATE.read_text(encoding='utf-8')
    start, end = text.index(BEGIN), text.index(END) + len(END)
    wanted = text[:start] + section() + text[end:]
    if '--check' in sys.argv:
        if wanted != text:
            print('the case tables are stale: run python tools/unicode_case.py')
            sys.exit(1)
        print('case tables current')
        return
    TEMPLATE.write_text(wanted, encoding='utf-8', newline='\n')
    print('wrote the case tables')


if __name__ == '__main__':
    main()
