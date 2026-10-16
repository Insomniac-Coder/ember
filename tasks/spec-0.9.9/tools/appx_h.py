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
                "unchanged; `a..` overflows past its type's maximum (Hardened_8; since 0.9.10 it gives the "
                'maximum first, D-526)',
     '`[CTL-3]`, `[STD-8]`, `[STD-26]`'),
    ('ODR-028', 'a method named like an inherited one replaces it: without `override` over a virtual method it '
                'is `E2111`, over a non-virtual one `E2110`; an `override` is itself virtual (Hardened_9)',
     '`[CLS-4]`'),
    ('ODR-029', '`parse[T]()` reads the whole text strictly (no white space; an optional sign and digits for '
                "integers, Rust's float grammar with `inf`/`nan`, `true`/`false`, one character), and `ParseError` "
                'is `Empty`, `Invalid` or `Overflow` (Hardened_10)',
     '`[TXT-10]`'),
    ('ODR-030', '`extend` is a contextual keyword, a keyword only at the start of an item before the type it '
                'extends, so `Array` can have its `extend` method (Hardened_11)',
     '`[LEX-15]`'),
    ('ODR-031', '`Array`\'s `sort_by(cmp: fn(T, T) -> Ordering)` and `sort_by_key[K: Ord](f: fn(T) -> K)` are '
                'stable, and `f` is called once per element; `windows(n)` yields shared, overlapping views one step '
                'apart, none when `n > len`, and panics on `0`; `drain(r) -> Array[T]` takes any integer range and '
                'panics outside `0..=len` (Hardened_12)',
     '`[STD-15]`'),
    ('ODR-032', '`Map[K, V, H, A]` and `Set[T, H, A]`: the hasher before the allocator; the full method lists '
                'with signatures; `AsKey[K]: Hash` has `is_key` and `to_key`, and every `K: Eq + Hash` is '
                '`AsKey[K]`; owned iteration of a `Map` yields its keys (Hardened_13)',
     '`[STD-11]`, `[STD-12]`, `[STD-16]`, `[ALC-1]`, `[CTL-1]`'),
    ('ODR-033', 'the `Hasher` methods, and users may implement it; `DefaultHasher`\'s values are fixed for one '
                'version on every target; making a `RandomState` is `Nondet`; a panicking `hash` or `eq` aborts; '
                '`Map`\'s constant time is amortised (Hardened_13)',
     '`[HASH-1]`, `[HASH-2]`, `[HASH-3]`, `[DET-2]`, `[STD-11]`'),
    ('ODR-034', 'an empty `Map` prints `{}` and an empty `Set` `set()`; strings `Debug` as Python\'s `repr`; '
                '`Map`/`Set` `==` compares as sets; a repeated literal key keeps its first position and last '
                'value; a list literal is never a `Set`; `sorted(m)` sorts the keys (Hardened_13)',
     '`[TYP-39]`, `[STD-16]`, `[TYP-38]`'),
    ('ODR-035', '`union` is contextual: reserved only where an item would begin `union` and a name, so `Set` '
                'can have its `union` method (Hardened_13)',
     '`[LEX-15]`'),
    ('ODR-036', 'a `Map`\'s keys and values and a `Set`\'s elements are not views (`E3063`); text in a `{…}` '
                'literal with no context is `String` (Hardened_13)',
     '`[STD-11]`, `[TYP-38]`'),
    ('ODR-037', '`std.math.Float`, implemented by `f32` and `f64` only (`E2042`), is the bound of the generic '
                'float functions and provides their operators, typed literals and methods; `PI`, `TAU` and '
                '`E` are untyped constants (Hardened_14)',
     '`[STD-21]`, `[STD-27]`, V.7'),
    ('ODR-038', 'ruled by the owner: `std.math`\'s functions take any number type through `std.math.Number`, '
                'answering in `T.Real` (`f64` for an integer, `f32` for an `f32`); `T.Name` names a type '
                'parameter\'s associated type (Hardened_15)',
     '`[IFC-4]`, `[STD-21]`, `[STD-27]`'),
    ('ODR-039', '`[STD-20]`\'s integer methods: `checked_`, `wrapping_`, `saturating_` and `overflowing_` forms of '
                '`add`, `sub`, `mul`, `floordiv`, `rem`, `pow` and `neg` (the shifts too, but not saturating), '
                'built into the integer types; bit counts are `int`s; a float\'s `MIN` is `-MAX` (Hardened_16)',
     '`[STD-20]`'),
    ('ODR-040', 'the operator interfaces\' methods are `add`, …, `floordiv`, `bitand`, …, `neg` and `not` (a method '
                'name after `fn` and `.`), each `…Assign` form\'s with `_assign`; every number type implements the '
                'interface of each operator it has, in `std.core`; a type has an operator only through its '
                'interface; one `type Output` serves every interface of an `extend` block; a binding is written '
                'only in a bound (Hardened_17)',
     '`[TYP-21]`, `[IFC-4]`, `[LEX-15]`'),
    ('ODR-041', '`f16` is a `Number` whose `Real` is `f32`; every float type, `f16` included, has `INF`, `NAN`, '
                '`EPSILON`, `MAX` and `MIN` (Hardened_18)',
     '`[STD-20]`, `[STD-21]`, `[STD-27]`'),
    ('ODR-042', 'an `extend` parameter that only the implemented interfaces name makes a blanket implementation; a '
                'bound brings its parents; indexing is only through `Index`, `IndexMut` and `IndexSet`, which `Array` '
                'and the views implement through their built-in indexing and `Map` for every `Q: AsKey[K]` (Hardened_19)',
     '`[GRM-34]`, `[IFC-3]`, `[TYP-21]`, `[STD-17]`'),
    ('ODR-043', '`std.math` has the module table\'s vectors, matrices, `Quat`, `Transform` and shapes, with public '
                'components, the operators and the methods graphics libraries agree on: matrices column-major, '
                'projections right-handed with depth in `[0, 1]`, a ray\'s intersections the least `t ≥ 0`; '
                '`KahanSum` is an `f64` accumulator with Neumaier\'s correction (Hardened_20)',
     '`[STD-21]`, `[STD-28]`, `[STD-5]`, `[STD-3]`'),
    ('ODR-044', 'the `FP_CONTRACT` pragma is emitted where the compiler implements it: clang\'s spelling, MSVC\'s own; '
                'gcc, which warns about it, has `-ffp-contract=off` alone (Hardened_20)',
     '`[CG-C-11]`, `[CG-C-1]`'),
    ('ODR-045', 'a type\'s `const` is named `T.NAME` (`Self.NAME` inside it) and is private unless `pub`; constants '
                'name one another in any order, a cycle is `E6001` and a panic while evaluating one `E6004` '
                '(Hardened_20)',
     '`[CT-1]`, `[CT-7]`, `[MOD-2]`'),
    ('ODR-046', '`std.math.det`\'s functions take `f32` or `f64` and answer in it, an `f32` result being the `f64` '
                'one rounded once; each within one unit in the last place (`atan2` 1.3); no integers (Hardened_21)',
     '`[DET-4]`'),
    ('ODR-047', '`NonZero[T]` is `std.core`\'s, read with `get()`, its field and constructor private; `T` is an '
                'integer type, through a private `Integer`; `x // d` and `x % d` take a `NonZero` of `x`\'s type '
                'and answer in it (Hardened_22)',
     '`[STD-4]`, `[TYP-13]`'),
    ('ODR-048', 'an instance of a generic passes a parameter its result may point into as the declaration does '
                '(`Holder[str]`\'s receiver by address, as `Holder[T]`\'s); a parameter an instance\'s type makes a '
                'view is a source of that instance (Hardened_23)',
     '`[BRW-8]`, `[LT-1]`'),
    ('ODR-049', "an associated type's default fills in an implementation that does not state it, meeting its bounds; it is not an equality a bound or a default method body may assume; a cycle of defaults is `E2043` — SP-003 (Hardened_24)",
     '`[IFC-4]`'),
    ('ODR-050', "`Result[T, E]` bounds neither parameter; each operation says what it needs of `E`; `main`'s error prints its chain when it has one — SP-009 (Hardened_24)",
     '`[ERR-1]`, `[ERR-10]`, `[ERR-12]`'),
    ('ODR-051', "a `Copy` value's copy is its bits, retaining each counted handle in it; a struct holding handles may be `Copy`; an `owned` handle argument is the caller's moved, or a retained copy — SP-010 (Hardened_24)",
     '`[PHIL-3]`, `[OWN-7]`, `[STR-3]`, `[FN-9]`'),
    ('ODR-052', "`RwLock[T]` and `SyncShared[T]` are `Sync` for a `Send + Sync` `T`, `Mutex[T]` for a `Send` one; a `@sync` class's fields are `Send` and `Sync`; the marker table agrees with `[THR-8]` — SP-011 (Hardened_24)",
     '`[THR-8]`, `[THR-9]`, `[THR-1]`, `[HEAP-10]`'),
    ('ODR-053', 'a view is what carries a region; a guard is a view that needs drop, and dropping it ends its access — SP-012 (Hardened_24)',
     '`[DRP-6]`, `[TYP-34]`'),
    ('ODR-054', '`@must_drop` is structural: a type owning such a value is one too — SP-015 (Hardened_24)',
     '`[THR-6]`'),
    ('ODR-055', "a float's comparison operators are IEEE in generic code too; `cmp`, sorting, `min` and `max` are total; an algorithm keeps to one relation — SP-018 (Hardened_24)",
     '`[TYP-37]`, `[DRV-1]`'),
    ('ODR-056', "compile time runs `std.math.det`'s functions when the source names them, and never puts one in place of the platform's — SP-019 (Hardened_24)",
     '`[CT-4]`'),
    ('ODR-057', "an operation's effects include what it runs: callbacks, copies, releases, drops, argument evaluation; a generator's by phase; `@nopanic(explicit)` without checks says no modelled panic — SP-020 (Hardened_24)",
     '`[EFF-1]`, `[EFF-16]`, `[STD-18]`, `[STD-19]`, `[CELL-2]`'),
    ('ODR-058', 'a body edit re-checks the callers that use a fact of it that changed: effects, inline bodies, borrow summaries — SP-023 (Hardened_24)',
     '`[BLD-2]`, `[BLD-8]`'),
    ('ODR-059', "`ember fmt` changes layout only; rewriting source is `--migrate`'s — SP-024 (Hardened_24)",
     '`[FMT-1]`'),
    ('ODR-060', 'the formatter writes LF with no manifest switch; `Weak.upgrade` never raises a count from zero; the attribute table lists every attribute; `@assume_noalloc` is an expression inside `unsafe`; `format_to` is variadic — SP-026 (Hardened_24)',
     '`[LEX-2]`, `[RT-10]`, `[EFF-7]`, `[TYP-26]`, `[ATT-1]`'),
    ('ODR-061', "a build's acceptance depends on its semantic inputs; build policy and budgets stop a build as themselves, never as a language error — SP-027 (Hardened_24)",
     '`[PHIL-13]`'),
    ('ODR-062', "the parallel and vectorisation analyses read through calls by the inline header's specified expansion, never by the C compiler's inlining — SP-028 (Hardened_24)",
     '`[PAR-2b]`, `[SIMD-5]`'),
    ('ODR-063', 'an object is deinitialised when its last owner ends as the source says, never earlier; `L3019` is retired — SP-014 (Hardened_24)',
     '`[RC-3]`'),
    ('ODR-064', '`alloc_array` initialises with `Default`, `alloc_zeroed` fills with zeros for a `Zeroable` `T`; a struct is `Zeroable` by `@derive(Zeroable)` — SP-017 (Hardened_24)',
     '`[ARN-3]`, `[ARN-11]`'),
    ('ODR-065', 'a borrow through a handle stored in an object, or a call made on one, goes through a retained copy that lasts to the end of the statement; kept longer it is `E3060`; `mem.drop` moves a handle — review of SP-010 (Hardened_25)',
     '`[RC-5]`, `[EXC-17]`, `[FN-9]`, `[OWN-6]`'),
    ('ODR-066', 'a const argument is any compile-time expression; two are equal by value when known, else by a stated integer normal form (`+`, `-`, `*`; other operations opaque), and not shown equal is a mismatch that says so — SP-007 (Hardened_26)',
     '`[TYP-41]`'),
    ('ODR-067', '`AsKey[K]` only matches; `ToKey[K]: AsKey[K]` makes the key, and `m[q] = v` needs it; every `K: Eq + Hash` is `AsKey[K]`, and `ToKey[K]` when also `Clone` — SP-016 (Hardened_26)',
     '`[STD-12]`, `[STD-17]`'),
    ('ODR-068', '`get_pair_mut(i, j)` on an `Array` or `MutSpan` gives two mutable references, or `None` for an index out of range or equal indices, after checking — SP-031 (Hardened_26)',
     '`[BRW-5]`, `[STD-15]`, `[SPN-5]`'),
    ('ODR-069', 'one storage rule for every owning container: a `Map`, `Set` or `Array` may hold `static` views, checked at each store wherever it happens (through a `mut` parameter or reference, or in a callee, whose callers answer for its parameters); an instance ties a result to a parameter its type names — SP-013 (Hardened_27)',
     '`[TYP-15]`, `[STD-11]`, `[TYP-38]`, `[LT-1]`'),
    ('ODR-070', 'every compiler accepts nesting 256 levels deep and states its own limit (this one 1,024); passing it is `E0112`, never a crash (Hardened_28)',
     '`[GRM-39]`'),
    ('ODR-071', 'the type `()` is `void`; a tuple type has two or more elements (Hardened_29)',
     '`[TYP-27]`'),
    ('ODR-072', 'in a class method `self` is the object the method was called on for the whole call; assigning to it is `E2103`, and re-pointing a caller\'s handle is a `mut` parameter\'s (Hardened_29)',
     '`[CLS-7]`, `[FN-9]`, `[EXC-15]`'),
    ('ODR-073', 'hand-declared pointer facts use `@ffi(param(name, words…), result(words…))`; a `safe fn` exposes the mapped safe Ember type, never a raw pointer (Hardened_30)',
     '`[FFI-10]`, `[FFI-11]`'),
    ('ODR-074', '`count(n)` on a hand-declared safe C function writes `n` at its C ABI position as an integer witness, hides it from the callable signature, and supplies it from a checked span length (Hardened_31)',
     '`[FFI-10]`, `[FFI-11]`'),
    ('ODR-075', '`ref_to_ptr(r)` remains a safe reference-to-raw-pointer conversion, and `r as *T` is safe when it preserves the pointee type and does not strengthen mutability (Hardened_32)',
     '§IV.5, `[TYP-7]`'),
    ('ODR-076', '`null[P]()` takes a complete raw-pointer type and returns that exact pointer type, including its mutability (Hardened_32)',
     '§IV.5'),
    ('ODR-077', 'a safe hand-declared borrowed `count(n)` result requires an input span sharing `n`; its length supplies the hidden C count and result length (Hardened_33)',
     '`[FFI-10]`, `[FFI-11]`'),
    ('ODR-078', 'an `exclusive` borrowed `count(n)` result is `MutSpan[T]` (nullable as `Option[MutSpan[T]]`) through mutable C `T*`, using ODR-077\'s input witness length (Hardened_34)',
     '`[FFI-10]`, `[FFI-11]`'),
    ('ODR-079', 'a nullable hand-declared borrowed `count(n)` input sends `None` as null pointer and zero, but preserves `Some(empty)` with a non-null aligned static sentinel and zero (Hardened_35)',
     '`[FFI-10]`, `[FFI-11]`'),
    ('ODR-080', '`Option[cstr]` has a null-pointer niche; nullable borrowed NUL-terminated C inputs and results use it without changing their lifetime contract (Hardened_36)',
     '`[TYP-13]`, `[FFI-11]`, `[FFI-15]`'),
    ('ODR-081', '`str.to_cstring()` returns `Result[CString, NulError]`, checking for interior NUL before allocation; `.as_cstr()` is an explicit borrow of its owner (Hardened_37)',
     '`[TXT-5]`, `[FFI-15]`'),
    ('ODR-082', '`@ffi(immutable)` asserts a `const` C object for a hand-declared foreign static, with a matching C declaration and safe `Sync` read (Hardened_38)',
     '`[FFI-8]`'),
    ('ODR-083', 'the `creator` thread contract of a static free-function export is anchored to successful module initialization; idempotent init does not transfer it and reinitialization after shutdown establishes a new creator (Hardened_39)',
     '`[FFI-33]`, `[FFI-33c]`, `[GRM-37]`'),
    ('ODR-084', '`#! module name(args)` attaches existing module attributes; overflow is lexical to declarations, defaults, and comptime expressions, while integer methods retain fixed contracts (Hardened_40)',
     '`[GRM-37]`, `[ATT-4]`, `[TYP-8]`, `[TYP-28]`, `[FN-5]`, `[CT-4]`, `[STD-20]`'),
    ('ODR-085', 'an access that is not held is checked but not counted; a `mut self` call to a quiet function is only checked; a check that nothing held anywhere could fail may be removed; each class\'s access words stand side by side before its fields, a zero word after an odd count (Hardened_41)',
     '`[EXC-3]`, `[EXC-19]`, `[OBJ-1]`, `[COST-3]`'),
    ('ODR-086', 'a running total is not grouped; it is vectorised under checked arithmetic when its widths prove it safe or, for a signed total of at least 16 bits, when a run-time test over each block of 64 iterations does; a block that fails the test runs checked, and so does the rest of the loop (Hardened_42)',
     '`[SIMD-5]`, `[SIMD-7]`'),
    ('ODR-087', 'LLVM is restored as the reference implementation\'s second backend over the same MIR, begun once the C backend passes the conformance suite and the default once it also passes the performance suite; `--backend c|llvm` (Hardened_42)',
     '`[CG-LL-1]`, `[CG-LL-2]`, `[CG-LL-3]`, `[SIMD-1]`, `[SIMD-3]`, `[BLD-6]`'),
    ('ODR-088', 'a loop that divides integers by a divisor that is not a constant, or integers of 64 bits or more by a constant that is not a power of two, is not in vectorisable form: no vector instruction set does either, so its overflow checks stay one per operation (Hardened_43)',
     '`[SIMD-5]`, `[SIMD-7]`'),
    ('ODR-089', 'an adapter or consumer takes the iterator it wraps; `take`, `skip`, `nth` and `step_by` count in `int`, and a negative count or index, or a step of zero or less, panics when the adapter is made or the consumer called; `enumerate(start=0)` as `[STD-26]` (Hardened_44)',
     '`[STD-19]`, `[STD-14]`, `[STD-26]`'),
    ('ODR-090', 'a float mode is lexical, as the overflow policy is: a lambda takes its function\'s; a parameter default keeps its declaration\'s and a field default is strict wherever evaluated; inside `@fastmath` no range fact about a float is derived, and integer facts are unaffected; neither mode relaxes range values, `as`, the float classifications, or makes a NaN undefined behaviour (Hardened_45)',
     '`[TYP-9]`, `[TYP-9b]`, `[RNG-4]`, `[CG-C-11]`'),
    ('ODR-091', 'an iterator runs backwards when it implements `DoubleEndedIterator` (`next_back`), whose method `rev` is; `ExactSizeIterator` gives `len`; the ranges with an end and the views\' element iterators are both, and each adapter over them where it can tell its last item (Hardened_46)',
     '`[STD-19]`, `[CTL-3b]`, `[MOD-5]`'),
    ('ODR-092', 'two implementations overlap where one type could be both, each one\'s type parameters standing for any type, found where they are declared; a bound separates two only where the type it bounds is written out in full and does not meet it (Hardened_47)',
     '`[TYP-19]`, `[TYP-20]`'),
    ('ODR-093', 'a range type is `Clone`, `Hash`, `Display` and `Debug` when its representation is, its `Eq` and `Ord` are its representation\'s, and a range value shows as its representation\'s value; it is never `Default` (Hardened_48)',
     '`[RNG-8]`, `[RNG-5a1]`, `[TYP-39]`'),
    ('ODR-096', 'of two instances of one generic interface a call\'s or an operator\'s arguments fit, one whose parameter types are the arguments\' own (an untyped literal as its default type) is chosen before one reached through a coercion, for every type; two that fit equally are `E2070`, and an operator chooses as its method does (Hardened_49)',
     '`[TYP-24]`, `[RNG-5a2]`, `[TYP-21]`'),
    ('ODR-095', 'a range type has every operator its representation implements over itself, `**` and the bitwise operators included, each generated as `[RNG-5a1]`\'s are with the representation as its `Output` (Hardened_50)',
     '`[RNG-5]`, `[RNG-5a1]`'),
    ('ODR-097', 'an implementation may also be written in the package that declares one of its interface\'s arguments, where, reading its type and then those arguments, the first type the package declares comes before any of the implementation\'s type parameters standing alone; anywhere else it is `E2121` (Hardened_50)',
     '`[TYP-20]`'),
    ('ODR-094', 'the owner\'s: `collect[C]()` builds any `C` implementing `std.core.FromIterator[Item]` (`Array`, `Set`, `Map` from pairs, `String` from `char`s and `String`s, and a program\'s types), `C` written or the type the call is expected to give; `peekable` gives `peek`, `peek_mut`, `next_if` and `next_if_eq`; `Option`\'s `as_ref` and `as_mut` reference the payload where it is; `Box[T]` and `Cell[T]` show as the `T` they hold (Hardened_51)',
     '`[STD-19]`, `[ERR-4]`, `[TYP-39]`'),
    ('ODR-098', 'String capacity is measured in bytes; reserve asks for additional bytes; insert takes a byte offset and a char, remove returns that char, truncate keeps a byte prefix, and shrinking retains capacity, with UTF-8 boundary and integer-size checks in every profile (Hardened_52)',
     '`[TXT-11]`, `[TXT-4]`, `[TYP-31]`, `[HEAP-8]`'),
    ('ODR-099', 'the owner\'s direction (2026-10-11), its detail under the delegation: where a program turns a callable type into `extern "C" fn`, a function whose value has that type returns no raw pointer made from a reference to a parameter it borrows by value, which a call from C passes as a copy that ends with the call (`E5042`); followed through copies, conversions and aggregates, not calls; a value read through the parameter and a pointer stored in it are not into the copy',
     '`[FFI-21a]`, `[FN-6]`, `[FN-1]`'),
    ('ODR-100', 'the owner\'s (2026-10-11): the iterator adapters take over the callable and the iterator they keep (`owned`); a lambda written at one keeps the captures `[CLO-2]` infers and the adapter lives no longer than they do; only a consumed callable parameter whose captured state must carry the static region (`thread.spawn`, `jobs.submit`, registries) makes a lambda written there capture by move',
     '`[CLO-15]`, `[CLO-4]`, `[CLO-7]`, `[STD-19]`'),
    ('ODR-101', 'ruled under the delegation (2026-10-11, the owner: "fix that autonomously"): the entry module\'s `main` in a form `[FN-8]` does not list (another parameter or more than one, another return type, type parameters) is `E2183` at its name; the one parameter may have any name and mode, its type `Array[String]`; an `E` that is not `Display` is `E2040`',
     '`[FN-8]`'),
    ('ODR-102', 'ruled under the delegation (2026-10-11, autopilot): a function or capture-free lambda used as a value is a plain `Copy` value that becomes an owned callable value where one is wanted; an owned callable value moves and drops its state, is lent mutably to a borrowed parameter (`E3023` from a place that is not mutable, `E3022` twice at once), is neither `Send` nor `Sync`, equals another calling the same function with the same state, and given to C panics when it holds state; a lambda copies what it only reads (`[CLO-13]`) where it becomes an owned callable value, "assigned after" being later in the body or anywhere in a loop around it',
     '`[CLO-3]`, `[CLO-10]`, `[CLO-13]`, `[CLO-2]`, `[THR-8]`, `[THR-9]`'),
    ('ODR-103', 'ruled under the delegation (2026-10-12, autopilot): a lambda that borrows, stored or sent where no view may go, is shape B9\'s `E3026` (a closure outlives what it captures), help `owned fn` or `thread.scope()`; `E3063` stays for other views',
     '`[CLO-3]`, `[CLO-4]`, `[THR-10]`'),
    ('ODR-104', 'ruled under the delegation (2026-10-12, autopilot): a `return` from a class `init`, like the end of its body, hands the object to its caller, so every field without a default is assigned on that path (`E2100`); `mut self` in a class `init` is accepted and means the same as `self`',
     '`[CLS-2]`, `[STR-6]`'),
    ('ODR-105', 'ruled under the delegation (2026-10-12, autopilot): a `let` field assigned outside its `init` is `E2104`, a code of its own beside `E2103`',
     '`[CLS-9]`'),
    ('ODR-106', 'ruled under the delegation (2026-10-16, the owner: "allow ?.is()", "make it an error"): `is` is a method name after `fn` in a type\'s body, `.` and `?.`, as `not` is (ODR-040), so `AnyError` has `is[E]()`; a function standing alone may take neither `not` nor `is` (`E0005`)',
     '`[LEX-15]`, `[ERR-8]`'),
    ('ODR-107', 'ruled under the delegation (2026-10-16): optional chaining `a?.f`, `a?.m[T](…)` on an `Option`: `None` when `a` is, else the member wrapped in `Some` unless it is an `Option` (then it is the result) or nothing; `a` read as an unconsumed place, a non-`Copy` field lent',
     '`[EXP-10]`'),
    ('ODR-108', 'ruled under the delegation (2026-10-16, the owner: "fix all the stuff yourself"): `[SEL-1]` reports every local, parameter and field by the row its type picks and every type by its kind; a type with no row is a `value`, a `view`, a `callable` or a `raw pointer`',
     '`[SEL-1]`'),
    ('ODR-109', 'ruled under the delegation (2026-10-16): a `Cell` shows a copy of its value, as `get` reads it; a non-`Copy` value that writes its own `Display` or `Debug` is not shown from a `Cell` (`E2040`), since a `Cell` lends its value to nothing',
     '`[TYP-39]`, `[CELL-2]`'),
    ('ODR-110', 'ruled under the delegation (2026-10-16): `[MAN-8]` names the keys of `[c]` (`compiler`, `flags`), `[link]` (`libs`), `[jobs]` (`workers`), `[realtime]` (`contracts`) and `[ffi]` (`evidence`); any other key or section is `E9001`',
     '`[MAN-8]`'),
    ('ODR-111', 'ruled under the delegation (2026-10-16): the marker interfaces are `std.core`\'s, usable as bounds; `unsafe extend T implements Send|Sync:` writes one by hand for a struct or enum of the package (generic ones per their parameters\' marker bounds), `E3100` without `unsafe`; a class only as `@sync`, `Copy` only by `@derive(Copy)`, `Sized` never (`E2040`)',
     '`[TYP-43]`, Part IV §8, grammar `extend_decl`'),
    ('ODR-112', 'ruled under the delegation (2026-10-16): an extension may target a parameter standing alone with blanket parameters besides (`extend[T, U] T implements Wrap[U]`); a call leaving a blanket parameter open takes it from the one instance of the interface the receiver\'s type implements through the block',
     '`[GRM-34]`'),
    ('ODR-113', 'ruled by the owner (2026-10-16): `[STD-9]`\'s example of a type with neither `Display` nor `Debug` is a `RefCell`, not a class: a class handle always has `Debug`, its class and address (`[TYP-36]`)',
     '`[STD-9]`'),
    ('ODR-114', 'ruled by the owner (2026-10-16): a moved-from place, read, moved again or borrowed, is `E3040` with `[OWN-3]`\'s help; `E3050` is a place uninitialised on some path, one never assigned there',
     '`[OWN-3]`, `[BRW-7]`'),
    ('ODR-115', 'ruled by the owner (2026-10-16): `unsafe fn(…) -> R` is a callable type whose call is an `unsafe` operation; an `unsafe fn` coerces only to one, any function to one; as a bound it is met by any function',
     '`[FN-6]`, `[UNS-1]`, grammar `fn_type`'),
    ('ODR-116', 'ruled by the owner (2026-10-16): a `MutSpan[T]` converts to `Span[T]` where one is expected, a shared reborrow; `[TYP-5]`\'s complete list lacked it, and Part XII.1\'s `integrate(ps.pos, ps.vel, …)` passes a mutable column to a `Span` parameter',
     '`[TYP-5]` rule 6, `[SPN-3]`, `[SOA-2]`'),
]

