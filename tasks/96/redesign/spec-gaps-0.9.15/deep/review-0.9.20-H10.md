# Review of 0.9.20_Hardened_10: the changes since Hardened_5 (the local session's lead, solo, 2026-11-10)

The owner: "done review changes, latest hardened version is 10 and it's present locally". Target:
`docs/spec-source/Ember_v0.9.20_Hardened_10.md` (commit `a442eb04`, branch `spec-0.9.20-hardening`,
local, not pushed; SHA-256 `D88B6EF1D687…`, 18,234 lines, 1,020 rules). Read: every line that
differs between `tasks/spec-0.9.9/parts-0.9.20-h5` and `parts-0.9.20-h10` (543 lines in 11 files,
about 15 rules), each tried for a program it lets through that breaks a promise, a correct program
it refuses, and a clash with another rule. Two claims were run, not reasoned: §3 items 1 and 5.

## 1. What changed, cut by cut

| Cut | What | My verdict |
|---|---|---|
| H6 (verdict 170) | my three holes and four notes (review-0.9.20-H5.md §8), N147-N152 | see §2 |
| H6 | N150: verdict 38 put back. A parameter of a function type is the shared type, one body and one indirect call; only a function given through a type parameter (`[F: needs fn(Item)]`) is compiled once per function given (`[EFF-2]`, `[EFF-2a]`, `[COST-3]`) | **agree**, and a correction of my H5 review: I listed "`[EFF-2a]` compiles a function-typed parameter once per function given" as a cost to measure; it was a silent departure from the owner's verdict 38 (Hardened_2), and the lead caught it. `[CLO-3]` agrees with the restored text |
| H7 (verdict 171) | no header is exempt from `unsafe import`, the toolchain's own included; judged by what the compiled header leaves in the program on its target | **agree**: general, and judged by output rather than a list of spellings |
| H8 (verdict 172) | every linked library is named under `[link] unsafe_libs`, and the import of a header declaring a name it defines is `unsafe import`; `libs` refused; the import option `link = [...]` removed | agree; the removal is not finished in the text (§3 item 2) |
| H9 (verdict 173) | a library a header asks the linker for is named too (`windows.h`: `uuid`) | agree; its sibling case is open (§3 item 5) |
| H10 (verdict 174) | `[link.windows]`, `[link.linux]`, `[link.macos]`, each taking `unsafe_libs` | agree |

## 2. My three holes and notes: fixed the better way?

| Item of my H5 review | Hardened_6 | Verdict |
|---|---|---|
| Hole 1, hot reload's list across threads (`[HR-12]`) | N147: an object of a `Send` type is registered the ordered way, a type that is not `Send` keeps plain stores, read from the object's own type; four stores, not two; `[HR-13]` containers the same | **fixed**, as I proposed |
| Hole 2, code before `main` through a start-up section (`[FFI-51]`) | N148, then verdict 171: a header with no `unsafe` "only declares", and everything else counts as running, judged by what the compiled header leaves (sections, constructors and destructors, `ifunc`, definitions the linker sees, assembly, orders to the linker) | **fixed, better than my proposal**: mine listed spellings; this judges the result |
| Hole 3, a dependency's C reading the builder's files (`[CT-2]`, `[MAN-10]`) | N149: reads only its own root and the toolchain's headers, checked against what the compiler reports it read, `#embed` included; no assembly; each flag one argument | **fixed in design, one gap**: links are not followed (§3 item 1) |
| Note, `[FFI-57]`'s "nearer type" | N151: number types rank as in C++ (own type, a promotion, every other alike). The lead found my premise wrong too: "the narrowest type" chose `set(short)` for an `i8` where C++ chooses `set(int)` | **fixed, better than my note** |
| Notes, form (`[BLD-16]` tags, self-citation) | N152 | fixed |

## 3. New problems found

1. **Hole, run: a link inside a dependency reads a file outside it (`[MAN-10]`).** The new sentence
   says a dependency's C "reads only files under the dependency's own root", checked against "the
   files the C compiler reports it read". It does not say links are followed, as the same rule says
   for `-I` paths and `[CT-2]` says for `comptime.read_file`. A C compiler reports the path it
   opened, not the real file. Run here with Clang 22.1.8: a folder link `dep/inc` pointing at a
   folder outside the dependency, and `#embed "inc/secret.txt"` in `dep/lexer.c`. The `-MD` report
   names `dep\inc\secret.txt`, under the dependency's root, and the object file holds the secret.
   A git dependency can carry such a link. **Fix:** "with every symbolic link followed", as the
   `-I` rule and `[CT-2]` already say: the build compares the real path of each file read.
2. **Left over from verdict 172: the removed `link` option is still in the text.** `[FFI-6]` still
   lists `link` among an import's options ("Options: `include_paths`, `defines`, `link`, `overlay`,
   `implementation`"), against `[FFI-49]` ("by nothing else"). Annex A's quick reference, and its
   fixture `docs/spec-source/appendix-a.em` (line 16), still write `import c "zlib.h" with
   (link=["z"]) as zlib`. Under verdict 172 that line is `unsafe import c "zlib.h" as zlib` with a
   `# SAFETY:` note, and `unsafe_libs = ["z"]` in the manifest. **Fix:** drop `link` from `[FFI-6]`;
   rewrite the Annex A line and its fixture.
