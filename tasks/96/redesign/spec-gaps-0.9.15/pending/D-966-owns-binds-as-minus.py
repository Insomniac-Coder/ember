"""D-966 (NOT APPLIED; for the next agent): `owns v as u8` is `(owns v) as u8`, as the grammar says (`unary := value_word unary`)."""
import os
# the repository root, from tasks/96/redesign/spec-gaps-0.9.15/pending/
R = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..", "..")).replace("\\", "/") + "/"


def edit(path, pairs):
    p = R + path
    s = open(p, 'rb').read().decode('utf-8')
    for old, new, n in pairs:
        c = s.count(old)
        assert c == n, (path, c, old[:90])
        s = s.replace(old, new)
    open(p, 'wb').write(s.encode('utf-8'))
    print('ok', path)


edit("compiler/ember_parser/src/body.rs", [
("""                self.bump_value_word();
                // `[IDE-3]` — the value it marks, which is checked as written.
                self.parse_expr_bp(BP_CAST, allow_block_lambda).kind""",
"""                self.bump_value_word();
                // `[IDE-3]` — the value it marks, which is checked as written.
                self.parse_expr_bp(BP_UNARY + 1, allow_block_lambda).kind""", 1),
("""            TokenKind::Keyword(Kw::Owns) => {
                self.bump();
                ExprKind::Owned(Box::new(self.parse_expr_bp(BP_CAST, allow_block_lambda)))
            }""",
"""            // It binds as `-` does (`unary := value_word unary`): `owns v as u8` is `(owns v) as u8`.
            TokenKind::Keyword(Kw::Owns) => {
                self.bump();
                ExprKind::Owned(Box::new(self.parse_expr_bp(BP_UNARY + 1, allow_block_lambda)))
            }""", 1),
])

test = """#$ test: compile-fail
#$ rules: GRM-15, FN-2a
# `[GRM-15]` — `owns` binds as `-` does (`unary := value_word unary`, `conversion := unary {"as"
# type}`): `owns v as u8` is `(owns v) as u8`, a `u8` made after `v` is handed over, so `push` is
# not given its value with `owns` (`E3028`); in brackets, `owns (v as u8)`, it is.

fn main():
    out: Array[u8] = []
    v = 300
    out.push(owns (v as u8))
    out.push(owns v as u8)    #$ error[E3028]: say so at the call: `out.push(owns (v as u8))`
    #$ error[E0900]: `owns` before a value here is not built by this compiler yet
    println(out.len())
"""
p = R + "tests/conformance/GRM-15/reject_owns_binds_as_minus_before_as.em"
open(p, 'wb').write(test.encode('utf-8'))
print('ok', p)

# DEFECTS.md: the open D-966 row becomes the fixed one
p = R + "docs/DEFECTS.md"
lines = open(p, 'rb').read().decode('utf-8').split('\n')
i = [k for k, l in enumerate(lines) if l.startswith('| D-966 |')]
assert len(i) == 1
lines[i[0]] = lines[i[0]].replace('| **open, fix ready** |', '| **fixed** |', 1)
open(p, 'wb').write('\n'.join(lines).encode('utf-8'))
print('ok', p)
