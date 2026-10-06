"""The README's benchmark charts and its count of slow programs, from one run's logs.

usage: python bench/update_readme.py <gcc log> <windows log> [more windows logs]
  A Windows log holds each set with MSVC, then with clang: a program's first line is MSVC's, its
  second clang's. A later Windows log overrides an earlier one's programs (a disturbed row timed
  again alone). Writes docs/benchmarks/{as-fast-as-c,close-to-c,slower-than-c}.svg and the
  "More than 10% slower than C" sentence in README.md. A program goes in the table of its slowest
  compiler: under x1.05, x1.05 to x1.10, over x1.10."""
import html
import pathlib
import re
import sys

from descriptions import DESC, MARKS

ROOT = pathlib.Path(__file__).resolve().parent.parent
ROW = re.compile(r'^(\S+)\s+ember\s+([\d.]+)s \| \w+\s+([\d.]+)s \| ember/ref\s+([\d.]+)x', re.M)
CCS = ('msvc', 'clang', 'gcc')
FILES = {'same': 'as-fast-as-c', 'close': 'close-to-c', 'slow': 'slower-than-c'}

# Colours of GitHub's dark theme.
S = dict(bg='#0d1117', text='#e6edf3', muted='#9198a1', rule='#30363d', band='#151b23',
         good='#3fb950', mid='#d29922', bad='#f85149', head='#e6edf3')
FONT = "-apple-system, 'Segoe UI', Helvetica, Arial, sans-serif"
MONO = "ui-monospace, 'SFMono-Regular', Consolas, monospace"
# Approximate advance widths (em) of a sans-serif face, for wrapping only.
NARROW, WIDE = set("ijlt.,:;'`!|()[]{} fr-"), set('mwMW%')


def read(gcc_log, windows_logs):
    """name -> {compiler: (ember seconds, C seconds, printed ratio)}"""
    results = {}
    for name, ember, ref, printed in ROW.findall(pathlib.Path(gcc_log).read_text(encoding='utf-8')):
        results.setdefault(name, {})['gcc'] = (float(ember), float(ref), float(printed))
    for log in windows_logs:
        seen = set()
        for name, ember, ref, printed in ROW.findall(pathlib.Path(log).read_text(encoding='utf-8')):
            cc = 'clang' if name in seen else 'msvc'
            seen.add(name)
            results.setdefault(name, {})[cc] = (float(ember), float(ref), float(printed))
    missing = [n for n in DESC if any(cc not in results.get(n, {}) for cc in CCS)]
    if missing:
        sys.exit(f'no result for: {missing}')
    return results


def width(text, size, mono=False):
    if mono:
        return len(text) * 0.6 * size
    total = 0.0
    for ch in text:
        total += 0.26 if ch in NARROW else 0.78 if ch in WIDE else 0.62 if ch.isupper() else 0.5
    return total * size


def tokens(text):
    """Words (split at spaces), each a list of (piece, is_code): backticks toggle code."""
    out, code = [], False
    for word in text.split(' '):
        pieces = []
        for i, piece in enumerate(word.split('`')):
            if i:
                code = not code
            if piece:
                pieces.append((piece, code))
        if pieces:
            out.append(pieces)
    return out


def wrap(text, size, limit):
    lines, line, used = [], [], 0.0
    space = width(' ', size)
    for word in tokens(text):
        w = sum(width(piece, size, code) for piece, code in word)
        if line and used + space + w > limit:
            lines.append(line)
            line, used = [], 0.0
        used += (space if line else 0) + w
        line.append(word)
    if line:
        lines.append(line)
    return lines


def ms(seconds):
    value = seconds * 1000
    return f'{value:.1f} ms' if value < 10 else f'{value:.0f} ms'


def colour(ratio):
    return S['good'] if ratio < 1.05 else S['mid'] if ratio <= 1.10 else S['bad']


