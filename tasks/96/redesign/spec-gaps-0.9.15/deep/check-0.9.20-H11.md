# Check of 0.9.20_Hardened_11 against the eight fixes (the local session's lead, solo, 2026-11-11)

The owner: "check now". Checked: commit `1a78917d`, `docs/spec-source/Ember_v0.9.20_Hardened_11.md`
(SHA `703D9713…`), against `fixes-0.9.20-H10.md`. Tests ran on this laptop: Clang 22.1.8 on Windows,
and GCC 15.2 and Clang 21.1.8 in WSL.

**Applied in 0.9.20_Hardened_12** (the owner, verdict 176: "option A and then fix the rest yourself"),
as N165 to N170 (`decisions.md` §23). Item 4 needed no test change: the fixture test already marks
the fixture as one the compiler does not build yet.

## What holds

- **The files.**
  - `spec_history.py verify`: 133 cuts rebuilt, 0 mismatches.
  - The live `parts/` equal the parts Hardened_11 records.
  - The pinned file's hash matches `development-target.json`.
- **The gates.** All ten pass against Hardened_11: check_ids, check_citations, code_registry,
  split_inline, check_examples, appx_h, rule_index, spec_check and phase_progress. `appx_h` rewrites
  no part.
- **The eight fixes are in, as written**, for items 1, 3, 4, 5 (`[MAN-10]`, `[FFI-51]`, `[BLD-FFI-3]`),
  6 (`[FFI-54]`, `[FFI-27]`, the cost row), 7 (`[FFI-57]`, `E5035`) and 8. `[FFI-6]` drops `link`
  and has the message.
- **The lead's addition to item 8 is right.** It adds "and which no header and no named library asks
  for". Without it, a library that item 5 makes the manifest name would be `E9001` for being named.

## What to fix (six; 1 and 3 are gaps in my own fixes)

### 1. With GCC, a header that only declares can still read a file into the program (`[FFI-51]`; my fix 3 was incomplete)

**Tested (WSL, GCC 15.2).** Two headers that `[FFI-51]` lets in with no `unsafe`:
- **A `weakref`.** `static int g(void) __attribute__((weakref("real_target\n\t.incbin \"…/secret.txt\"\n\t#")));`
  is a declaration of a function. Once the shim calls `g`, GCC writes the string as it is, and the
  file goes into the object, at `-O0` and at `-O2`.
- **An `#ident` line.** `#ident "v1\"\n\t.incbin \"…/secret.txt\"\n\t#"` does the same. `[FFI-51]`'s
  closed list already counts it as running, but Clang's AST, which the importer reads, does not show
  `#ident` at all.

Clang quotes both strings and reads nothing. The new label clause covers only `asm` labels.

**Fix (text for `[FFI-51]`).** "Where the C compiler writes assembly text and runs an assembler on it
(GCC), the build reads the text of the shim and of the thunk unit as `[MAN-10]` reads a dependency's.
A directive there that reads a file or makes directives makes the header one that runs, so the
import is `unsafe import`, or `E3100` naming the directive and the header line it came from."

This reads GCC's actual output, as item 3 does for a dependency, so it covers any attribute GCC has
or adds later, not only these two. Cost: one `-S` step and a scan, for a GCC compile only. Nothing
for Clang, the bundled compiler.

### 2. `[FFI-57]` now says two things about one call

The new rule says a call "stands for the member C++ chooses". The older paragraph after it still says
this: "two members … alike on this target (`put(int)` and `put(long)` for an `i32` argument where
`long` is 32 bits; a literal still takes `int`), fit such a call together", which is `E5035`.

**Tested (Clang 22.1.8, Windows, where `long` is 32 bits).** For an `int32_t` argument and for `7`,
C++ chooses `put(int)` (`static_assert` on the result type).

**Fix.** The paragraph is about uses without a call, where the member is chosen by its first line
and two members can share one. Reword it so:
- for a call, Clang decides, as for any other (`put(int)` for an `i32`);
- for a function value, or a line of the overlay, two members alike as the import spells them are
  told apart by the overlay, and `E5035` gives that line.

### 3. The runtime's own libraries are listed nowhere (`[BLD-FFI-3]`; my fix 5's text)

The exemption reads: "the C runtime and the system libraries `ember_rt` needs (`[RT-1]`), which the
toolchain names for each system (`[TOOL-2]`)". `[TOOL-2]` names none, and neither do `[RT-1]` or
`[TCB-4]`. So which libraries are exempt is left to the implementation.

What the build links today:
- **Linux:** `-lm -pthread` (`compiler/ember_build/src/lib.rs:1093`).
- **Windows:**
  - the C runtime the profile picks (`libcmt`, `msvcrt` or their debug forms, `lib.rs:2443`);
  - `kernel32`;
  - `synchronization.lib`, which `ember_rt.c` asks for with `#pragma comment(lib, …)`.

  `rand_s` is the C runtime's, so `advapi32` is not needed.

**Fix.** Write the list in `[RT-1]` and cite it in `[BLD-FFI-3]` in place of `[TOOL-2]`:
- **Windows:** the C runtime the profile chooses, with `ucrt`, `vcruntime` and `oldnames`, plus
  `kernel32` and `synchronization`.
- **Linux:** `libc`, `libm` and `libpthread`.
- **macOS:** `libSystem`.

A library outside the list is named under `unsafe_libs`.

### 4. Annex A shows zlib in the one way it is never used (`N158`, the lead's choice)

`import c "zlib.h" as zlib` is legal only while nothing links zlib. Any call into zlib needs
`unsafe_libs = ["z"]`, and with that the same line is `E3100`.

The lead's reason: the fixture test compiles the line, and today's compiler does not build
`unsafe import`. That lets today's compiler shape the spec. The owner's rule is the other way round:
tests follow the spec.

**Fix.**
- The fix as proposed: a `# SAFETY:` line, `unsafe import c "zlib.h" as zlib`, and
  `unsafe_libs = ["z"]` in the fixture's manifest.
- The fixture test drops the word `unsafe` before `import c` until the compiler builds it, as
  `check_examples.py` now does.

### 5. The removed-spellings check misses most ways to write `link`

`spec_check.py`'s pattern `with \(link\s*=` finds `with (link=["z"])` only. **Tested:** it misses
these three:
- `with (defines=["X"], link=["z"])`
- `with ( link = ["z"])`
- `with(link=["z"])`

**Fix:** the pattern `\bwith\s*\([^)]*\blink\s*=`.

### 6. Two small wordings

- **The `E9001` row.** It says "a library named there that nothing the package imports or links
  uses". The rule judges by what is declared, not used: "of which no import and no `unsafe extern`
  block declares a name, and which nothing asks for".
- **`[FFI-57]`'s literal.** It says "an untyped literal as it is written, `c_int` for a whole
  number". **Tested:** `3000000000` written as it is is C++'s `long` on Linux and `long long` on
  Windows. Ember's `1_000` is not C++ at all. **Fix:** "written as C++ spells its value, so C++ gives
  it its own type (`int` for `7`)".

## Whose call

- Items 1, 2, 3, 5 and 6 make a promise already written hold. The other agent can apply them.
- Item 4 reverses the lead's choice in favour of the owner's rule that tests follow the spec. I
  recommend it; the owner can keep the lead's form instead.
