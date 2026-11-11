# Fixes for the eight items of review-0.9.20-H10.md (the local session's lead, solo, 2026-11-10)

The owner: "I need you to come up with a strong fix for these". Each fix below was tested before it
was written, unless it says otherwise. Tests ran on this laptop: Clang 22.1.8 and clang-cl on
Windows, and GCC 15.2 and Clang 21.1.8 with glibc 2.43 in WSL Ubuntu. Nothing is applied to the spec.

**Whose call.**
- Items 1, 2, 3, 4 and 6 make an existing promise hold. They are precisions the other agent can
  apply.
- Items 5, 7 and 8 are language decisions, the owner's, one at a time:
  - 5 and 8 extend verdicts 172 and 173;
  - 7 replaces the lead's K21 ranking rule.

## 1. A link inside a dependency reads a file outside it (`[MAN-10]`)

**Fix (text for `[MAN-10]`).** "Each file the C compiler reports it read is opened, and its real
path is asked of the system: every symbolic link, junction and `..` resolved, and on Windows the
case and short names made canonical. It must lie under the real path of the dependency's root or of
a toolchain folder (item 4). A file under the dependency's root with more than one hard link is
refused too, since its real path names the copy inside. A package cache must copy files, not hard
link them."

**Why strong.** It judges the file actually read, not the name it was reached by. A folder link, a
symbolic link, `..`, or Windows's short and differently cased names all reduce to one real path. The
link count closes the one case a real path cannot see.

**Tested.**
- A junction `dep/inc` points at a folder outside `dep`, and `#embed "inc/secret.txt"` sits in
  `dep/lexer.c`. Clang's `-MD` names `dep\inc\secret.txt`, and the object holds the secret.
- A 10-line check, run on these files:

| File | Real path | Hard links | Result |
|---|---|---|---|
| `dep/own.h` | inside | 1 | allowed |
| `dep/inc/secret.txt` (the junction) | outside | 2 | refused |
| `dep/hard.txt` (a hard link) | inside | 2 | refused, by the link count alone |

**Cost.** One open and one system call per file the C compile read, in a dependency's build only.

## 2. The removed `link` import option is still in the text (`[FFI-6]`, Annex A)

**Fix.**
- `[FFI-6]`'s options become `include_paths`, `defines`, `overlay`, `implementation`. The message
  for `link` (`E0104`) says where libraries go: "`link` is not an option of `import c`; name the
  library under `[link] unsafe_libs` in `ember.toml`".
- Annex A's line and its fixture (`docs/spec-source/appendix-a.em`, line 16) become a
  `# SAFETY:` line and `unsafe import c "zlib.h" as zlib`, and the fixture's manifest names
  `unsafe_libs = ["z"]`.