H5_HEAD = """
## H.5 Changes from 0.9.9_Hardened_2

Each ambiguity found while implementing 0.9.9 is an owner decision request (`docs/OWNER-QUEUE.md`),
ruled under the owner's delegation and recorded here. Hardened_3 carries ODR-021 and ODR-022;
Hardened_4 adds ODR-023; Hardened_5 adds ODR-024; Hardened_6 adds ODR-025; Hardened_7 adds ODR-026;
Hardened_8 adds ODR-027; Hardened_9 adds ODR-028;
Hardened_10 adds ODR-029; Hardened_11 adds ODR-030; Hardened_12 adds ODR-031; Hardened_13 adds ODR-032 to ODR-036;
Hardened_14 adds ODR-037; Hardened_15 adds ODR-038; Hardened_16 adds ODR-039; Hardened_17 adds ODR-040;
Hardened_18 adds ODR-041; Hardened_19 adds ODR-042; Hardened_20 adds ODR-043 to ODR-045;
Hardened_21 adds ODR-046; Hardened_22 adds ODR-047; Hardened_23 adds ODR-048; Hardened_24 adds
ODR-049 to ODR-064, the owner's simplification pass (`docs/proposals/Ember_Simplification_Pass_Revised.md`);
Hardened_25 adds ODR-065, the review of SP-010's access question; Hardened_26 adds ODR-066 to
ODR-068, the owner's Part II adoptions SP-007, SP-016 and SP-031; Hardened_27 adds ODR-069, SP-013;
Hardened_28 adds ODR-070, the nesting limit (D-331); Hardened_29 adds ODR-071 and ODR-072, found
closing the last open defects (D-355, D-202); Hardened_30 adds ODR-073, the hand-declared C pointer
contract syntax and safe surface; Hardened_31 adds ODR-074, the ABI witness for a hand-declared
counted pointer; Hardened_32 adds ODR-075 and ODR-076, the safe reference-to-pointer spellings and
the exact pointer type of `null[P]()`; Hardened_33 adds ODR-077, the input length source for a
borrowed counted C pointer result; Hardened_34 adds ODR-078, the exclusive counted result's
mutable view and C pointer; Hardened_35 adds ODR-079, the nullable counted input's distinction
between absent and present-empty views; Hardened_36 adds ODR-080, nullable borrowed C strings and
their null-pointer niche; Hardened_37 adds ODR-081, the fallible owned C-string conversion and
explicit borrowed view; Hardened_38 adds ODR-082, the C qualifier asserted by an immutable
foreign static; Hardened_39 adds ODR-083, the initialization anchor for static export creator
contracts; Hardened_40 adds ODR-084, module-attribute syntax and lexical integer overflow policy;
Hardened_41 adds ODR-085, what the dynamic exclusivity checks cost and the packed access words;
Hardened_42 adds ODR-086 and ODR-087, running totals checked block by block and the LLVM backend;
Hardened_43 adds ODR-088, integer division no vector instruction set does keeps a loop out of
vectorisable form; Hardened_44 adds ODR-089, the iterator adapters' counts and ownership;
Hardened_45 adds ODR-090, where a float mode holds; Hardened_46 adds ODR-091, which iterators
run backwards; Hardened_47 adds ODR-092, when two implementations overlap; Hardened_48 adds
ODR-093, what a range type implements; Hardened_49 adds ODR-096, how a call chooses between two
instances of one generic interface; Hardened_50 adds ODR-095, which operators a range value has,
and ODR-097, where an implementation may be written; Hardened_51 adds ODR-094, the owner's
ruling on `collect`'s target, `peekable`'s methods, and how `Box` and `Cell` show; Hardened_52 adds
ODR-098, the byte offsets, UTF-8 checks and capacity contracts of `String` mutation methods.
0.9.10_Hardened_3 adds ODR-099, a pointer into a parameter C passes as a copy (§H.8);
0.9.10_Hardened_4 adds ODR-100, what a lambda written at an adapter captures (§H.9);
0.9.10_Hardened_5 adds ODR-101, a `main` of another form (§H.10);
0.9.10_Hardened_6 adds ODR-102, owned callable values (§H.11);
0.9.10_Hardened_7 adds ODR-103, a lambda outliving its captures is `E3026` (§H.12);
0.9.10_Hardened_8 adds ODR-104, a constructor's `return` and its receiver (§H.13);
0.9.10_Hardened_9 adds ODR-105, `E2104` for a `let` field assigned outside `init` (§H.14);
0.9.10_Hardened_10 adds ODR-106 to ODR-112, the owner's `is` and `?.` and the agents' open
questions (§H.15); 0.9.10_Hardened_11 adds ODR-113 to ODR-115, the owner's rulings on what printing,
a use after a move and an `unsafe fn` value are (§H.16); 0.9.10_Hardened_12 adds ODR-116, a
`MutSpan` where a `Span` is expected (§H.17).

| ODR | Ruling | Rules |
|---|---|---|
"""


