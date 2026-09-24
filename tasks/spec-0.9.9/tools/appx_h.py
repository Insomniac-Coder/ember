"""Writes parts/p26-appx-h.md: the change list from 0.9.8_Hardened_3 plus the generated list of
0.9.8 rule ids that 0.9.9 no longer defines, grouped by family with the reason."""
import re, glob, os, collections

HERE = os.path.dirname(os.path.abspath(__file__))
OLD = os.path.join(HERE, '..', '..', '..', 'docs', 'spec-source', 'Ember_v0.9.8_Hardened_3.md')

REASON = {
    'ABI': 'condensed into `[ABI-1]`',
    'ARN': 'sub-rules folded into `[ARN-5]` and `[ARN-8]`',
    'AST': 'compiler internals; out of the language document',
    'ATT': 'folded into the attribute table and `[GRM-20]`',
    'BEN': 'benchmark plan; replaced by the performance gate `[TST-28]`',
    'BLD': 'build-system internals; the user-visible parts are §XVII.3',
    'BLD-FFI': 'C++ build integration; see Annex C',
    'BUD': 'compile-time budget details; condensed into `[BLD-10]`',
    'CAT': 'rule categories retired; conformance profiles (`[CONF-*]`) replace them',
    'CELL': 'folded into `[CELL-6]` and the prelude (`[MOD-5]`)',
    'CLI': 'condensed into the command listing of §XVII.1 and `[CLI-19]`',
    'CLO-ABI': 'compiler internals',
    'CMP': 'compiler internals',
    'COMP': 'compiler internals',
    'CONF': 'folded into `[CONF-1]`',
    'COR-ABI': 'compiler internals',
    'CTL': 'folded into `[CTL-3]`; its conformance obligations are `[TST-4a]`',
    'CTR': 'retired in 0.6.2',
    'CXX': 'C++ corpus details; `[CXX-1]` in Annex C',
    'DET': 'folded into `[DET-1]`; `ember inspect --deterministic` is in §XVII.1',
    'DET-IMPL': 'compiler internals',
    'DIA': 'condensed into §XVII.6 (shape tables, `[DIA-21]`, `[DIA-24]`)',
    'DOC': 'folded into `[DOC-1]` and `[DOC-2]`',
    'EFF': 'folded into `[EFF-11]`, `[EFF-16]` and `[EFF-19]`',
    'EXC': 'performance gate folded into `[TST-28]`',
    'EXP': 'replaced by floor semantics, `[TYP-28]`',
    'FFI': 'C++ interop details (Annex C) or folded into the five-axis contract `[FFI-11]`',
    'FFI-CB': 'folded into `[FFI-21]`',
    'FFI-IMPL': 'compiler internals',
    'FMT': 'folded into `[FMT-1]`',
    'FN': 'removed with `@latebound` (F-168); callable types elide regions per call (`[LT-7]`)',
    'GATE': 'implementation plan; out of the language document',
    'GEN-COH': 'folded into `[TYP-20]`',
    'GPU': 'condensed into Annex D',
    'GRM': 'folded into Part III (`GRM-14` retired with the access-mode generic kind, F-024)',
    'HIR': 'compiler internals',
    'HOT': 'compiler internals of hot reload',
    'HR': 'condensed into Annex B',
    'HR-IMPL': 'compiler internals of hot reload',
    'IDE': 'language-server design; out of the language document',
    'IMP': 'implementation plan; out of the language document (`[IMP-11]` is new)',
    'JOB': 'debug-only access-set verification removed (a check in one profile is not a guarantee, `[PHIL-13]`)',
    'LAY': 'replaced by `[LAY-2]`',
    'LEX': 'folded into `[LEX-14]` and `[LEX-15]`',
    'LT': '`with_views*` removed (F-168); multi-region rules condensed into §VII.4',
    'MAN': 'folded into `[MAN-8]`',
    'MIR': 'compiler internals',
    'MIR-REG': 'compiler internals',
    'MNG': 'folded into `[MNG-1]`',
    'MOD': 'language-version selectors removed (`[VER-8]`)',
    'MONO': 'folded into `[MONO-2]` and `[MONO-8]`',
    'OPT': 'conformance obligation folded into `[TST-4a]`',
    'OQ8': 'open questions closed by this revision',
    'PRV': 'retired with the prover in 0.6.2',
    'RC': 'still citable as the lettered cases of `[RC-2]`',
    'RNG': 'folded into `[RNG-5]` and `[RNG-10]`',
    'RT': 'folded into `[RT-1]` and `[HND-3]`',
    'RV': 'host-engine integration plan; out of the language document',
    'SOA': '`columns_mut` retired: columns are disjoint places (`[SOA-2]`)',
    'SPN': 'folded into `[SPN-4]` and Part XV',
    'STD': 'folded into `[STD-7]`',
    'STD-IMPL': 'compiler internals',
    'TCB': 'condensed into `[TCB-1]` (Annex C)',
    'TOOL': 'toolchain milestones; out of the language document (`[CLI-4]` keeps the no-manifest rule)',
    'TST': 'condensed into §XVII.5',
    'UNS': 'the reason category moved into the `# SAFETY(…):` note (`[LEX-23]`, `[UNS-8]`)',
    'VER': 'folded into `[VER-2]`',
    'VERIFY': 'compiler internals',
    'WK': 'folded into `[WK-4]`, `[WK-6]` and `[WK-15]`',
}

