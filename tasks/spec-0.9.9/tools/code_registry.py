"""The diagnostic code registry of 0.9.9 (`[DIA-6a]`), generated into parts/p17a-codes.md.

Source: the 0.9.8-era compiler registry (compiler/ember_diag/src/codes.rs), with 0.9.9's changes
applied (OVERRIDE), retirements (RETIRED) and new codes (NEW). `--check` reports:
  - codes named in the parts but absent from the registry;
  - live registry rows whose rule references are not defined in 0.9.9.
"""
import re, os, glob, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, '..', '..', '..')
PARTS = os.path.join(HERE, '..', 'parts')

# code -> (title, [rules]) for rows whose 0.9.9 meaning or rule reference differs from the registry
OVERRIDE = {
    'E0006': ('a `#!` language directive names a version other than the current one', ['VER-8']),
    'E0100': ('unexpected token', ['GRM-2']),
    'E0101': ('unclosed delimiter', ['LEX-6']),
    'E0102': ('membership test chained (`a in b in c`)', ['GRM-23']),
    'E0104': ('attribute or construct not admitted here', ['ATT-1', 'PHIL-12']),
    'E0108': ('attribute is not permitted on this statement', ['GRM-20']),
    'E2030': ('`@view` on a type that is not a view', ['TYP-34']),
    'E2130': ('a `const` of a type that owns heap memory; use a `static`', ['TYP-1']),
    'E6001': ("comptime evaluation exceeded its limits, or a constant's value depends on itself", ['CT-3']),
    'E2131': ('array length must be a constant', ['CT-1']),
    'E2150': ('`is` on an operand that is neither a handle, a reference nor an `Option`', ['EXP-9']),
    'E2151': ('integer `**` with a constant negative exponent', ['TYP-30']),
    'E2170': ('reference to a field of a packed struct', ['LAY-2']),
    'E2200': ('type has infinite size (a recursive value type without `Box`)', ['TYP-14']),
    'E2228': ('callable parameter-mode mismatch (shape B15)', ['CLO-3']),
    'E3023': ('two writers of one value (shape B4)', ['BRW-1']),
    'E3024': ('a struct field would borrow another field of the same struct (shape B5)', ['TYP-15']),
    'E3025': ('a method takes all of `self` (shape B8)', ['BRW-10']),
    'E3026': ('a closure outlives what it captures (shape B9)', ['CLO-4']),
    'E3027': ('a `mut` argument is not a mutable place (shape B10)', ['FN-2a']),
    'E3062': ('returned view does not derive from a parameter (shape B6)', ['LT-1']),
    'E2031': ('`@borrows` names a parameter the result cannot borrow from (a borrowed `Copy` '
              'parameter, or an `owned` one that is not a reference or view)', ['LT-1a']),
    'E3060': ('borrowed value does not live long enough', ['LT-3', 'BRW-8']),
    'E3065': ('multi-region result provenance cannot be inferred', ['LT-35']),
    'E3095': ('disjointness is not establishable for these operands', ['DSJ-1']),
    'E3105': ('`UnsafeCell` in `@static_safe` code', ['UNS-10']),
    'E4030': ('`@static_safe` function performs a dynamically checked access', ['EFF-12']),
    'E5002': ('foreign call requires `unsafe`: its contract is incomplete, or `safe fn` on a function whose contract is', ['FFI-2', 'FFI-10']),
    'E5030': ('`std::function` as a parameter', ['FFI-17a']),
    'E5031': ('C++ parameter cannot be mapped', ['FFI-17']),
    'E5032': ('opaque C++ type may not be constructed', ['FFI-32']),
    'E5034': ('unsupported C++ construct', ['FFI-17']),
    'E5050': ('a foreign fact claims a grade whose evidence is absent or stale', ['TCB-1']),
    'E5051': ('a foreign callee retains a pointer its contract does not declare `retained`', ['FFI-35a']),
    'E5053': ('an instrumented run contradicted a declared foreign effect', ['FFI-37']),
    'E5054': ('range type in a foreign signature', ['RNG-10']),
    'E5055': ('an Ember generic may not instantiate a C++ template', ['FFI-17b']),
    'E5056': ('override of a C++ virtual the overlay does not name', ['FFI-39']),
    'E5057': ('the overlay names a method that is not virtual in the header', ['FFI-39']),
    'E5058': ('C++ base has no default constructor and no declared `init`', ['FFI-39']),
    'E5059': ('C++ trampoline base has no virtual destructor', ['FFI-39']),
    'E5060': ('upcast in a context that cannot hold the `Retained` token', ['FFI-39']),
    'E5061': ('imported C++ function has no exception policy and none can be derived', ['FFI-24']),
    'E5062': ('contradictory C++ exception policies', ['FFI-43']),
    'E5064': ('interior NUL in a value converted to `cstr`', ['FFI-15']),
    'E5065': ('`Shared`/`Weak` and `CppShared`/`CppWeak` do not interconvert', ['WK-14']),
    'E5090': ('inline assembly is not supported by the C backend', ['UNS-6']),
    'E6010': ('operation not available at compile time', ['CT-2']),
    'E7001': ('`@sync` class with a field that is not `Sync`, or a non-`@sync` base or derived class', ['THR-1']),
    'E8001': ('GPU layout does not match the CPU layout', ['GPU-10']),
    'E9001': ('invalid manifest, including an unknown key', ['MAN-1', 'PRF-3']),
    'E9002': ('no C compiler found, or it failed; the help names the host\'s remedy', ['BLD-FFI-1']),
    'E9003': ('invalid command line', ['CLI-1']),
    'E9010': ('`[lints]` names a lint the compiler does not define', ['MAN-3']),
    'E9011': ('the toolchain cannot disable floating-point contraction', ['CG-C-11']),
    'E9020': ('translation units of one target disagree on an inherited flag', ['BLD-FFI-1a']),
    'E9021': ('C++ standard library and CRT heap could not be determined', ['BLD-FFI-1b']),
    'E9013': ('invalid `[ffi]` manifest section', ['TCB-1']),
    'E9030': ('hot reload refused', ['HR-18']),
    'E9031': ('invalid `reload` value', ['MAN-7']),
    'E9033': ('`reload` is forbidden in `shipping`', ['PRF-2']),
    'E9034': ('invalid `max_instantiations` value', ['MONO-3']),
    'E9035': ('packages in one process disagree about the object-header layout', ['HR-12a']),
    'E9037': ('reload ABI mismatch on image load', ['ABI-1']),
    'W1002': ('binding shadows an enum variant of the same name', ['GRM-12']),
    'W2015': ('float literal has more digits than its type keeps', ['LEX-17a']),
    'W2091': ('unreachable match arm', ['CTL-5']),
    'W2111': ('`virtual` has no effect in a final class', ['CLS-4']),
    'W5031': ('C++ declaration skipped: the header could not be parsed', ['FFI-20a']),
    'W5033': ('`&&`-qualified member skipped', ['FFI-40']),
    'W5034': ('anonymous-namespace entity skipped', ['FFI-41']),
    'W5050': ('unbacked or stale grade', ['TCB-1']),
    'W5054': ('unexercised foreign fact', ['FFI-37']),
    'W5002': ('declaration not imported (unsupported convention or construct)', ['FFI-20a']),
    'W9030': ('reload requires a restart', ['HR-18']),
    'L2001': ('unnecessary clone', ['LNT-6']),
    'L2002': ('large `Copy` value passed by value', ['LNT-6']),
    'L2003': ('fallible construction where a total one exists', ['RNG-3a']),
    'L3002': ('borrow held longer than necessary', ['LNT-6']),
    'L3010': ('`unsafe` block larger than necessary', ['UNS-3']),
    'L3014': ('return region is the intersection of several parameters', ['LT-1b']),
    'L3015': ('undocumented unsafe obligation', ['UNS-7']),
    'L3016': ('`@safety` text still reads `TODO`', ['UNS-7']),
    'L4001': ('allocation in a hot loop', ['LNT-6']),
    'L4002': ('dynamic dispatch on a final type', ['LNT-6']),
    'L5001': ('`unsafe extern` declaration with no contract', ['LNT-6']),
    'L5002': ('conversion at the FFI boundary copies', ['LNT-6']),
    'L7001': ('lock held across a call that may block', ['LNT-6']),
    'W0001': ('reserved (0.9.8 dangling doc comment; now discarded in silence)', ['LEX-11']),
}