def h2_to_now():
    head = H5_HEAD + '\n'.join(f'| {o} | {r} | {rules} |' for o, r, rules in ODRS)
    diff = rule_diff(H2_PARTS, H52_PARTS)
    return head + '\n\nGenerated: every rule whose text differs from Hardened_2 in Hardened_52, the last\n' \
        + '0.9.9.\n\n| Rule | H2 → Hardened_52 |\n|---|---|\n' + diff


# 0.9.10_Hardened_1: the owner's design G8-4, a new language version over 0.9.9_Hardened_52;
# 0.9.10_Hardened_2: the owner's rulings of 2026-10-06 on three of the audit's decisions.
CURRENT = ('0.9.10', 12)
H52_PARTS = os.path.join(HERE, '..', 'parts-h52')
V010_H1_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h1')
V010_H2_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h2')
V010_H3_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h3')
V010_H4_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h4')
V010_H5_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h5')
V010_H6_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h6')
V010_H7_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h7')
V010_H8_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h8')
V010_H9_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h9')
V010_H10_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h10')
V010_H11_PARTS = os.path.join(HERE, '..', 'parts-0.9.10-h11')

H6_HEAD = """
## H.6 Changes in 0.9.10 (from 0.9.9_Hardened_52)

**0.9.10_Hardened_1** is the owner's design G8-4 (`docs/proposals/G8-4-counts-and-positions.md`,
decided 2026-10-05, built with `docs/MIGRATION-0.9.10.md`). It changes what programs are accepted,
so it is a new language version (`[VER-9]`):

* Every iterator counts and numbers its items in a type that holds every count and number it can
  have: `Iterator.Count` and `Iterator.Position`, `int` unless the iterator says otherwise; ranges
  of 64-bit numbers count in `u64` (`..`) and `u128` (`..=`) and number in `i128`; ranges of
  128-bit numbers count in `u128` (`..`) and `u256` (`..=`) and number in `i256` (`[STD-19]`).
  This replaces ODR-089's "counts are `int`s" for what Ember hands back (what a caller passes stays
  an `int`) and ODR-091's backwards-`enumerate` panic.
* `chain`, `flatten` and `flat_map` step up only as far as their counts need (`u256` the limit).
* The visible-numbers rule: where a range's numbers are written in place, `enumerate`, `len` and
  `count` give the smallest type that holds the values (`[STD-19]`, `[STD-26]`).
* `skip_front`, `skip_back` and `nth_back` leave out items at once through the adapters, so none
  passes a gigantic run of items one at a time (`[STD-19]`).
* `u256` and `i256`, the 256-bit counts (`[TYP-42]`).
* `a..` gives its type's maximum and panics only when the value after it is asked for, in a `for`
  and through its iterator, which leaves values out at once (`[CTL-3]`, D-526; ODR-027 said the
  loop overflowed on reaching the maximum).
* `for i in a..=T.MAX` ends after `T.MAX`, as `[CTL-3]` said (D-525: it ran forever).
* An index or a size wider than `usize` that `usize` does not hold is out of bounds (`[TYP-31]`,
  D-527: its low bits were used).
* `i256` and `u256` implement `Add`, `Sub`, `Neg`, `AddAssign` and `SubAssign` (ODR-040); `parse`
  does not read them (`[TYP-42]`).
* A loop numbering in a type wider than 64 bits has a copy computing the numbers as `int`s, taken
  when every number fits one (`[CTL-3b]`, the owner's decision C).
* A program that put such a count or number into an `int` converts it (`as int`, or `to_int()` in
  generic code); `docs/MIGRATION-0.9.10.md` lists the cases.

Generated: every rule whose text differs from Hardened_52.

| Rule | Hardened_52 → 0.9.10_Hardened_1 |
|---|---|
"""