HEAD = """---

# Appendix H — Changes from 0.9.8_Hardened_3

## H.1 What changed, by goal

**Memory safety.**
* Only `@sync` classes are shareable, and their fields are immutable after `init`; Safe Ember has no
  data race (`[THR-1]`, `[THR-13]`).
* Assigning a non-`Copy` field through a class handle is a checked write access (`[EXC-16]`).
* No setting removes a safety check: `exclusivity = "unchecked"` and `gpu.validate` are gone, overflow
  and stale-handle checks run in every profile (`[PRF-1]`, `[TYP-8]`, `[HND-1]`, `[GPU-1]`).
* Checked capacity arithmetic and honoured alignment in the runtime (`[HEAP-8]`, `[HEAP-9]`).
* Compile-time heap data can never be grown or freed at run time (`[CT-5]`).

**C-like speed.**
* UB-free C, floating-point flags, injective mangling, inline count fast paths (`[CG-C-1]`, `[CG-C-11]`,
  `[MNG-1]`, `[RT-10]`).
* Grouped overflow checks keep checked arithmetic vectorisable (`[SIMD-7]`).
* A performance gate against equivalent C (`[TST-28]`).

**Python ergonomics.**
* `int` is `i64`, `float` is `f64`; integer `/` is rejected, `//` and `%` are floor (`[TYP-1]`,
  `[TYP-28]`).
* String literals initialise `String`; collection literals and comprehensions; chained comparisons;
  `x is None`; unparenthesised tuples; names assigned in every branch; implicit `Eq`/`Debug`/`Clone`;
  lazy statics; generators and `some` returns; `Result[T]` with `AnyError`; ordered `Map`; `print`
  with several arguments; callable fields; Python-habit fix-its (`[TXT-9]`, `[TYP-38]`, `[GRM-25]`,
  `[EXP-9]`, `[GRM-29]`, `[CTL-10]`, `[STR-5]`, `[STA-3]`, `[CORO-1]`, `[TYP-32]`, `[ERR-9]`,
  `[STD-11]`, `[STD-9]`, `[CLO-11]`, `[DIA-21]`).
* Rust-style friction removed: no `::`, no lifetime syntax ever, `@view` inferred, coherence per
  package, `with_views*` and `@latebound` removed, field-sensitive private methods, arena elision,
  location-sensitive borrow checking (`[GRM-24]`, `[LEX-22]`, `[TYP-34]`, `[TYP-20]`, `[LT-7]`,
  `[BRW-10]`, `[LT-44]`, `[BCK-2]`).

**Coherence.**
* No silent acceptance; one meaning per program; Python spelling means Python meaning; costs are named
  (`[PHIL-12]`–`[PHIL-15]`).
* One language before 1.0 (`[VER-8]`). The document is the language only: compiler architecture, the
  implementation plan and the host-engine plan are no longer in it; C++, hot reload and GPU are
  annexes.

## H.2 Removed constructs

`::`; `'a` lifetime reservation; `with_views`, `with_views2..4`, `@latebound`; `@thread_local`;
`@derive(SoA)` and `columns_mut`; `Callable[…]` in source; `PartialEq`/`PartialOrd`; `unsafe(reason =
…)`; `#! language` selectors other than the current version; the manifest keys `exclusivity`,
`overflow`, `bounds_checks` and `gpu.validate`; job access-set declarations verified only in debug.

## H.3 Rule identifiers no longer defined

Generated: every 0.9.8 identifier that this revision does not define, grouped by family. An identifier
is never reused for another meaning.

| Family | Identifiers | Why |
|---|---|---|
"""