RETIRED = {
    'E0007': 'no lifetime syntax exists; a stray `\'` is an unterminated character literal (`E0008`)',
    'E0105': 'kept: `;` is not a statement separator',  # not retired; see LIVE_KEEP
    'E1040': 'glob imports are allowed from any module (`[MOD-8]`)',
    'E2224': 'folded into `E2225`: `migrate_from` sees a read-only view',
    'E2227': 'folded into `[HR-35]`: foreign failure becomes `ReloadError.Foreign`',
    'E3064': 'multi-region view structs are legal',
    'E4071': 'an undeclared `extern` is `Nondet` (`[DET-2]`), reported as `E4070`',
    'E4073': 'generator frames never allocate (`[CORO-1]`)',
    'E6002': 'compile-time evaluation is deterministic by construction (`[CT-4]`)',
    'E9012': 'reserved',
    'L3018': 'the reason category is part of the `# SAFETY(…):` note and optional (`[UNS-8]`)',
    'L3019': 'an object is never deinitialised before its last owner ends (`[RC-3]`, ODR-063)',
    'E9032': 'reserved',
    'W0001': 'a dangling doc comment is discarded in silence (`[LEX-11]`)',
}
RETIRED.pop('E0105')
OVERRIDE.pop('W0001')

NEW = {
    'E0008': ('unterminated character literal', ['LEX-22']),
    'E0110': ('function declared without a body outside an interface, extern block or abstract class', ['GRM-33']),
    'E0111': ('`ref` of an expression that is not a place', ['GRM-36']),
    'E0900': ('construct not implemented by this compiler', ['PHIL-12', 'CLI-19']),
    'E0901': ('construct this specification leaves unspecified', ['PHIL-12']),
    'E1031': ('a name bound by two glob imports is used', ['MOD-8']),
    'E1052': ('item, field or constructor not visible here', ['MOD-2']),
    'E1060': ('no module of that name', ['MOD-3']),
    'E1061': ('no item of that name in the module', ['MOD-3']),
    'E2011': ('negative literal index', ['TYP-31', 'LEX-24']),
    'E2071': ('`SoA[T]` of a type that is not a struct', ['SOA-1']),
    'E2072': ('no method or field of that name (with the Ember name for a Python one; shape N13 for an undeclared field)', ['STD-13']),
    'E2101': ('derived class with a field that has no default needs an `init`', ['CLS-10']),
    'E2181': ('`?` on an `Option` in a `Result` function, or the reverse', ['ERR-2']),
    'E2229': ('class generator method takes `mut self` or holds an access across `yield`', ['CORO-12']),
    'E2230': ('a name assigned in every branch at different types', ['CTL-10']),
    'E2240': ('`/` on two integers', ['TYP-28']),
    'E2250': ('format spec does not apply to the value\'s type', ['LEX-19']),
    'E2260': ('`some` type outside a return position', ['TYP-32']),
    'E2261': ('returns of a `some` function have different types', ['TYP-32']),
    'E4002': ('`@nosync` function reaches a synchronising operation', ['EFF-5']),
    'E4003': ('`@noblock` function reaches a blocking operation', ['EFF-5']),
    'E6004': ('panic during compile-time evaluation', ['CT-7']),
    'E6005': ('`comptime(e)` refers to a run-time local', ['CT-6']),
    'E7002': ('`static` of a type that is not `Sync`', ['STA-1']),
    'E7003': ('write to a field of a `@sync` class after `init`, or a `mut self` method on one', ['THR-1']),
    'E7004': ('value crossing a thread boundary is not `Send`', ['THR-10', 'THR-11', 'FFI-22']),
    'E7005': ('value shared with a task is not `Sync`', ['THR-11']),
    'E7006': ('memory order an atomic operation does not support', ['THR-14']),
    'E9036': ('a reloadable package may not link the runtime statically', ['HR-29']),
    'E9040': ('two generic instances hash to one symbol', ['MNG-1']),
    'E9041': ('the toolchain cannot honour `@fastmath` or `@fp(…)` for a function', ['TYP-9c']),
    'E5066': ('an overlay marks a C++ function both `noexcept` and throwing', ['FFI-43']),
    'W2016': ('`debug_assert` argument has side effects', ['PAN-1']),
    'W2190': ('unused `Result`', ['ERR-5']),
    'L3019': ('handle to an object with an observable `drop` is bound and never read', ['RC-3']),
    'L2004': ('`gen fn` with no `yield`', ['LNT-4']),
    'L2005': ('`@noreload` function calls a reloadable one in a loop', ['LNT-5']),
    # added in 0.9.9_Hardened_2
    'E2073': ('`len` of a string (Python counts characters, `s.len()` bytes)', ['STD-26']),
    'E2102': ('derived `init` uses `self` or an inherited field before `super.init`, or leaves an own field unassigned', ['CLS-11']),
    'E2231': ('`yield` while a `@must_drop` value is live', ['CORO-13']),
    'E2182': ('a function that returns a value can reach the end of its body', ['FN-10']),
    'E5067': ('an overlay that states facts is not declared `unsafe overlay`', ['GRM-35', 'TIER-1']),
    'W1003': ('a package module shadows the standard module of the same name', ['MOD-3']),
    'L4003': ('large `Array[int]`/`Array[float]` in a hot loop whose values fit 32 bits', ['LNT-6']),
}