def h52_to_now():
    return H6_HEAD + rule_diff(H52_PARTS, V010_H1_PARTS)


H7_HEAD = """
## H.7 Changes in 0.9.10_Hardened_2 (from 0.9.10_Hardened_1)

**0.9.10_Hardened_2** records the owner's rulings of 2026-10-06 on three decisions of the audit of
the delegated rulings, and on the type of `enumerate`'s numbers in a function written for any type
(`docs/DECISIONS.md`, ADR-140). It changes what programs are accepted
(`docs/MIGRATION-0.9.10.md`):

* `parse` reads `u256` and `i256` over their whole range, with the statuses the narrower integers
  give; `Overflow` only past the real top or below the real bottom (`[TXT-10]`, `[TYP-42]`). D-528
  had refused them (`E2020`).
* The known-numbers rule replaces the visible-numbers rule: `enumerate`'s numbers, and `len` and
  `count`, are in the smallest type that holds every value, read from the numbers the compiler
  knows: numbers written as numbers, names set once and never changed, a loop's counter,
  arithmetic on known numbers, and lengths (`[STD-19]`, `[STD-26]`). Before, a variable counted as
  any value of its type.
* `enumerate` numbers in a type that holds every number it can give, stepping up from its
  iterator's `Position` where the numbers could pass that type's top, so no number of an
  `enumerate` overflows; a counted loop computes its numbers with no check (`[STD-19]`,
  `[CTL-3b]`). This replaces G8-4 decision F (a number past `int`'s top panicked) and ODR-091's
  backwards panic.
* A whole number converts by itself to another whole-number type where the compiler knows every
  value it can have fits, at a coercion site and in an operator (`[TYP-5]` rule 12, `[TYP-4]`):
  after `n = 5`, `y: u64 = n` is accepted. Floats, `bool` and `char` keep their rules.
* In a function written for any type, where the type of `enumerate`'s numbers depends on the item
  type, that type and the lines using the numbers are checked for each use, which numbers by what
  it knows; a use a line does not fit is refused there, naming it (`[TYP-17]`, `[STD-19]`).

Generated: every rule whose text differs from 0.9.10_Hardened_1.

| Rule | 0.9.10_Hardened_1 → 0.9.10_Hardened_2 |
|---|---|
"""