H4_HEAD = """
## H.4 Changes from 0.9.9_Hardened_1 to 0.9.9_Hardened_2

Hardened_2 applies the change list drawn from three passes over Hardened_1 (memory safety; consistency
and speed; ergonomics). Before 1.0 a hardening may change the language when this log lists each such
change (`[VER-9]`). These change which programs are accepted, or what they do:

| Change | Effect on existing programs | Rules |
|---|---|---|
| a thread or unscoped job carries only the static region | a spawn capturing a view of a local is rejected; use `thread.scope()` | `[THR-10]`, `[JOB-2]` |
| `Shared[T]` is one-thread; `SyncShared[T]` crosses threads; `get`/`get_mut` are checked accesses | sending a `Shared` is rejected; overlapping `get_mut` panics | `[HEAP-4]`, `[HEAP-5]`, `[HEAP-10]`, `[THR-8]`, `[THR-9]` |
| two-phase construction | a derived `init` that calls `super.init` before assigning its own fields is rejected (`E2102`) | `[CLS-2]`, `[CLS-4]`, `[CLS-10]`, `[CLS-11]` |
| retained and once C callbacks own static captures; callback captures are `Send` unless the contract names a thread | such callbacks capturing borrows or non-`Send` handles are rejected | `[FFI-21]`, `[FFI-22]` |
| foreign functions are safe to call only when asserted: `safe fn`, or an overlay entry | calls to unlisted header functions and to extern-block functions not marked `safe` need `unsafe` | `[FFI-1]`, `[FFI-2]`, `[FFI-10]` |
| `yield` while a `@must_drop` value is live | rejected (`E2231`) | `[CORO-13]` |
| `char32_t` is `u32` | foreign signatures using it change type | `[FFI-8]` |
| SIMD memory operations are bounds-checked | out-of-range `load`/`store`/`gather`/`scatter` panic | `[SIMD-9]` |
| calling a callable field is a write access; owned callable values are called from a mutable place | a callback that replaces itself while running panics | §VIII.3, `[CLO-2]` |
| accesses held across a call into C++ stay checked | a conflicting re-entrant override panics | `[FFI-39d]`, `[EXC-7]` |
| `run_parallel` checks systems given as values | a conflict panics before any system starts | `[ECS-4]` |
| phantom parameters count for `Send`/`Sync`; `comptime.read_file` stays inside the package | fewer types are `Send`; reads outside the package are `E6010` | `[TYP-35]`, `[CT-2]` |
| exclusivity is tracked per field | reading one field while writing another no longer panics | `[EXC-19]`, `[EXC-1]`, `[EXC-2]`, `[EXC-15]` |
| a `Map` iterates its keys | `for k, v in m:` becomes `for k, v in m.items():` (`ember fmt --migrate`) | `[CTL-1]`, `[STD-16]` |
| the entry file may hold top-level statements | scripts need no `main` | `[GRM-2]`, `[FN-8]` |
| `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all` in the prelude | accepted with Python's meaning; `len` of a string is `E2073` | `[STD-26]`, `[MOD-5]` |
| generator expressions | accepted | `[GRM-38]` |
| printing falls back from `Display` to `Debug` | more values print; none prints differently | `[STD-9]`, `[LEX-19]` |
| `T` converts to `Option[T]`; `None` then `T` infers `Option[T]` | accepted | `[TYP-5]`, `[TYP-23]` |
| a `Result[void, E]` function returns `Ok(())` at its end | accepted | `[FN-10]` |
| a lambda passed to a consumed callable parameter captures by move | accepted without `owned fn` | `[CLO-15]` |
| standard modules importable without `std.` | accepted; a package module of the same name wins (`W1003`) | `[MOD-3]` |
| iterator adapters on any `Iterable` | accepted | `[STD-19]` |
| float `Display` is the shortest round-trip text | printed floats may change | `[STD-20]` |
| `W2015` only for more digits than the type keeps | fewer warnings | `[LEX-17a]` |
| contract verdicts assume every listed proof applied; grouping required in vectorisable loops | verdicts agree between implementations | `[EFF-15]`, `[SIMD-7]`, `[SIMD-5]` |

Generated: every rule whose text differs between the two hardenings.

| Rule | H1 → H2 |
|---|---|
"""