- `unsafe import c` is added to the unbuilt-features list in `check_examples.py` (`NEW_SYNTAX`) and
  to the fixture check in `spec_check.py`. **Tested:** today's compiler refuses `unsafe import c "zlib.h"
  as zlib` (`E0100`, not built yet) and still accepts the removed `with (link=["z"])`. So without
  this, the corrected fixture fails its check, which is probably why it was left alone.

**Stops it happening again.** The examples are checked by today's compiler, which still accepts
every removed spelling. So a removal in the text is never caught there. Add one table of removed
spellings: a pattern, the version that removed it, and the rule. `spec_check` then fails when an
example, a fixture or a rule outside Appendix H contains one. First rows: `with (link=`, `[link]`
`libs =`.

## 3. What "assembly" is: glibc's labels, and file reads GCC copies into its output (`[FFI-51]`, `[MAN-10]`)

**Tested (WSL, glibc 2.43):**
- **glibc's labels.** `#include <stdio.h>` leaves five file-scope `__asm__` labels in the file
  (`__asm__ ("" "__isoc23_fscanf")`, and the same for `scanf`, `sscanf`, `vfscanf`, `vscanf`). The
  object holds only an undefined reference to `__isoc23_fscanf`. As written today, "assembly at file
  scope" makes every Linux import of `<stdio.h>` an `unsafe import`.
- **A label can read a file, with GCC.** A label on a declaration
  (`__asm__("f\n\t.incbin \"/tmp/embt/secret.txt\"\n\t#")`) puts the file into the object, and GCC's
  `-MD` report names only `b.c`.
- **So can a section name and a `symver`, with GCC.** Both put the file into the object the same
  way.
- **Clang writes all three quoted**, as a symbol named with the whole text, and reads nothing.
- **An assembler macro beats a text check.** An `asm` statement that defines a macro building
  `.incbin` out of `.inc` and `bin` reads the file with both compilers. The word `incbin` appears in
  neither compiler's assembly text, and Clang's shows no `.macro` either.

**Fix.**
- **`[FFI-51]` (the root's imports).** "An `asm` label whose text, after preprocessing, is one plain
  symbol name only renames the declaration it is on. A plain symbol name is made of letters,
  digits, `_`, `$`, `.` and `@`, and does not start with a digit. Such a label is judged as that
  declaration (glibc's `__isoc23_fscanf`, Apple's `fopen$DARWIN_EXTSN`); any other label is
  assembly."
- **`[MAN-10]` (a dependency's C), two parts.**
  - Keep the ban on `asm` statements, at file scope and in bodies, judged after preprocessing. The
    macro test shows it cannot be replaced by reading the output.
  - Add: "where the C compiler writes assembly text and runs an assembler on it (GCC), the build
    reads that text before it is assembled. It refuses the text when it holds, outside a quoted
    name, a directive that reads a file or makes directives: `.incbin`, `.include`, `.macro`, `.irp`,
    `.irpc`, `.rept`, `.altmacro`. GCC never writes these itself, so one there came from a string
    the source gave it: a label, a section name, a `symver`, or an attribute not yet known. Clang
    writes such strings quoted, and needs no such read."

**Why strong.**
- The root's rule judges the label by what it does, so glibc's and Apple's headers pass and
  nothing else does.
- The dependency rule doesn't list GCC's attributes. It reads GCC's actual output, so an attribute
  added in a later GCC is covered too.

**Cost.**
- One `-S` step, then a scan of the assembly text, for a dependency's C compiled with GCC; nothing
  for Clang.
- A dependency whose C calls a toolchain function holding `asm` (rare: Clang's intrinsics are
  built in) needs `native`.

## 4. Which folders "the toolchain's headers" are (`[MAN-10]`)

**Fix (text).** "The toolchain's headers are in the folders the C compiler searches for
`#include <…>` when it is given no flag from any manifest, and none of the environment variables
that add folders (`INCLUDE`, `CPATH`, `C_INCLUDE_PATH`, `CPLUS_INCLUDE_PATH`, `OBJC_INCLUDE_PATH`).
They are taken as the compiler lists them itself (`-E -v`; for MSVC's `cl`, the MSVC and Windows SDK
installations the toolchain finds, `[TOOL-2]`), each by its real path. The build compiles a
dependency's C with those variables removed."

**Tested.** The lists, with those variables removed:

| Compiler | Folders it lists |
|---|---|
| Windows Clang | its own folder, MSVC 14.51's `include`, and the Windows Kits' `ucrt`, `shared`, `um`, `winrt` and `cppwinrt` (found without `INCLUDE`) |
| WSL GCC and Clang | their own folder, `/usr/local/include`, `/usr/include/x86_64-linux-gnu`, `/usr/include` |

`/usr/include` holds `zlib.h`, and it and `/usr/local/include` are owned by root, mode 755. So a
dependency wrapping a library the system installed needs no `native`. The environment can no longer
add a folder, which also makes the build reproducible.

## 5. Libraries asked for by a library, and the runtime's own (`[FFI-49]`, `[BLD-FFI-3]`) — the owner's word

**Tested.**
- A one-line C file compiled the MSVC way asks for `/DEFAULTLIB:libcmt.lib /DEFAULTLIB:oldnames.lib`.
  Every static library built so carries the same requests, and libraries such as libcurl add more.
- The same file compiled with `/Zl` asks for nothing.
- A file with `#pragma comment(lib, "uuid.lib")`, compiled with `/Zl`, asks for `uuid.lib` only, and
  `#include <windows.h>` gives the same.

**Fix (text for `[BLD-FFI-3]`).**
- **The runtime's libraries.** "Besides the libraries the manifest names, a build links only the
  runtime's own: the C runtime and the system libraries `ember_rt` needs (`[RT-1]`). The toolchain
  names them for each system (`[TOOL-2]`), and `ember tcb` lists them under the runtime."
- **Libraries something asks for.** "Every other library the link would add because something
  asked for it is named under `unsafe_libs` too: a header (verdict 173), or an object of a named
  library (`/DEFAULTLIB` in a COFF `.drectve` section, `.deplibs` in ELF, `LC_LINKER_OPTION` in
  Mach-O). Otherwise it is `E9001`, naming the library, what asked for it, and the line to add."
- **Shims.** "Shims and thunk units are compiled without the compiler's own default-library names
  (`/Zl`), so a request left in them is the header's."
- **Shared libraries.** "What a shared library loads when it runs is that library's code, vouched
  for with it, and `ember tcb` lists it under it."

**Why this line.** The manifest names everything the program's own link adds. The link step is
where a request can be refused, so it is the one place naming means something. What a DLL loads by
itself, the manifest could not stop anyway.

**Without the `/Zl` part,** every shim carries `libcmt` and becomes `unsafe import` under
`[FFI-51]`'s "order to the linker".

## 6. Which thread C may end an object on (`[FFI-54]`, `[FFI-27]`)

**Fix (text).**
- **Checked on its thread.** "`ember_rt_object_release` and `from_raw` of an object whose type is
  not `Send` run on the thread that made it. `into_raw()` records that thread: one entry in a
  runtime table, removed when C ends the object or Ember takes it back. Both compare it with the
  calling thread, in every profile, and stop the program otherwise ("C ended a `Widget` made on
  another thread; `Widget` is not `Send`: its `log` is a `shares` owner"). A `threads(creator)`
  trampoline already does the same (`[FFI-17d]`, `[FFI-22]`)."
- **After its thread ends.** "The objects of that thread C still holds are only ended, one at a time
  under the runtime's lock; `from_raw` of one stops the program."
- **`Send` types.** "An object of a `Send` type may be ended or taken back on any attached thread."

**Why strong.** It is checked, not left as a promise. Cost: one table entry per object of a
non-`Send` type handed to C, and one compare when C gives it back. Both are on paths that hand an
object to C, never a hot loop. Not run: the runtime does not build this yet.

## 7. `[FFI-57]` and C++'s tie-breaks — the owner's word (it replaces the lead's ranking rule)

**Tested (Clang 22.1.8).** Clang names the member it chose for each call:

| Call | Clang's choice | What the rule today gives |
|---|---|---|
| `w.set(a)`, `a` an `i8` | `set(int)` | the same |
| `w.g(b)`, `b` an `int` | the plain `g(int)` over the template | `E5035` |
| `w.h(p)`, `p` an `int*` | the more specialised template `h(T*)` (seen by its `int` result) | `E5035` |
| `w.put(b)` | ambiguous, both candidates named | the same |

A Clang parse of `<string>`, `<vector>` and `<map>` takes 257 ms, and 612 ms with the output read.

**Fix (text for `[FFI-57]`, replacing the ranking paragraph).**
- "The member a call stands for is the one C++ chooses. The importer writes each call of a name with
  several members as a C++ call in the package's thunk unit, with the arguments' C++ types and words:
  `owns` an rvalue, `ref mut` a non-const lvalue, none a const lvalue, and an untyped literal as
  written. Clang's own overload resolution, which the importer already runs (`[FFI-6]`), names the
  member."
- "A call Clang finds ambiguous, or that fits no member, is `E5035` with Clang's candidates."
- "A member Clang reaches only through a conversion Ember does not make (a converting constructor,
  a user conversion) is `E5035` too, naming it."
- "So Ember chooses what C++ chooses or refuses, never another."

**Why strong.** It is exact by construction: every C++ rule, tie-breaks, references and templates
included, with no Ember copy of C++'s ranking to drift. It also replaces about 30 lines of the spec
with five.

**Cost.** One Clang parse per package of its overloaded calls, cached on their text.

## 8. A library named but used by no import (verdict 172) — the owner's word

**Tested (WSL).** `libfoo.so` has a start-up function and is linked into a program that never calls
it:

| Linker | Default | `--as-needed` |
|---|---|---|
| LLD | its start-up code runs (`NEEDED: 1`) | not loaded |
| GNU ld | its start-up code runs (`NEEDED: 1`) | not loaded |

**Fix (text for `[BLD-FFI-3]`).** "A library named under `unsafe_libs` of which no import and no
`unsafe extern` block of the package declares a name is `E9001`: '`unsafe_libs` names `foo`
(`ember.toml`, line 7), which nothing in this package imports, and its start-up code would still run
when the program loads; remove the line, or import its header with `unsafe import`'."

**Why "declares", not "calls".** A library used only for its start-up code (a plugin that registers
itself) stays possible, with the code saying so. It also uses the same names the build already reads
for verdict 172.