def v010_h1_to_h2():
    return H7_HEAD + rule_diff(V010_H1_PARTS, V010_H2_PARTS)


H8_HEAD = """
## H.8 Changes in 0.9.10_Hardened_3 (from 0.9.10_Hardened_2)

**0.9.10_Hardened_3** records ODR-099, the owner's choice of 2026-10-11 (`docs/OWNER-QUEUE.md`,
`docs/DECISIONS.md` ADR-151). It changes what programs are accepted:

* Where a program turns a callable type into `extern "C" fn`, a function whose value has that type
  returns no raw pointer into a parameter it borrows by value: C passes the argument as a copy that
  ends when the call returns (`[FFI-21a]`, `E5042`). The same function called from Ember borrows
  the caller's value, as before.

Generated: every rule whose text differs from 0.9.10_Hardened_2.

| Rule | 0.9.10_Hardened_2 → 0.9.10_Hardened_3 |
|---|---|
"""


def v010_h2_to_h3():
    return H8_HEAD + rule_diff(V010_H2_PARTS, V010_H3_PARTS)


H9_HEAD = """
## H.9 Changes in 0.9.10_Hardened_4 (from 0.9.10_Hardened_3)

**0.9.10_Hardened_4** records ODR-100, the owner's choice of 2026-10-11 (`docs/OWNER-QUEUE.md`,
`docs/DECISIONS.md` ADR-153). It changes what programs are accepted:

* The iterator adapters take over the callable and the other iterator they keep (`owned`); a named
  iterator given to `zip` or `chain` is moved (`[STD-19]`).
* A lambda written at an adapter keeps the captures `[CLO-2]` infers; the adapter lives no longer
  than they do. Only a consumed callable parameter whose captured state must carry the static region
  (`thread.spawn`, `jobs.submit`, registries) makes a lambda written there capture by move
  (`[CLO-15]`, `[CLO-4]`, `[CLO-7]`).

Generated: every rule whose text differs from 0.9.10_Hardened_3.

| Rule | 0.9.10_Hardened_3 → 0.9.10_Hardened_4 |
|---|---|
"""


