"""Verdict 50: a generic function's (method's, struct's, class's) type parameters go on their own line
above it: `fn for_each[C, F: fn(Item) changes C](owns f: F) owns this changes C:` becomes
`[C, F: fn(Item) changes C]` on one line and `fn for_each(owns f: F) owns this changes C:` on the next.

In design code blocks the parameters move to a line of their own at the same indentation. In inline
code (prose, tables, messages), where a span cannot hold two lines, `fn f[P](...)` becomes
"`[P]` above `fn f(...)`". Today's Ember is left alone: an answer's parts 2 and 5, and spans written in
today's words (`owned`, `self`), today's compiler help quoted in the vocabulary's `some` row, the owner's own verdict 21 example, and the probe `r5_change_list.em`'s line, whose output is today's.
Usage: python v50_template_line.py <v3 dir> [--apply]
"""
import re
import sys
from pathlib import Path

FILES = ["rules.md", "vocabulary.md", "changes.md", "area-A.md", "area-B.md", "area-C.md", "area-D.md", "area-E.md"]
PART = re.compile(r"^\s*(?:(\d)\.\s+\*\*|\*\*(\d)\.\s)")
ANS = re.compile(r"^#{2,4} (AN-\d\d)")
HEAD = re.compile(r"((?:(?:pub|static|override|virtual|unsafe|open|safe)\s+)*(?:fn|struct|class)\s+\w+)\[")
TODAY_WORDS = re.compile(r"\bowned\b|\bself\b")
KEEP = {"fn f[T: I](x: T)", "fn for_each[C](f: fn(T) changes C) changes C:", "fn apply[C](f: fn(int) changes C) changes C:"}


def split_params(text, m):
    """Return (head, params, rest) for the declaration starting at m, matching nested brackets."""
    i = m.end() - 1
    depth = 0
    for j in range(i, len(text)):
        if text[j] == "[":
            depth += 1
        elif text[j] == "]":
            depth -= 1
            if depth == 0:
                return m.group(1), text[i:j + 1], text[j + 1:]
    raise ValueError("unbalanced: " + text)


def fix_block_line(line):
    code, sep, comment = line.partition("#")
    m = HEAD.search(code)
    if not m:
        return None
    indent = code[: len(code) - len(code.lstrip())]
    head, params, rest = split_params(code, m)
    before = code[: m.start()]
    if before.strip():
        return None
    return f"{indent}{params}\n{indent}{head}{rest}{sep}{comment}"


def fix_span(span):
    if span in KEEP or TODAY_WORDS.search(span):
        return span
    m = HEAD.search(span)
    if not m or m.start() != span.find(m.group(1)) or span[: m.start()].strip():
        return span
    head, params, rest = split_params(span, m)
    return f"{params}` above `{head}{rest}"


def main():
    root, apply = Path(sys.argv[1]), "--apply" in sys.argv
    total = 0
    for name in FILES:
        p = root / name
        out, ans, part, fenced, lang, changed = [], None, None, False, "", []
        for n, line in enumerate(p.read_text(encoding="utf-8").split("\n"), 1):
            s = line.strip()
            new = line
            if s.startswith("```"):
                fenced = not fenced
                lang = s[3:].strip() if fenced else ""
            elif fenced:
                today = ans is not None and part in ("2", "5")
                if lang == "ember" and not today:
                    r = fix_block_line(line)
                    if r is not None:
                        new = r
            else:
                m = ANS.match(line)
                if m:
                    ans, part = m.group(1), None
                else:
                    m = PART.match(line)
                    if m and ans:
                        part = m.group(1) or m.group(2)
                today = ans is not None and part in ("2", "5")
                if not today:
                    new = re.sub(r"`([^`\n]+)`", lambda mm: "`" + fix_span(mm.group(1)) + "`", line)
            if new != line:
                changed.append(f"{name}:{n}\n  - {line.strip()}\n  + {new.strip()}")
            out.append(new)
        total += len(changed)
        print("\n".join(changed))
        if apply:
            p.write_text("\n".join(out), encoding="utf-8")
    print(f"\n{total} lines changed" + ("" if apply else " (dry run)"))


if __name__ == "__main__":
    main()