def registry():
    reg = open(os.path.join(ROOT, 'compiler', 'ember_diag', 'src', 'codes.rs'), encoding='utf-8').read()
    rows = {}
    for m in re.finditer(r'([EWL]\d{4}) = \(\w+, \d+, \w+, "([^"]*)", "([^"]*)"\)', reg):
        rules = re.findall(r'\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]', m.group(2))
        rows[m.group(1)] = (m.group(3), rules)
    return rows


def defined_ids():
    ids = set()
    for p in glob.glob(os.path.join(PARTS, 'p*.md')):
        ids.update(re.findall(r'^\* `\[([A-Z][A-Z0-9-]*-[0-9]+[a-z0-9]*)\]`', open(p, encoding='utf-8').read(), re.M))
    return ids


def table():
    rows = registry()
    rows.update(OVERRIDE)
    rows.update(NEW)
    live = {c: v for c, v in rows.items() if c not in RETIRED}
    return live


def check():
    live = table()
    ids = defined_ids()
    named = set()
    for p in glob.glob(os.path.join(PARTS, 'p*.md')):
        if p.endswith(('p17a-codes.md', 'p25-appx-g.md', 'p26-appx-h.md')):
            continue
        text = open(p, encoding='utf-8').read()
        text = re.sub(r'`[EWL][0-9]{4}`\s*[–-]\s*`[EWL][0-9]{4}`', '', text)  # ranges, not codes
        named.update(re.findall(r'\b([EWL][0-9]{4})\b', text))
    bad = 0
    for c in sorted(named - set(live) - set(RETIRED)):
        print('NAMED-NOT-REGISTERED', c); bad += 1
    for c in sorted(named & set(RETIRED)):
        print('NAMED-BUT-RETIRED', c); bad += 1
    for c, (t, rules) in sorted(live.items()):
        for r in rules:
            if r not in ids:
                print('STALE-RULE', c, r, t); bad += 1
    print('live codes', len(live), 'retired', len(RETIRED), 'problems', bad)
    return bad


def write():
    live = table()
    key = lambda c: (c[0] != 'E', c[0], c)
    out = ['', '## XVII.9 The diagnostic code registry', '',
           'Every diagnostic code, what it reports and the rule it enforces (`[DIA-6a]`). Codes are never',
           'reused; retired codes are listed with the reason.', '',
           '| Code | Reports | Rule |', '|---|---|---|']
    for c in sorted(live, key=key):
        t, rules = live[c]
        out.append(f'| `{c}` | {t.replace("|", chr(92) + "|")} | ' + ', '.join(f'`[{r}]`' for r in rules) + ' |')
    out += ['', '**Retired codes.**', '', '| Code | Why |', '|---|---|']
    for c in sorted(RETIRED, key=key):
        out.append(f'| `{c}` | {RETIRED[c]} |')
    open(os.path.join(PARTS, 'p17a-codes.md'), 'w', encoding='utf-8', newline='\n').write('\n'.join(out) + '\n')
    print('wrote p17a-codes.md')


if __name__ == '__main__':
    if '--write' in sys.argv:
        write()
    sys.exit(1 if check() else 0)