def v010_h3_to_h4():
    return H9_HEAD + rule_diff(V010_H3_PARTS, V010_H4_PARTS)


H10_HEAD = """
## H.10 Changes in 0.9.10_Hardened_5 (from 0.9.10_Hardened_4)

**0.9.10_Hardened_5** records ODR-101, ruled under the owner's delegation on 2026-10-11
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-157). It changes what programs are accepted:

* The entry module's `main` in a form `[FN-8]` does not list is `E2183`, reported at its name: another
  parameter than one `Array[String]` (any name, any mode), more than one, another return type than
  `void` or `Result[void, E]`, or type parameters. An `E` that is not `Display` is `E2040`, an unmet
  bound. Such a `main` used to be accepted and run wrongly (D-551).

Generated: every rule whose text differs from 0.9.10_Hardened_4.

| Rule | 0.9.10_Hardened_4 → 0.9.10_Hardened_5 |
|---|---|
"""


def v010_h4_to_h5():
    return H10_HEAD + rule_diff(V010_H4_PARTS, V010_H5_PARTS)


H11_HEAD = """
## H.11 Changes in 0.9.10_Hardened_6 (from 0.9.10_Hardened_5)

**0.9.10_Hardened_6** records ODR-102, ruled under the owner's delegation on 2026-10-11 while building
owned callable values (`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-159, D-554). It changes what
programs are accepted:

* A function, or a lambda that captures nothing, used as a value is a plain `Copy` value; it becomes
  an owned callable value where one is wanted. An owned callable value moves and drops its state
  (`[CLO-3]`).
* An owned callable value given to a borrowed parameter is lent mutably: from a place that is not
  mutable it is `E3023`, and the same value given twice to one call is `E3022` (`[CLO-2]`, `[CLO-3]`).
* An owned callable value is neither `Send` nor `Sync`; two are equal when they call the same function
  with the same state (`[CLO-10]`). Converting one to `extern "C" fn` reads its C entry without moving
  it, and panics if it holds captured state (`[CLO-3]`).
* A lambda copies what it only reads where it becomes an owned callable value; "assigned after" is an
  assignment later in the body or anywhere in a loop around it (`[CLO-13]`).

Generated: every rule whose text differs from 0.9.10_Hardened_5.

| Rule | 0.9.10_Hardened_5 → 0.9.10_Hardened_6 |
|---|---|
"""