3. **What "assembly" means is not said, and the pilot ran on Windows only (`[FFI-51]`, `[MAN-10]`).**
   Both rules count assembly "at file scope or in a function's body". On Linux, glibc's `<stdio.h>`
   renames `fscanf`, `scanf` and `sscanf` with a file-scope `__asm__` label (its `__REDIRECT` macro,
   to `__isoc99_fscanf`). This is from glibc's source; I could not run it here. MinGW's headers hold
   inline assembly in `static inline` functions. If an asm label counts, every Linux `import c` that
   reaches `<stdio.h>` is `unsafe import`, and every Linux dependency with C needs `native`. If
   toolchain-header assembly counts for `[MAN-10]`, so does every MinGW one. **Fix:** an asm label
   that names a plain symbol is a declaration, not assembly. `[MAN-10]`'s ban on assembly applies to
   files under the dependency's root, the only files that could `.incbin` something the dependency
   chose. A pilot on Linux (glibc) and MinGW is owed.
4. **Which folders "the toolchain's headers" are on Linux (`[MAN-10]`).** There `/usr/include` holds
   glibc's headers and every installed library's (`zlib.h`, `openssl/`). If it counts as "the
   platform kit's", a dependency that wraps a system library compiles as it does today. If not, every
   such wrapper needs `native` in the root. **Fix:** say which. My recommendation: the compiler's
   default include directories count. They hold headers, not secrets, and the dependency cannot
   write to them.
5. **Libraries asked for by a library, and the runtime's own (`[FFI-49]`, `[BLD-FFI-3]`; needs the
   owner's word).** "Libraries are linked by `[link] unsafe_libs` … and by nothing else" is not true
   in two cases:
   - **The runtime's own libraries.** `[RT-1]` says the runtime depends on libc and the OS, and
     those are linked with no entry.
   - **What a named library asks for.** A static library compiled the MSVC way asks the linker for
     more libraries. Run here: a one-line C file compiled with `clang-cl` carries
     `/DEFAULTLIB:libcmt.lib /DEFAULTLIB:oldnames.lib`, and libraries such as libcurl add
     `ws2_32` and others the same way.

   Verdict 173 named the libraries a header asks for. It says nothing of those a named library asks
   for. The same record means the importer must compile its shims without the compiler's own
   default-library names (`/Zl`); otherwise every shim leaves "an order to the linker" and every
   import becomes `unsafe import`. **Recommendation:**
   - the runtime's own C and OS libraries (`[RT-1]`) are the runtime's, listed under it by `ember
     tcb`, and need no entry;
   - a library a named library asks for is named too, as verdict 173 has for a header (`E9001` with
     the line to add), since it brings code nothing checks;
   - shims are compiled without default-library names.
6. **Which thread C may end an object on (`[FFI-54]`, `[FFI-27]`; old, made visible by N147).** C ends
   an object it was given "on an attached thread", any thread. For a type that is not `Send`, ending
   it on another thread races on its plain counts, and in a reloadable build on its plain list. N147
   also now keeps a dead thread's list "for what unsafe code gave to C from it". **Fix:** state C's
   obligation (an object of a type that is not `Send` is ended on the thread that made it, or, after
   that thread ends, by one thread at a time), listed with the `unsafe:` note. `ember_rt_object_release`
   unlinks the ordered way, since it is never on a hot path.
7. **`[FFI-57]` still promises more than it does.** "Where C++ chooses one Ember chooses the same" is
   not true for C++'s two tie-breaks. A non-template function beats a template instance that is
   alike in every parameter. A more specialised template beats a less specialised one (`f(T*)` over
   `f(T)` for an `int*`). Ember refuses both (`E5035`), and never picks a different member, so it is
   not a hole. **Fix:** add the two tie-breaks, or say Ember refuses there.
8. **A library named but used by no import (verdict 172; the owner's word).** `unsafe_libs =
   ["foo"]` with no `unsafe import` or `unsafe extern "C":` that uses it still brings code. With
   LLD's default (no `--as-needed`), a shared library is still loaded, and its start-up code runs.
   The manifest then says "unsafe library" and the code says nothing. **Recommendation:** refuse it,
   naming the line ("`unsafe_libs` names `foo`, which no import uses; remove it, or import its
   header with `unsafe import`").

## 4. Verdict

Hardened_6 to _10 hold up. My three holes are closed: two as I proposed, one better (`[FFI-51]`
judged by what the compiled header leaves). The lead also put back verdict 38, which my H5 review
had missed, and corrected my `[FFI-57]` note. Verdicts 171 to 174 are in as the owner wrote them.

Left: one real hole (item 1, a one-phrase fix, run), one leftover of verdict 172 (item 2), four
precisions (items 3, 4, 6, 7) and two questions that extend the owner's verdicts 172 and 173 (items
5 and 8). None of them is in the rules of ownership, threads or types.