H1_PARTS = os.path.join(HERE, '..', 'parts-h1')


def _bullets(folder):
    out = {}
    for p in sorted(glob.glob(os.path.join(folder, 'p*.md'))):
        if p.endswith(('p17a-codes.md', 'p25-appx-g.md', 'p26-appx-h.md', 'p27-appx-i.md')):
            continue
        lines = open(p, encoding='utf-8').read().split('\n')
        for i, line in enumerate(lines):
            m = re.match(r'^\* `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]`', line)
            if m:
                k = i + 1
                while k < len(lines) and lines[k].startswith('  '):
                    k += 1
                out[m.group(1)] = ' '.join(' '.join(lines[i:k]).split())
    return out


H2_PARTS = os.path.join(HERE, '..', 'parts-h2')

# One row per ODR a hardening after Hardened_2 records: (ODR, ruling, rules).
ODRS = [
    ('ODR-021', 'float `//` and `%` are Python\'s: the exact floor modulo rounded once, and the '
                'floor quotient consistent with it', '`[TYP-29]`'),
    ('ODR-022', 'an untyped literal is never left open: `total = 0` declares an `int` whatever '
                'the later uses', '`[TYP-23]`'),
    ('ODR-023', 'a function returning a value that can reach the end of its body is `E2182` '
                '(Hardened_4)', '`[FN-10]`, §XVII.9'),
    ('ODR-024', 'a borrowed or `mut` parameter whose type is not `Copy` is a source parameter a '
                'returned view may borrow, and a borrowed parameter is passed by address except a '
                'view or a `Copy` value holding no `Cell` (Hardened_5)',
     '`[LT-1]`, `[LT-1a]`, `[LT-1b]`, `[LT-7]`, `[LT-44]`, `[FN-1]`, `[FN-3]`, `[FN-6]`, `[BRW-8]`, '
     '`[CORO-6]`, §VIII.3, §XVII.6 B7, §XVII.9, Appendix F'),
    ('ODR-025', '`[ERR-4]`\'s methods that take a function are eager, move the payload in (`filter` '
                'borrows it) and take `once fn`; an unannotated lambda parameter takes `owned` from the '
                'expected callable type, never `mut` (Hardened_6)',
     '`[ERR-4]`, `[CLO-7]`, `[TYP-23]`'),
    ('ODR-026', 'a type that declares `drop` is not implicitly `Clone`; `@derive(Clone)` or a written '
                '`clone` gives it one (Hardened_7)', '`[STR-5]`'),
    ('ODR-027', "a range is a value: the prelude's range types are structs with public bounds, `Copy` "
                'when the bound is, and a `for` over one counts over a copy of its bounds, leaving it '
                "unchanged; `a..` overflows at its type's maximum (Hardened_8)",
     '`[CTL-3]`, `[STD-8]`, `[STD-26]`'),
]