def v010_h5_to_h6():
    return H11_HEAD + rule_diff(V010_H5_PARTS, V010_H6_PARTS)


H12_HEAD = """
## H.12 Changes in 0.9.10_Hardened_7 (from 0.9.10_Hardened_6)

**0.9.10_Hardened_7** records ODR-103, ruled under the owner's delegation on 2026-10-12
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-161). It changes a diagnostic's code only:

* A lambda that borrows, stored where no view may go (an owned callable value, a list, a class
  field) or sent to a thread, is `E3026`, shape B9 (a closure outlives what it captures), with the
  help `owned fn` or `thread.scope()`; it was `E3063`, which stays for every other view
  (`[CLO-3]`, `[CLO-4]`, `[THR-10]`).

Generated: every rule whose text differs from 0.9.10_Hardened_6.

| Rule | 0.9.10_Hardened_6 → 0.9.10_Hardened_7 |
|---|---|
"""


def v010_h6_to_h7():
    return H12_HEAD + rule_diff(V010_H6_PARTS, V010_H7_PARTS)


H13_HEAD = """
## H.13 Changes in 0.9.10_Hardened_8 (from 0.9.10_Hardened_7)

**0.9.10_Hardened_8** records ODR-104, ruled under the owner's delegation on 2026-10-12
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-165):

* A `return` from a class `init`, like the end of its body, hands the object to its caller, so every
  field without a default is assigned on that path (`E2100`) (`[CLS-2]`).
* `mut self` is accepted in a class `init` and means the same as `self`, as for a struct (`[CLS-2]`,
  `[STR-6]`).

It also corrects the overview's `Emitter` example, which moved a borrowed `String` into a field
(`[FN-1]`): its parameter is `owned name: String`.

Generated: every rule whose text differs from 0.9.10_Hardened_7.

| Rule | 0.9.10_Hardened_7 → 0.9.10_Hardened_8 |
|---|---|
"""


def v010_h7_to_h8():
    return H13_HEAD + rule_diff(V010_H7_PARTS, V010_H8_PARTS)


H14_HEAD = """
## H.14 Changes in 0.9.10_Hardened_9 (from 0.9.10_Hardened_8)

**0.9.10_Hardened_9** records ODR-105, ruled under the owner's delegation on 2026-10-12
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-166). It gives a diagnostic a code only:

* A `let` field assigned outside its `init` is `E2104` (`[CLS-9]`); the code table gave the rule
  none.

Generated: every rule whose text differs from 0.9.10_Hardened_8.

| Rule | 0.9.10_Hardened_8 → 0.9.10_Hardened_9 |
|---|---|
"""