def table(names, results):
    size, line_h, pad = 14, 20, 10
    desc_w, col, x_desc = 400, 74, 16
    x0 = x_desc + desc_w + 18
    groups = [('MSVC', 'msvc', x0), ('clang', 'clang', x0 + 3 * col + 24), ('gcc (Linux)', 'gcc', x0 + 6 * col + 48)]
    total_w = groups[2][2] + 3 * col + 16
    head_h = 58
    rows, y = [], head_h
    for n in names:
        lines = wrap(DESC[n] + MARKS.get(n, '').replace(' ', '\u00a0'), size, desc_w)
        h = len(lines) * line_h + 2 * pad
        rows.append((n, lines, y, h))
        y += h
    total_h = y + 8
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="{total_h}" viewBox="0 0 {total_w} {total_h}" '
           f'font-family="{FONT}" font-size="{size}">',
           f'<rect width="{total_w}" height="{total_h}" fill="{S["bg"]}" rx="6"/>']
    out.append(f'<text x="{x_desc}" y="46" fill="{S["muted"]}" font-size="12">What the program does</text>')
    for label, _, gx in groups:
        out.append(f'<text x="{gx + 1.5 * col}" y="22" fill="{S["head"]}" font-weight="600" text-anchor="middle">{label}</text>')
        out.append(f'<line x1="{gx + 6}" y1="30" x2="{gx + 3 * col - 6}" y2="30" stroke="{S["rule"]}"/>')
        for i, name in enumerate(('C', 'Ember', 'Ember ÷ C')):
            out.append(f'<text x="{gx + (i + 1) * col - 8}" y="46" fill="{S["muted"]}" font-size="12" text-anchor="end">{name}</text>')
    out.append(f'<line x1="0" y1="{head_h - 1}" x2="{total_w}" y2="{head_h - 1}" stroke="{S["rule"]}"/>')
    for k, (n, lines, top, h) in enumerate(rows):
        if k % 2 == 1:
            out.append(f'<rect x="0" y="{top}" width="{total_w}" height="{h}" fill="{S["band"]}"/>')
        for j, line in enumerate(lines):
            parts = []
            for i, word in enumerate(line):
                parts.append(' ' if i else '')
                for piece, code in word:
                    piece = html.escape(piece)
                    parts.append(f'<tspan font-family="{MONO}" font-size="13">{piece}</tspan>' if code else piece)
            out.append(f'<text x="{x_desc}" y="{top + pad + 15 + j * line_h}" fill="{S["text"]}" xml:space="preserve">{"".join(parts)}</text>')
        mid = top + h / 2 + 5
        for _, cc, gx in groups:
            ember, ref, ratio = results[n][cc]
            out.append(f'<text x="{gx + col - 8}" y="{mid}" fill="{S["muted"]}" text-anchor="end">{ms(ref)}</text>')
            out.append(f'<text x="{gx + 2 * col - 8}" y="{mid}" fill="{S["text"]}" text-anchor="end">{ms(ember)}</text>')
            # The owner (2026-10-02): a value under x0.95 (Ember at least 5% faster than C) is bold
            # and green; every other value is in the regular weight, in its colour.
            weight = '700' if round(ratio, 2) < 0.95 else '400'
            out.append(f'<text x="{gx + 3 * col - 8}" y="{mid}" fill="{colour(ratio)}" text-anchor="end" font-weight="{weight}">×{ratio:.2f}</text>')
    out.append('</svg>')
    return '\n'.join(out)


def counts_sentence(results):
    slow = {cc: {n for n in DESC if results[n][cc][2] > 1.10} for cc in CCS}
    everywhere = slow['msvc'] & slow['clang'] & slow['gcc']
    all_three = 'none with all three' if not everywhere else f'{len(everywhere)} with all three'
    print({cc: sorted(v) for cc, v in slow.items()})
    return (f"More than 10% slower than C: {len(slow['msvc'])} of the {len(DESC)} programs with MSVC, "
            f"{len(slow['clang'])} with clang, {len(slow['gcc'])} with gcc; {all_three}.")


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    results = read(sys.argv[1], sys.argv[2:])
    worst = {n: max(results[n][cc][2] for cc in CCS) for n in DESC}
    sections = {
        'same': sorted([n for n in DESC if worst[n] < 1.05], key=lambda n: min(results[n][cc][2] for cc in CCS)),
        'close': [n for n in DESC if 1.05 <= worst[n] <= 1.10],
        'slow': sorted([n for n in DESC if worst[n] > 1.10], key=lambda n: -worst[n]),
    }
    for section, names in sections.items():
        if names:
            path = ROOT / 'docs' / 'benchmarks' / f'{FILES[section]}.svg'
            path.write_text(table(names, results), encoding='utf-8', newline='\n')
    counts = counts_sentence(results)
    readme = ROOT / 'README.md'
    text = readme.read_text(encoding='utf-8')
    old = re.search(r'More than 10% slower than C: \d+ of the \d+ programs with MSVC, \d+ with clang, \d+ with gcc;[^.]*\.', text)
    if not old:
        sys.exit('README.md: the counts sentence is not there')
    lines, line = [], ''
    for word in counts.split(' '):
        if line and len(line) + 1 + len(word) > 100:
            lines.append(line)
            line = word
        else:
            line = f'{line} {word}' if line else word
    lines.append(line)
    readme.write_text(text[:old.start()] + '\n'.join(lines) + text[old.end():], encoding='utf-8', newline='\n')
    print({k: len(v) for k, v in sections.items()}, counts)


if __name__ == '__main__':
    main()