H5_HEAD = """
## H.5 Changes from 0.9.9_Hardened_2

Each ambiguity found while implementing 0.9.9 is an owner decision request (`docs/OWNER-QUEUE.md`),
ruled under the owner's delegation and recorded here. Hardened_3 carries ODR-021 and ODR-022;
Hardened_4 adds ODR-023; Hardened_5 adds ODR-024; Hardened_6 adds ODR-025; Hardened_7 adds ODR-026;
Hardened_8 adds ODR-027.

| ODR | Ruling | Rules |
|---|---|---|
"""


def h2_to_now():
    head = H5_HEAD + '\n'.join(f'| {o} | {r} | {rules} |' for o, r, rules in ODRS)
    diff = rule_diff(H2_PARTS, os.path.join(HERE, '..', 'parts'))
    return head + '\n\nGenerated: every rule whose text differs from Hardened_2.\n\n' \
        + '| Rule | H2 → now |\n|---|---|\n' + diff


def h1_to_h2():
    return rule_diff(H1_PARTS, H2_PARTS)


def rule_diff(old_folder, new_folder):
    old, new = _bullets(old_folder), _bullets(new_folder)
    fam = lambda x: re.sub(r'-[0-9]+[a-z0-9]*$', '', x)
    num = lambda x: int(re.search(r'-([0-9]+)[a-z0-9]*$', x).group(1))
    rows = []
    for rid in sorted(set(old) | set(new), key=lambda x: (fam(x), num(x), x)):
        if rid not in old:
            rows.append(f'| `[{rid}]` | added |')
        elif rid not in new:
            rows.append(f'| `[{rid}]` | removed |')
        elif old[rid] != new[rid]:
            rows.append(f'| `[{rid}]` | changed |')
    return '\n'.join(rows)


def main():
    old = open(OLD, encoding='utf-8').read()
    oldids = set(re.findall(r'`\[([A-Z][A-Z0-9]*(?:-[A-Z]+)?-[0-9]+[a-z0-9]*)\]`', old))
    new = set()
    for p in sorted(glob.glob(os.path.join(HERE, '..', 'parts', 'p*.md'))):
        if p.endswith(('p25-appx-g.md', 'p26-appx-h.md', 'p27-appx-i.md')):
            continue
        new.update(re.findall(r'^\* `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]`', open(p, encoding='utf-8').read(), re.M))
    gone = oldids - new
    fam = collections.defaultdict(list)
    for x in gone:
        fam[re.sub(r'-[0-9]+[a-z0-9]*$', '', x)].append(x)
    key = lambda x: (int(re.search(r'-([0-9]+)[a-z0-9]*$', x).group(1)), x)
    rows = []
    missing = []
    for f in sorted(fam):
        ids = ', '.join(sorted(fam[f], key=key))
        if f not in REASON:
            missing.append(f)
        rows.append(f'| {f} | {ids} | {REASON.get(f, "TODO")} |')
    h4 = h1_to_h2()
    open(os.path.join(HERE, '..', 'parts', 'p26-appx-h.md'), 'w', encoding='utf-8', newline='\n').write(
        HEAD + '\n'.join(rows) + '\n' + H4_HEAD + h4 + '\n' + h2_to_now() + '\n')
    print('retired ids', len(gone), 'families', len(fam), 'families without a reason', missing,
          'H1->H2 rows', h4.count('\n') + 1)
    return gone

if __name__ == '__main__':
    main()