def v010_h8_to_h9():
    return H14_HEAD + rule_diff(V010_H8_PARTS, V010_H9_PARTS)


H15_HEAD = """
## H.15 Changes in 0.9.10_Hardened_10 (from 0.9.10_Hardened_9)

**0.9.10_Hardened_10** records ODR-106 to ODR-112, ruled under the owner's delegation on 2026-10-16
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-191):

* `is` is a method name after `fn` in a type's body, `.` and `?.`, as `not` is; a function standing
  alone may take neither (`[LEX-15]`, ODR-106).
* Optional chaining is `[EXP-10]`: `None` propagates, an `Option` member is the result, a field that
  is not `Copy` is lent (ODR-107).
* What a declaration is, and what a type with no row of IX.0's table is reported as, in
  `[SEL-1]` (ODR-108).
* A `Cell` shows a copy of its value; a non-`Copy` value with its own `Display` or `Debug` is not
  shown from one (`[TYP-39]`, ODR-109).
* The keys of `[c]`, `[link]`, `[jobs]`, `[realtime]` and `[ffi]`, in `[MAN-8]` (ODR-110).
* The marker interfaces are `std.core`'s and usable as bounds; `Send` and `Sync` are written by
  hand with `unsafe extend` (`[TYP-43]`, the grammar's `extend_decl`, ODR-111).
* An extension over a parameter standing alone may have blanket parameters, taken at a call from
  the receiver's one instance of the interface (`[GRM-34]`, ODR-112).

Generated: every rule whose text differs from 0.9.10_Hardened_9.

| Rule | 0.9.10_Hardened_9 → 0.9.10_Hardened_10 |
|---|---|
"""


def v010_h9_to_h10():
    return H15_HEAD + rule_diff(V010_H9_PARTS, V010_H10_PARTS)


H16_HEAD = """
## H.16 Changes in 0.9.10_Hardened_11 (from 0.9.10_Hardened_10)

**0.9.10_Hardened_11** records ODR-113 to ODR-115, ruled by the owner on 2026-10-16
(`docs/OWNER-QUEUE.md`, `docs/DECISIONS.md` ADR-198):

* The example of a type with neither printer is a `RefCell`: a class handle always has `Debug`
  (`[STD-9]`, `[TYP-36]`, ODR-113).
* A moved-from place, read, moved again or borrowed, is `E3040`; `E3050` is a place never assigned on
  some path (`[OWN-3]`, `[BRW-7]`, ODR-114).
* `unsafe fn(…) -> R` is a callable type whose call needs `unsafe`; an `unsafe fn` coerces only to one
  (`[FN-6]`, `[UNS-1]`, the grammar's `fn_type`, ODR-115).

Generated: every rule whose text differs from 0.9.10_Hardened_10.

| Rule | 0.9.10_Hardened_10 → 0.9.10_Hardened_11 |
|---|---|
"""


def v010_h10_to_h11():
    return H16_HEAD + rule_diff(V010_H10_PARTS, V010_H11_PARTS)


H17_HEAD = """
## H.17 Changes in 0.9.10_Hardened_12 (from 0.9.10_Hardened_11)

**0.9.10_Hardened_12** records ODR-116, ruled by the owner on 2026-10-16 (`docs/OWNER-QUEUE.md`,
`docs/DECISIONS.md` ADR-200):

* A `MutSpan[T]` converts to `Span[T]` where one is expected, a shared reborrow of the same elements.
  `[TYP-5]` gives its list as complete and lacked it, while Part XII.1's `integrate(ps.pos, ps.vel, …)`
  passes a mutable column to a `Span` parameter (`[TYP-5]` rule 6, `[SPN-3]`, `[SOA-2]`, ODR-116).

Generated: every rule whose text differs from 0.9.10_Hardened_11.

| Rule | 0.9.10_Hardened_11 → 0.9.10_Hardened_12 |
|---|---|
"""


def v010_h11_to_now():
    return H17_HEAD + rule_diff(V010_H11_PARTS, os.path.join(HERE, '..', 'parts'))


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


def header_lines(n):
    """The front matter's Version and Supersedes lines: a 0.9.10 hardening supersedes the earlier
    0.9.10 ones and every 0.9.9 hardening, the last of them `n`."""
    version, hardening = CURRENT
    earlier = ', '.join([f'{version}_Hardened_{k}' for k in range(hardening - 1, 0, -1)]
                        + [f'0.9.9_Hardened_{k}' for k in range(n, 0, -1)])
    return (f'**Version:** {version}_Hardened_{hardening}',
            f'**Supersedes:** {earlier}, 0.9.8_Hardened_3 (development target) and 0.8.5_Hardened_1 '
            '(adopted). This document is')


def write_header():
    """The header names the newest hardening in ODRS. It was written by hand, and every cut from
    Hardened_8 to Hardened_13 went out still calling itself Hardened_7; now each cut writes it."""
    latest = max(int(n) for _, text, _ in ODRS for n in re.findall(r'\(Hardened_([0-9]+)\)', text))
    front = os.path.join(HERE, '..', 'parts', 'p00-front.md')
    text = open(front, encoding='utf-8').read()
    version, supersedes = header_lines(latest)
    text, a = re.subn(r'^\*\*Version:\*\* .*$', version, text, count=1, flags=re.M)
    text, b = re.subn(r'^\*\*Supersedes:\*\* .* This document is$', supersedes, text, count=1, flags=re.M)
    assert a == 1 and b == 1, (a, b)
    open(front, 'w', encoding='utf-8', newline='\n').write(text)
    return latest


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
        HEAD + '\n'.join(rows) + '\n' + H4_HEAD + h4 + '\n' + h2_to_now() + '\n' + h52_to_now() + '\n'
        + v010_h1_to_h2() + '\n' + v010_h2_to_h3() + '\n' + v010_h3_to_h4() + '\n' + v010_h4_to_h5() + '\n'
        + v010_h5_to_h6() + '\n' + v010_h6_to_h7() + '\n' + v010_h7_to_h8() + '\n' + v010_h8_to_h9() + '\n' + v010_h9_to_h10() + '\n' + v010_h10_to_h11() + '\n' + v010_h11_to_now() + '\n')
    latest = write_header()
    print('retired ids', len(gone), 'families', len(fam), 'families without a reason', missing,
          'H1->H2 rows', h4.count('\n') + 1, 'header %s_Hardened_%d over 0.9.9_Hardened_%d' % (CURRENT + (latest,)))
    return gone

if __name__ == '__main__':
    main()
